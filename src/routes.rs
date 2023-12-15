//! All possible routes with their params are defined in a big enum.

use super::{controllers, count_chat, models};
use axum::routing::{delete, get, post, Router};

/// This enum contains all of the route strings in the application. This
/// solves several problems.
///
/// 1. Maintaining a single source of truth for route paths, even if it has
///    multiple controllers for various HTTP methods
/// 2. Making it easier to refactor routing without needing to keep the axum
///    router and paths referenced in routers in sync.
/// 3. Making it easier to jump from a component to the handlers in a route it
///    references and visa versa.
/// 4. Further decoupling the app from the underlying HTTP.
/// 5. Allowing documentation on a route, which is super useful for quick
///    reference when authoring components.
///
/// For each route, the parameters are inside an Option<T>. If no parameters
/// are provided, we'll construct the route with the `:id` template in it
/// for the Axum router.
pub enum Route<'a> {
    HandleChat,
    ChatForm,
    SaveMeal,
    DeleteMeal(Option<i32>),
    /// The `String` slug is unnecessary here, but this is the general pattern
    /// for handling routes that have slug parameters.
    UserHome(Option<&'a str>),
    Root,
    Ping,
    Register,
    Login,
    /// The static content route where HTMX javascript library is served, which
    /// we are vendoring.
    Htmx,
}

impl Route<'_> {
    pub fn as_string(&self) -> String {
        match self {
            Self::HandleChat => "/chat".into(),
            Self::ChatForm => "/chat-form".into(),
            Self::SaveMeal => "/save-meal".into(),
            Self::DeleteMeal(slug) => match slug {
                Some(value) => format!("/delete-meal/{value}"),
                None => "/delete-meal/:id".into(),
            },
            Self::UserHome(slug) => match slug {
                Some(value) => format!("/home/{value}"),
                None => "/home/:slug".into(),
            },
            Self::Root => "/".into(),
            Self::Ping => "/ping".into(),
            Self::Register => "/authentication/register".into(),
            Self::Login => "/authentication/login".into(),
            Self::Htmx => "/static/htmx-1.9.9".into(),
        }
    }
}

impl std::fmt::Display for Route<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// In [crate::main], protected routes are registered in a router with
/// [crate::middlware::auth] middleware. This causes any requesters who are not
/// authenticated to be redirected to the login page before the request handlers
/// are called.
pub fn get_protected_routes() -> Router<models::AppState> {
    Router::new()
        .route(
            &Route::UserHome(None).as_string(),
            get(controllers::user_home),
        )
        .route(
            &Route::HandleChat.as_string(),
            post(count_chat::handle_chat),
        )
        .route(&Route::ChatForm.as_string(), get(count_chat::chat_form))
        .route(
            &Route::SaveMeal.as_string(),
            post(count_chat::handle_save_meal),
        )
        .route(
            &Route::DeleteMeal(None).as_string(),
            delete(controllers::delete_meal),
        )
}

/// In [crate::main], these routes are not protected by any authentication, so
/// any requester can access these routes.
pub fn get_public_routes() -> Router<models::AppState> {
    Router::new()
        .route(&Route::Root.as_string(), get(controllers::root))
        .route(&Route::Ping.as_string(), get(controllers::pong))
        .route(
            &Route::Register.as_string(),
            get(controllers::get_registration_form),
        )
        .route(
            &Route::Register.as_string(),
            post(controllers::handle_registration),
        )
        .route(&Route::Login.as_string(), get(controllers::get_login_form))
        .route(&Route::Login.as_string(), post(controllers::handle_login))
        .route(&Route::Htmx.as_string(), get(controllers::get_htmx_js))
}
