use super::{
    auth, chrono_utils, client_events, components, components::Component,
    count_chat, db_ops, errors::ServerError, htmx, metrics, models::AppState,
    preferences::UserPreference, pw, routes::Route, session, session::Session,
    stripe,
};
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue},
    response::{IntoResponse, Redirect},
    Form,
};
use chrono::{DateTime, Utc};
use futures::join;
use serde::Deserialize;
use sqlx::{query, query_as};

pub async fn root() -> impl IntoResponse {
    components::Page {
        title: "calcount",
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
    (headers, include_str!("./htmx-1.9.10.vendor.js"))
}

pub async fn user_home(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let Session {
        user, preferences, ..
    } = Session::from_headers_err(&headers, "user home")?;
    let (macros, meals) = join![
        metrics::get_macros(&db, &user, &preferences),
        count_chat::list_meals_op(&db, user.id, 0),
    ];
    let macros = macros?;
    let meals = meals?;
    let content = Box::new(components::UserHome {
        user: &user,
        meals: &meals,
        macros: &macros,
        preferences,
    });
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
    let registration_keys = db_ops::get_registraton_keys(&db).await?;
    let user_key = form.registration_key;
    if !registration_keys.contains(&user_key) {
        println!("{user_key} is not a known registration key");
        let register_route = Route::Register;
        return Ok((
            headers,
            format!(
                r#"<p hx-trigger="load delay:1s" hx-get="{register_route}">Wrong registration key.</p>"#
            ),
        ));
    };
    let hashed_pw = pw::hash_new(&form.password);

    let stripe_id =
        stripe::create_customer(&form.username, &form.email).await?;
    let payment_portal_url =
        stripe::create_billing_portal_session(&stripe_id).await?;

    let user = db_ops::create_user(
        &db,
        form.username,
        form.email,
        &hashed_pw,
        stripe_id,
        stripe::SubscriptionTypes::Initializing,
    )
    .await?;
    let now: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        .try_into()
        .expect("today can fit in i64");
    let session = session::Session {
        user,
        preferences: UserPreference::default(),
        created_at: now,
    };
    let headers = session.update_headers(headers);
    let headers = htmx::redirect(headers, &payment_portal_url);
    Ok((headers, "OK".to_string()))
}

pub async fn get_login_form(headers: HeaderMap) -> impl IntoResponse {
    let form = components::LoginForm {};
    let session = Session::from_headers(&headers);

    let response_headers = HeaderMap::new();
    if headers.contains_key("Hx-Request") {
        if session.is_some() {
            let response_headers =
                htmx::redirect(response_headers, &Route::UserHome.as_string());
            (response_headers, "OK").into_response()
        } else {
            (response_headers, form.render()).into_response()
        }
    } else if session.is_some() {
        (
            response_headers,
            Redirect::temporary(&Route::UserHome.as_string()),
        )
            .into_response()
    } else {
        (
            response_headers,
            components::Page {
                title: "Login",
                children: Box::new(form),
            }
            .render(),
        )
            .into_response()
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
        let homepage = Route::UserHome.as_string();
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

pub async fn delete_meal(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers(&headers)
        .ok_or_else(|| ServerError::forbidden("delete meal"))?;
    struct Qres {
        created_at: DateTime<Utc>,
    }
    let Qres { created_at } = query_as!(
        Qres,
        "select created_at from meal where user_id = $1 and id = $2",
        session.user.id,
        id
    )
    .fetch_one(&db)
    .await?;
    query!(
        "delete from meal where user_id = $1 and id = $2",
        session.user.id,
        id
    )
    .execute(&db)
    .await?;
    if !chrono_utils::is_before_today(&created_at, session.preferences.timezone)
    {
        Ok(
            (client_events::reload_macros(HeaderMap::new()), "")
                .into_response(),
        )
    } else {
        Ok("".into_response())
    }
}

pub async fn add_meal_to_today(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "add meal to today")?;
    let existing_meal = query_as!(
        count_chat::MealInfo,
        "select
            calories,
            protein protein_grams,
            carbohydrates carbohydrates_grams,
            fat fat_grams,
            name meal_name,
            created_at
        from meal
        where id = $1 and user_id = $2",
        id,
        session.user.id
    )
    .fetch_one(&db)
    .await?;
    query!(
        "insert into meal (calories, protein, carbohydrates, fat, name, user_id)
        values ($1, $2, $3, $4, $5, $6)",
        existing_meal.calories,
        existing_meal.protein_grams,
        existing_meal.carbohydrates_grams,
        existing_meal.fat_grams,
        existing_meal.meal_name,
        session.user.id
    )
    .execute(&db)
    .await?;

    let headers = client_events::reload_meals(HeaderMap::new());
    let headers = client_events::reload_macros(headers);
    Ok((headers, ""))
}

pub async fn void() -> &'static str {
    ""
}
