//! User preferences

use crate::{
    components::{Page, PageContainer, Saved},
    prelude::*,
};
use axum::http::Method;
use chrono_tz::TZ_VARIANTS;
use serde::Serialize;
use std::default::Default;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct UserPreference {
    pub timezone: Tz,
    pub caloric_intake_goal: Option<i32>,
}
impl Default for UserPreference {
    fn default() -> Self {
        Self {
            timezone: Tz::UTC,
            caloric_intake_goal: None,
        }
    }
}

impl Component for UserPreference {
    fn render(&self) -> String {
        let tz = self.timezone;
        let goal = self
            .caloric_intake_goal
            .map_or("".to_string(), |g| g.to_string());
        let options = TZ_VARIANTS.iter().fold(String::new(), |mut acc, tz_choice| {
            let selected = if *tz_choice == tz {
                "selected"
            } else {
                ""
            };
            acc.push_str(&format!(r#"<option {selected} value="{tz_choice}">{tz_choice}</option>\n"#));
            acc
        });
        let self_url = Route::UserPreference;
        let home = Route::UserHome;
        format!(
            r#"
            <div class="flex flex-col items-center justify-center max-w-prose">
                <form
                    hx-post="{self_url}"
                    class="p-4 bg-slate-200 text-black rounded w-prose flex
                    flex-col gap-2"
                >
                    <h1 class="text-2xl font-extrabold">User Preferences</h1>
                    <label for="timezone">Timezone</label>
                    <select
                        id="timezone"
                        name="timezone"
                    >{options}</select>
                    <label for="caloric_intake_goal">Caloric Intake Goal</label>
                    <p class="text-sm">
                        This should be based on your Total Daily Energy
                        Expenditure (TDEE), and your goals for weight loss,
                        maintainance, or gain. Use an online resource like
                        <a class="link" href="https://tdeecalculator.net/">the
                        TDEE calculator</a> to calculate the perfect calorie
                        goal for you.
                    </p>
                    <input
                        type="number"
                        step="100"
                        value="{goal}"
                        name="caloric_intake_goal"
                        id="caloric_intake_goal"
                    />
                    <button class="bg-blue-200 rounded">Save</button>
                    <a
                        class="text-center rounded border-slate-800 border-2"
                        href="{home}"
                    >Go back</a>
                </form>
            </div>
            "#
        )
    }
}

struct SavedPreference {
    preferences: UserPreference,
}
impl Component for SavedPreference {
    fn render(&self) -> String {
        let saved = Saved {
            message: "User preferences saved",
        }
        .render();
        let form = self.preferences.render();
        format!(
            r#"
            {saved}
            {form}
            "#
        )
    }
}

pub async fn get_user_preference(
    db: &PgPool,
    user: &User,
) -> Aresult<Option<UserPreference>> {
    struct Qres {
        timezone: String,
        caloric_intake_goal: Option<i32>,
    }
    let pref = query_as!(
        Qres,
        "select timezone, caloric_intake_goal from user_preference
        where user_id = $1",
        user.id
    )
    .fetch_optional(db)
    .await?;
    match pref {
        Some(pref) => Ok(Some(UserPreference {
            timezone: pref.timezone.parse().map_err(|_| {
                Error::msg(
                    "could not parse timezone returned from the database",
                )
            })?,
            caloric_intake_goal: pref.caloric_intake_goal,
        })),
        None => Ok(None),
    }
}

pub async fn save_user_preference(
    db: &PgPool,
    user: &User,
    preference: &UserPreference,
) -> Aresult<()> {
    query!(
        "insert into user_preference
        (user_id, timezone, caloric_intake_goal) values ($1, $2, $3)
        on conflict (user_id)
        do update set timezone = $2, caloric_intake_goal = $3",
        user.id,
        preference.timezone.to_string(),
        preference.caloric_intake_goal
    )
    .execute(db)
    .await?;

    Ok(())
}

#[derive(Deserialize)]
pub struct UserPreferencePayload {
    pub timezone: Tz,
    pub caloric_intake_goal: String,
}

pub async fn user_preference_controller(
    State(AppState { db }): State<AppState>,
    method: Method,
    headers: HeaderMap,
    preferences: Option<Form<UserPreferencePayload>>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "user preferences")?;
    let response_headers = HeaderMap::new();
    match method {
        Method::GET => {
            let preferences = get_user_preference(&db, &session.user)
                .await?
                .unwrap_or_default();

            Ok((
                response_headers,
                Page {
                    title: "User Preferences",
                    children: &PageContainer {
                        children: &preferences,
                    },
                }
                .render(),
            ))
        }
        Method::POST => {
            match preferences {
                Some(pref) => {
                    let pref = UserPreference {
                        timezone: pref.timezone,
                        caloric_intake_goal: if pref
                            .caloric_intake_goal
                            .is_empty()
                        {
                            None
                        } else {
                            let goal_int = pref.caloric_intake_goal
                                .parse()
                                .map_err(|_| {
                                    let msg = "caloric intake cannot be parsed into a number";
                                    ServerError::bad_request(
                                        msg,
                                        Some(msg.to_string())
                                    )
                                })?;
                            Some(goal_int)
                        },
                    };
                    save_user_preference(&db, &session.user, &pref).await?;
                    let new_session = Session {
                        // We will inherit the `created_at` timestamp from the
                        // current session, as to avoid implicitly re-logging-in
                        // the user, and allowing users to extend sessions by
                        // updating user preferences!
                        created_at: session.created_at,
                        user: session.user,
                        preferences: pref,
                    };
                    let response_headers =
                        new_session.update_headers(response_headers);
                    Ok((
                        response_headers,
                        SavedPreference { preferences: pref }.render(),
                    ))
                }
                None => Err(ServerError::bad_request(
                    "form data is missing",
                    Some("form data is missing".into()),
                )),
            }
        }
        _ => Err(ServerError::method_not_allowed()),
    }
}
