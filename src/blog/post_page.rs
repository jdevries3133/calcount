use super::post::{Comment, Post};
use crate::{components::Span, prelude::*};
use axum::{http::StatusCode, response::Redirect};
use futures::join;

impl Component for Post {
    fn render(&self) -> String {
        let posts = Route::BlogPostList;
        let html = clean(&markdown::to_html(&self.post_markdown));
        format!(
            r#"
            <a class="link" href="{posts}">View All Posts</a>
            {html}
            "#
        )
    }
}

impl Component for Comment {
    fn render(&self) -> String {
        let username = clean(&self.username);
        let body = clean(&self.body);
        format!(
            r#"
            <div
                class="prose dark:prose-invert my-2 p-2 bg-blue-100
                    dark:bg-blue-950 rounded"
            >
                <p class="text-sm font-bold">{username}</p>
                <p>{body}</p>
            </div>
            "#
        )
    }
}

impl Component for [Comment] {
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
            </form>
            "#
        )
    }
}

struct PostPage<'a> {
    post: &'a Post,
    comments: &'a [Comment],
}
impl Component for PostPage<'_> {
    fn render(&self) -> String {
        let post = self.post.render();
        let comment_form = CreateCommentForm {
            post_id: self.post.id,
        }
        .render();
        let comments = self.comments.render();
        format!(
            r#"
            <div class="prose dark:prose-invert">
                {post}
            </div>
            <div>
                {comment_form}
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
    Query(PostPageQuery { comment_page }): Query<PostPageQuery>,
    State(AppState { db }): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
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
            u.username,
            c.body
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
