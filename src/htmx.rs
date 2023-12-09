//! HTMX utils

use axum::http::{HeaderMap, HeaderValue};

/// Inserts a `Hx-Redirect` header into the provided headers. Will panic if
/// `to` cannot be encoded as an [axum::http::HeaderValue].
pub fn redirect(mut headers: HeaderMap, to: &str) -> HeaderMap {
    headers.insert(
        "Hx-Redirect",
        HeaderValue::from_str(to)
            .unwrap_or(HeaderValue::from_str("/").unwrap()),
    );
    headers
}
