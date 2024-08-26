use super::post::{Comment, Post};
use crate::{
    auth::{is_anon, InitAnonNextRoute},
    components::{BackIcon, Brand, Span},
    config::ADMINISTRATOR_USER_IDS,
    prelude::*,
};
use axum::{http::StatusCode, response::Redirect};
use futures::join;

impl Component for Post {
    fn render(&self) -> String {
        let posts = Route::BlogPostList;
        let back_icon = BackIcon {}.render();
        let render_result = &markdown::to_html_with_options(
            &self.post_markdown,
            &markdown::Options::gfm(),
        );
        let html = match render_result {
            Ok(html) => html,
            Err(msg) => {
                let post_id = self.id;
                eprintln!("Error: failed to render {post_id} :: {msg}");
                "Something went wrong."
            }
        };
        format!(
            r#"
            <a class="link flex items-center gap-1 sm:py-3" href="{posts}">Back {back_icon}</a>
            {html}
            "#
        )
    }
}

struct CommentUI<'a> {
    comment: &'a Comment,
    can_delete: bool,
}
impl Component for CommentUI<'_> {
    fn render(&self) -> String {
        let username = if is_anon(&self.comment.username) {
            "anon".to_string()
        } else {
            clean(&self.comment.username)
        };
        let body = clean(&self.comment.body);
        let delete_button = if self.can_delete {
            let delete_route = Route::DeleteComment(Some(self.comment.id));
            format!(
                r#"
                <button
                    hx-delete="{delete_route}"
                    hx-target="closest div"
                    class="self-start text-xs p-1 my-2 bg-red-100
                    hover:bg-red-200 rounded-full text-black"
                >
                    Delete
                </button>
                "#
            )
        } else {
            "".into()
        };
        format!(
            r#"
            <div
                class="prose dark:prose-invert my-2 p-2 bg-blue-100
                    dark:bg-blue-950 rounded"
            >
                <p class="text-sm font-bold">{username}</p>
                <p>{body}</p>
                {delete_button}
            </div>
            "#
        )
    }
}

impl Component for [CommentUI<'_>] {
    fn render(&self) -> String {
        self.iter().fold(String::new(), |mut acc, comment| {
            acc.push_str(&comment.render());
            acc
        })
    }
}

struct CreateCommentForm {
    post_id: i32,
}
impl Component for CreateCommentForm {
    fn render(&self) -> String {
        let home = Route::UserHome;
        let submit = Route::BlogCommentSubmission;
        let post_id = self.post_id;
        format!(
            r#"
            <form
                class="my-2 p-2 bg-blue-100 dark:bg-blue-950 rounded flex
                flex-col max-w-md"
                method="POST"
                action="{submit}"
            >
                <input type="hidden" name="post_id" value="{post_id}" />
                <label for="comment_body">Comment</label>
                <textarea
                    id="comment_body"
                    name="comment_body"
                    required
                    placeholder="Leave a comment"
                    class="block p-2.5 w-full text-sm text-gray-900 bg-gray-50
                    rounded-lg border border-gray-300 focus:ring-blue-500
                    focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600
                    dark:placeholder-gray-400 dark:text-white
                    dark:focus:ring-blue-500 dark:focus:border-blue-500"
                ></textarea>
                <button
                    class="self-start text-xs p-1 my-2 bg-emerald-100
                    hover:bg-emerald-200 rounded-full text-black"
                >Submit</button>
                <a href="{home}">
                    <button
                        class="
                            bg-gradient-to-tr
                            from-blue-700
                            to-indigo-700
                            from-blue-100
                            to-indigo-200
                            p-2
                            rounded
                            shadow-md
                            hover:shadow-sm
                            font-extrabold
                            text-white
                            my-2
                        "
                    >Try Bean Count!</button>
                </a>
            </form>
            "#
        )
    }
}

