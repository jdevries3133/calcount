use super::post::PostSummary;
use crate::prelude::*;

impl Component for PostSummary {
    fn render(&self) -> String {
        let title = clean(&self.title);
        let summary = clean(&self.summary);
        let href = Route::BlogPost(Some(self.id));
        format!(
            r#"
            <a href="{href}">
                <div class="prose m-2 p-2 bg-blue-100 dark:bg-blue-950
                    dark:text-slate-400 rounded">
                    <h1 class="text-lg dark:text-slate-200">{title}</h1>
                    <p>{summary}</p>
                </div>
            </a>
            "#
        )
    }
}

#[derive(Deserialize)]
pub struct PostListParams {
    page: Option<i64>,
}

pub async fn post_list(
    State(AppState { db }): State<AppState>,
    Query(PostListParams { page }): Query<PostListParams>,
) -> Result<impl IntoResponse, ServerError> {
    let limit: i64 = 100;
    let offset = limit * page.unwrap_or_default();
    let posts: Vec<PostSummary> = query_as!(
        PostSummary,
        "select
            id,
            title,
            summary
        from post
        order by created_at desc
        limit $1
        offset $2",
        limit,
        offset
    )
    .fetch_all(&db)
    .await?;

    Ok(Page {
        title: "Posts",
        children: &BrandedContainer { children: &posts },
    }
    .render())
}
