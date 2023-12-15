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

pub fn trigger_event(
    mut headers: HeaderMap,
    event_name: &'static str,
) -> HeaderMap {
    if headers.contains_key("Hx-Trigger") {
        let val = headers.get("Hx-Trigger").expect("we know it's here");
        let as_str = val.to_str().expect("existing trigger is ascii");
        let new_header = format!("{as_str}, {event_name}");
        headers.insert(
            "Hx-Trigger",
            HeaderValue::from_str(&new_header)
                .expect("event name is a valid string"),
        );
    } else {
        headers.insert(
            "Hx-Trigger",
            HeaderValue::from_str(event_name)
                .expect("event name is a valid string"),
        );
    }
    headers
}
