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
    err: Option<Error>,
    status: StatusCode,
    /// Public-facing response message
    response_body: String,
}
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        println!("HTTP {} {:?}", self.status, self.err);
        (self.status, self.response_body).into_response()
    }
}
impl ServerError {
    /// This can be used for things like bad requests or 404 errors, where
    /// nothing is really "wrong," it's just the expected beahvior of the
    /// API.
    pub fn forbidden(msg: &'static str) -> Self {
        ServerError {
            err: Some(Error::msg(msg)),
            status: StatusCode::FORBIDDEN,
            response_body: "Forbidden".into(),
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
            err: Some(err.into()),
            status: StatusCode::INTERNAL_SERVER_ERROR,
            response_body: "something went wrong".into(),
        }
    }
}
