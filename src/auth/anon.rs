use super::{pw::hash_new, register::create_user};
use crate::{config, htmx, preferences::save_user_preference, prelude::*};
use chrono::Days;
use regex::Regex;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AnonForm {
    timezone: Tz,
}

pub async fn init_anon(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    Form(AnonForm { timezone }): Form<AnonForm>,
) -> Result<impl IntoResponse, ServerError> {
    let session = match Session::from_headers(&headers) {
        Some(ses) => ses,
        None => {
            let uuid = Uuid::new_v4().to_string();
            let username = format!("anon-{uuid}");
            let email = format!("anon-{uuid}@example.com");
            let password = hash_new(&Uuid::new_v4().to_string());
            let user = create_user(
                &db,
                username,
                email,
                &password,
                "".to_string(),
                SubscriptionTypes::FreeTrial(config::FREE_TRIAL_DURATION),
            )
            .await?;
            let preferences = UserPreference {
                timezone,
                ..Default::default()
            };
            save_user_preference(&db, user.id, &preferences).await?;
            Session {
                user_id: user.id,
                username: user.username,
                // I'm just going to fake a distant future created date, which
                // creates a long-lived token. This user won't know their
                // password until they convert into a non-anon registered
                // user, so we don't want to surprise log them out.
                created_at: utc_now().checked_add_days(Days::new(365)).expect(
                    "can add 1 year to the current date w/o overflowing",
                ),
            }
        }
    };
    let response_headers = HeaderMap::new();
    let headers = session.update_headers(response_headers);

    Ok(htmx::redirect_2(headers, &Route::UserHome.as_string()))
}

pub fn is_anon(username: &str) -> bool {
    let pattern = r"^anon-[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$";
    let regex = Regex::new(pattern).unwrap();
    regex.is_match(username)
}
