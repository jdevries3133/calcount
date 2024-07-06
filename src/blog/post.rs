#[derive(Debug)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub post_markdown: String,
}

#[derive(Debug)]
pub struct PostSummary {
    pub id: i32,
    pub title: String,
    pub summary: String,
}

pub struct Comment {
    pub username: String,
    pub body: String,
}
