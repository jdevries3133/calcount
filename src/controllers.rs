use super::{
    auth::Session, balancing, chrono_utils, client_events, components,
    components::Component, count_chat, errors::ServerError, htmx, metrics,
    models::AppState, stripe,
};
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use futures::join;
#[cfg(feature = "live_reload")]
use serde::Deserialize;
use sqlx::{query, query_as};

pub async fn root() -> impl IntoResponse {
    components::Page {
        title: "Bean Count",
        children: &components::Home {},
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
        "Content-Type",
        HeaderValue::from_str("text/javascript")
            .expect("We can insert text/javascript headers"),
    );
    headers.insert(
        "Cache-Control",
        HeaderValue::from_str("public, max-age=31536000")
            .expect("we can set cache control header"),
    );

    (headers, htmx::get_client_script())
}

pub async fn get_favicon() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_str("image/x-icon")
            .expect("We can insert image/x-icon header"),
    );
    (headers, include_bytes!("./static/favicon.ico"))
}

fn png_controller(bytes: &'static [u8]) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_str("image/png")
            .expect("We can insert image/png header"),
    );
    (headers, bytes)
}

pub async fn get_tiny_icon() -> impl IntoResponse {
    png_controller(include_bytes!("./static/icon-16x16.png"))
}

pub async fn get_small_icon() -> impl IntoResponse {
    png_controller(include_bytes!("./static/icon-32x32.png"))
}

pub async fn get_medium_icon() -> impl IntoResponse {
    png_controller(include_bytes!("./static/icon-192x192.png"))
}

pub async fn get_large_icon() -> impl IntoResponse {
    png_controller(include_bytes!("./static/icon-512x512.png"))
}

pub async fn get_maskable_small_icon() -> impl IntoResponse {
    png_controller(include_bytes!("./static/maskable_icon_x72.png"))
}

pub async fn get_maskable_medium_icon() -> impl IntoResponse {
    png_controller(include_bytes!("./static/maskable_icon_x128.png"))
}

pub async fn get_maskable_large_icon() -> impl IntoResponse {
    png_controller(include_bytes!("./static/maskable_icon_x192.png"))
}

pub async fn get_apple_icon() -> impl IntoResponse {
    png_controller(include_bytes!("./static/apple-touch-icon.png"))
}

pub async fn get_manifest() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_str("application/manifest+json")
            .expect("We can insert application/manifest+json header"),
    );
    (headers, include_str!("./static/manifest.json"))
}

pub async fn get_robots_txt() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_str("text/plain")
            .expect("We can insert text/plain header"),
    );
    (headers, "# beep boop\nUser-agent: *\nAllow: /")
}

pub async fn user_home(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "user home")?;
    let (user, preferences) =
        join![session.get_user(&db), session.get_preferences(&db)];
    let user = user?;
    let preferences = preferences?;
    let (macros, meals, sub_type, caloric_intake_goal) = join![
        metrics::get_macros(&db, session.user_id, &preferences),
        count_chat::list_meals_op(&db, user.id, &preferences, 0),
        stripe::get_subscription_type(&db, user.id),
        balancing::get_current_goal(&db, user.id, &preferences)
    ];
    let macros = macros?;
    let meals = meals?;
    let sub_type = sub_type?;
    let caloric_intake_goal = caloric_intake_goal?;
    let html = components::Page {
        title: "Home Page",
        children: &components::PageContainer {
            children: &components::UserHome {
                user: &user,
                food_items: &meals,
                macros: &macros,
                preferences,
                subscription_type: sub_type,
                caloric_intake_goal,
            },
        },
    }
    .render();

    Ok(html)
}

pub async fn delete_food(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers(&headers)
        .ok_or_else(|| ServerError::forbidden("delete meal"))?;
    let preferences = session.get_preferences(&db).await?;

    struct Qres {
        eaten_at: DateTime<Utc>,
    }

    let Qres { eaten_at } = query_as!(
        Qres,
        "select eaten_at from food_eaten_event
        where id = $1 and user_id = $2",
        id,
        session.user_id
    )
    .fetch_one(&db)
    .await?;

    query!(
        "delete from food_eaten_event
        where
            id = $1
            and user_id = $2",
        id,
        session.user_id,
    )
    .execute(&db)
    .await?;
    if !chrono_utils::is_before_today(&eaten_at, preferences.timezone) {
        Ok(
            (client_events::reload_macros(HeaderMap::new()), "")
                .into_response(),
        )
    } else {
        Ok("".into_response())
    }
}

pub async fn add_food_to_today(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "add meal to today")?;
    let existing_meal = query!(
        "select f.id
        from food_eaten_event fee
        join food f on fee.food_id = f.id
        where
            f.id = $1
            and f.user_id = $2
            and fee.user_id = $2",
        id,
        session.user_id
    )
    .fetch_optional(&db)
    .await?;
    if existing_meal.is_none() {
        return Err(ServerError::bad_request(
            "attempt to add non-existent meal to toay",
            None,
        ));
    }
    query!(
        "insert into food_eaten_event (user_id, food_id)
        values ($1, $2)",
        session.user_id,
        id
    )
    .execute(&db)
    .await?;

    let headers = client_events::reload_food(HeaderMap::new());
    let headers = client_events::reload_macros(headers);
    Ok((headers, ""))
}

pub async fn void() -> &'static str {
    ""
}

pub async fn about() -> impl IntoResponse {
    components::Page {
        title: "About Bean Count",
        children: &components::PageContainer {
            children: &components::AboutPage {},
        },
    }
    .render()
}
