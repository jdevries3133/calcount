use crate::htmx::trigger_event;
use axum::http::HeaderMap;

pub fn reload_macros(headers: HeaderMap) -> HeaderMap {
    trigger_event(headers, "reload-macros")
}

pub fn reload_food(headers: HeaderMap) -> HeaderMap {
    trigger_event(headers, "reload-food")
}
