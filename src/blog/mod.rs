mod post;
mod post_list;
mod post_page;

pub use post_list::post_list;
pub use post_page::{
    handle_comment_submission, handle_delete_comment, post_page,
};
