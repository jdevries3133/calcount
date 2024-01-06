//! HTMX utils

use axum::http::{HeaderMap, HeaderValue};
#[cfg(feature = "stripe")]
use axum::response::{IntoResponse, Redirect};

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

/// Like the above, but better
#[cfg(feature = "stripe")]
pub fn redirect_2(headers: HeaderMap, to: &str) -> impl IntoResponse {
    let headers = redirect(headers, to);
    let response = Redirect::to(to);
    (headers, response)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_trigger_event() {
        let headers = trigger_event(HeaderMap::new(), "test-event");
        let header_val = headers
            .get("Hx-Trigger")
            .expect("we have a trigger header")
            .to_str()
            .expect("trigger header can be stringified");
        assert_eq!(header_val, "test-event")
    }

    #[test]
    fn test_trigger_event_with_multiple_events() {
        let headers = trigger_event(HeaderMap::new(), "test-event");
        let headers = trigger_event(headers, "second-test-event");
        let header_val = headers
            .get("Hx-Trigger")
            .expect("we have a trigger header")
            .to_str()
            .expect("trigger header can be stringified");
        assert_eq!(header_val, "test-event, second-test-event")
    }
}
