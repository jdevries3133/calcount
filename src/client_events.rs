use crate::htmx::trigger_event;
use axum::headers::HeaderMap;

pub fn reload_macros(headers: HeaderMap) -> HeaderMap {
    trigger_event(headers, "reload-macros")
}

pub fn reload_meals(headers: HeaderMap) -> HeaderMap {
    trigger_event(headers, "reload-meals")
}
