use super::{
    auth, components, components::Component, db_ops, errors::ServerError, htmx,
    models::AppState, pw, routes::Route, session, session::Session,
};
use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
    Form,
};
use serde::Deserialize;
use std::env;

pub async fn root() -> impl IntoResponse {
    components::Page {
        title: "NC!",
        children: Box::new(components::Home {}),
    }
    .render()
}

#[cfg(feature = "live_reload")]
#[derive(Deserialize)]
pub struct PongParams {
    pub poll_interval_secs: u64,
}
/// The client will reload when this HTTP long-polling route disconnects,
/// effectively implementing live-reloading.
#[axum_macros::debug_handler]
#[cfg(feature = "live_reload")]
pub async fn pong(
    axum::extract::Query(PongParams { poll_interval_secs }): axum::extract::Query<PongParams>,
) -> impl IntoResponse {
    tokio::time::sleep(std::time::Duration::from_secs(poll_interval_secs))
        .await;
    "pong"
}

#[cfg(not(feature = "live_reload"))]
pub async fn pong() -> impl IntoResponse {
    "pong"
}

/// You may be wondering why this sits on a separate response while the
/// tailwind styles are inlined into the page template and basically
/// hard-coded into every initial response. This is because the CSS is a
/// blocker for page rendering, so we want it right there in the initial
/// response. Meanwhile, it's fine for the browser to fetch and run HTMX
/// asynchronously since it doesn't really need to be on the page until the
/// first user interaction.
///
/// Additionally, our HTMX version does not change very often. We can exploit
/// browser cachine to mostly never need to serve this resource, making the
/// app more responsive and cutting down on overall bandwidth. That's also why
/// we have the HTMX version in the URL path -- because we need to bust the
/// browser cache every time we upgrade.
pub async fn get_htmx_js() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        "content-type",
        HeaderValue::from_str("text/javascript")
            .expect("We can insert text/javascript headers"),
    );
    headers.insert(
        "cache-control",
        HeaderValue::from_str("Cache-Control: public, max-age=31536000")
            .expect("we can set cache control header"),
    );
    (headers, include_str!("./htmx-1.9.9.vendor.js"))
}

pub async fn user_home(
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let Session { user, .. } =
        Session::from_headers(&headers).expect("user is authenticated");
    let content = Box::new(components::UserHome { user: &user });
    let html = components::Page {
        title: "Home Page",
        children: content,
    }
    .render();

    Ok(html)
}

pub async fn get_registration_form(headers: HeaderMap) -> impl IntoResponse {
    let form = components::RegisterForm {};

    if headers.contains_key("Hx-Request") {
        form.render()
    } else {
        components::Page {
            title: "Register",
            children: Box::new(form),
        }
        .render()
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    username: String,
    email: String,
    password: String,
    registration_key: String,
}

pub async fn handle_registration(
    State(AppState { db }): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> Result<impl IntoResponse, ServerError> {
    let headers = HeaderMap::new();
    let registration_key = env::var("REGISTRATION_KEY").map_err(|_| {
        ServerError::internal_server_error("registration key is missing")
    })?;
    let user_key = form.registration_key;
    if user_key != registration_key {
        println!("keys {user_key} and {registration_key} (actual) don't match");
        let register_route = Route::Register;
        return Ok((
            headers,
            format!(
                r#"<p hx-trigger="load delay:1s" hx-get="{register_route}">Wrong registration key.</p>"#
            ),
        ));
    };
    let hashed_pw = pw::hash_new(&form.password);
    let user =
        db_ops::create_user(&db, form.username, form.email, &hashed_pw).await?;
    let now: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        .try_into()
        .expect("today can fit in i64");
    let session = session::Session {
        user,
        created_at: now,
    };
    let headers = session.update_headers(headers);
    let home = Route::UserHome(Some(&session.user.username)).as_string();
    let headers = htmx::redirect(headers, &home);
    Ok((headers, "OK".to_string()))
}

pub async fn get_login_form(headers: HeaderMap) -> impl IntoResponse {
    let form = components::LoginForm {};

    if headers.contains_key("Hx-Request") {
        form.render()
    } else {
        components::Page {
            title: "Login",
            children: Box::new(form),
        }
        .render()
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    /// Username or email
    identifier: String,
    password: String,
}

pub async fn handle_login(
    State(AppState { db }): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Result<impl IntoResponse, ServerError> {
    let session =
        auth::authenticate(&db, &form.identifier, &form.password).await;
    let headers = HeaderMap::new();
    if let Ok(session) = session {
        let homepage =
            Route::UserHome(Some(&session.user.username)).as_string();
        let headers = session.update_headers(headers);
        let headers = htmx::redirect(headers, &homepage);
        Ok((headers, "OK".to_string()))
    } else {
        let login_route = Route::Login;
        Ok((
            headers,
            format!(
                r#"<p hx-trigger="load delay:1s" hx-get="{login_route}">Invalid login credentials.</p>"#
            ),
        ))
    }
}