struct NoAuthCommentActions {
    post_id: i32,
}
impl Component for NoAuthCommentActions {
    fn render(&self) -> String {
        let init_anon = Route::InitAnon(InitAnonNextRoute::CustomNextRoute(
            Box::new(Route::BlogPost(Some(self.post_id))),
        ));
        let login = Route::Login;
        format!(
            r#"
            <div class="flex gap-2">
                <form method="POST" action="{init_anon}">
                    <input type="hidden" value="" name="timezone" id="timezone" />
                    <button
                        class="self-start text-xs p-1 bg-emerald-100
                        hover:bg-emerald-200 rounded-full text-black"
                    >Create an account to comment</button>
                </form>
                <a class="text-xs" href="{login}">
                    <button
                        class="self-start text-xs p-1 rounded-full border-2
                        jorder-emerald-200"
                    >Or, login if you have an account
                    </button>
                </a>
            </div>
            <script>
                (() => {{
                    for (const el of document.querySelectorAll("[name='timezone'")) {{
                        el.value = Intl.DateTimeFormat().resolvedOptions().timeZone;
                    }}
                }})();
            </script>
            "#
        )
    }
}

struct PostPage<'a> {
    post: &'a Post,
    comments: &'a [Comment],
    user_id: Option<i32>,
}
impl Component for PostPage<'_> {
    fn render(&self) -> String {
        let post = self.post.render();
        let brand = Brand {}.render();
        let action = if self.user_id.is_some() {
            CreateCommentForm {
                post_id: self.post.id,
            }
            .render()
        } else {
            NoAuthCommentActions {
                post_id: self.post.id,
            }
            .render()
        };
        let comments =
            self.comments
                .iter()
                .fold(String::new(), |mut acc, comment| {
                    acc.push_str(
                        &CommentUI {
                            comment,
                            can_delete: self.user_id.map_or(false, |uid| {
                                uid == comment.user_id
                                    || ADMINISTRATOR_USER_IDS.contains(&uid)
                            }),
                        }
                        .render(),
                    );
                    acc
                });
        format!(
            r#"
            {brand}
            <div class="prose dark:prose-invert">
                {post}
            </div>
            <div>
                {action}
            </div>
            <div>
                {comments}
            </div>
            "#
        )
    }
}

#[derive(Deserialize)]
pub struct PostPageQuery {
    comment_page: Option<i64>,
}

pub async fn post_page(
    Path(id): Path<i32>,
    headers: HeaderMap,
    Query(PostPageQuery { comment_page }): Query<PostPageQuery>,
    State(AppState { db }): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers(&headers);
    let comment_limit: i64 = 100;
    let comment_offset = comment_limit * comment_page.unwrap_or_default();
    let post = query_as!(
        Post,
        "select
            id,
            title,
            post_markdown
        from post
        where id = $1",
        id
    )
    .fetch_optional(&db);
    let comments = query_as!(
        Comment,
        "select
            c.id,
            c.user_id,
            c.body,
            u.username
        from comment c
        join users u on u.id = c.user_id
        where c.post_id = $1
        order by c.created_at desc
        limit $2
        offset $3",
        id,
        comment_limit,
        comment_offset
    )
    .fetch_all(&db);
    let (post, comments) = join![post, comments];
    let post = post?;
    let comments = comments?;

    Ok(match post {
        Some(post) => (
            StatusCode::OK,
            Page {
                title: &post.title,
                children: &PageContainer {
                    children: &PostPage {
                        post: &post,
                        comments: &comments,
                        user_id: session.map(|s| s.user_id),
                    },
                },
            }
            .render(),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Page {
                title: "Post not found",
                children: &PageContainer {
                    children: &Span {
                        content: "This post does not exist".into(),
                    },
                },
            }
            .render(),
        ),
    })
}

#[derive(Deserialize)]
pub struct CommentForm {
    post_id: i32,
    comment_body: String,
}

pub async fn handle_comment_submission(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<CommentForm>,
) -> Result<impl IntoResponse, ServerError> {
    let session =
        Session::from_headers_err(&headers, "handle comment submission")?;
    query!(
        "insert into comment
        (
            user_id,
            post_id,
            body
        ) values ($1, $2, $3)",
        session.user_id,
        form.post_id,
        form.comment_body
    )
    .execute(&db)
    .await?;

    let post_route = Route::BlogPost(Some(form.post_id));

    Ok(Redirect::to(&post_route.as_string()))
}

pub async fn handle_delete_comment(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "handle_delete_comment")?;
    if session.is_administrator() {
        query!("delete from comment where id = $1", id)
            .execute(&db)
            .await?;
    } else {
        query!(
            "delete from comment
            where
                user_id = $1
                and id = $2",
            session.user_id,
            id
        )
        .execute(&db)
        .await?;
    }
    Ok("")
}
