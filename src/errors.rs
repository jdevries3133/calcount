//! Allow conversion from [anyhow::Error] to [ServerError], which is the error
//! type returned from all of our route handlers. Since [ServerError]
//! implements [axum::response::IntoResponse], we're able to return
//! [anyhow::Error] right out of our route handlers with this little bit of
//! code; allowing good `?` ergonomics throughout error-generating code paths.

use anyhow::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Generic 500 error if we bubble up out of HTTP request handlers.
#[derive(Debug)]
pub struct ServerError {
    /// The actuall error, which will be logged.
    err: Error,
    status: StatusCode,
    /// Public-facing response message
    response_message: &'static str,
}
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        println!("HTTP {} {:?}", self.status, self.err);
        (self.status, self.response_message).into_response()
    }
}
impl ServerError {
    pub fn forbidden(msg: &'static str) -> Self {
        ServerError {
            err: Error::msg(msg),
            status: StatusCode::FORBIDDEN,
            response_message: "Forbidden",
        }
    }
}

/// This enables using `?` on functions that return `Result<_, anyhow::Error>`
/// to turn them into `Result<_, AppError>`. That way you don't need to do that
/// manually.
impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self {
            err: err.into(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
            response_message: "something went wrong",
        }
    }
}
