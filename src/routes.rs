//! All possible routes with their params are defined in a big enum.

use super::{controllers, count_chat, metrics, models, preferences};
use axum::routing::{any, delete, get, post, Router};

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
pub enum Route {
    /// Route which will return an empty string. This is mainly an HTMX utility
    /// to allow a component to easily be swapped with nothing.
    ChatForm,
    DeleteMeal(Option<i32>),
    AddMealToToday(Option<i32>),
    DisplayMacros,
    HandleChat,
    Htmx,
    ListMeals,
    Login,
    Ping,
    Register,
    Root,
    SaveMeal,
    UserHome,
    UserPreference,
    Void,
}

impl Route {
    pub fn as_string(&self) -> String {
        match self {
            Self::ListMeals => "/list-meals".into(),
            Self::ChatForm => "/chat-form".into(),
            Self::DeleteMeal(slug) => match slug {
                Some(value) => format!("/delete-meal/{value}"),
                None => "/delete-meal/:id".into(),
            },
            Self::AddMealToToday(slug) => match slug {
                Some(value) => format!("/add-meal-to-today/{value}"),
                None => "/add-meal-to-today/:id".into(),
            },
            Self::DisplayMacros => "/metrics/macros".into(),
            Self::HandleChat => "/chat".into(),
            Self::Htmx => "/static/htmx-1.9.9".into(),
            Self::Login => "/authentication/login".into(),
            Self::Ping => "/ping".into(),
            Self::Register => "/authentication/register".into(),
            Self::Root => "/".into(),
            Self::SaveMeal => "/save-meal".into(),
            Self::UserHome => "/home".into(),
            Self::UserPreference => "/preferences".into(),
            Self::Void => "/void".into(),
        }
    }
}

impl std::fmt::Display for Route {
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
        .route(&Route::UserHome.as_string(), get(controllers::user_home))
        .route(
            &Route::HandleChat.as_string(),
            post(count_chat::handle_chat),
        )
        .route(&Route::ChatForm.as_string(), get(count_chat::chat_form))
        .route(&Route::ChatForm.as_string(), post(count_chat::chat_form))
        .route(
            &Route::SaveMeal.as_string(),
            post(count_chat::handle_save_meal),
        )
        .route(
            &Route::DeleteMeal(None).as_string(),
            delete(controllers::delete_meal),
        )
        .route(
            &Route::AddMealToToday(None).as_string(),
            post(controllers::add_meal_to_today),
        )
        .route(
            &Route::DisplayMacros.as_string(),
            get(metrics::display_macros),
        )
        .route(
            &Route::UserPreference.as_string(),
            any(preferences::user_preference_controller),
        )
        .route(&Route::ListMeals.as_string(), get(count_chat::list_meals))
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
        .route(&Route::Void.as_string(), get(controllers::void))
}
