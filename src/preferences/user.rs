//! User preferences

use crate::{components::Saved, prelude::*};
use axum::http::Method;
use chrono_tz::TZ_VARIANTS;
use serde::Serialize;

#[derive(Copy, Clone, Serialize, Debug)]
pub struct UserPreference {
    pub timezone: Tz,
    pub caloric_intake_goal: Option<i32>,
    pub calorie_balancing_enabled: bool,
    /// This is optional because it's possible for calorie counts to just keep
    /// rising forever.
    pub calorie_balancing_max_calories: Option<i32>,
    /// Though this will implicitly be zero if not set (since we don't want to
    /// show negative calorie goals), we'll still model it as an optional
    /// property so that we know whether to render an explicit zero, or a
    /// blank form field on the preferences page.
    pub calorie_balancing_min_calories: Option<i32>,
}

impl Default for UserPreference {
    fn default() -> Self {
        Self {
            timezone: Tz::UTC,
            caloric_intake_goal: None,
            calorie_balancing_enabled: false,
            calorie_balancing_max_calories: None,
            calorie_balancing_min_calories: None,
        }
    }
}

struct UserPreferenceForm<'a> {
    preferences: UserPreference,
    field_validation_error: Option<&'a [(&'a str, &'a str)]>,
}

impl<'a> UserPreferenceForm<'a> {
    fn get_field_validation_err<'b>(&self, field: &'b str) -> Option<&'a str> {
        self.field_validation_error.map(|some| {
            some.iter()
                .find(|(f, _msg)| *f == field)
                .map(|(_f, msg)| *msg)
        })?
    }
}

impl Component for UserPreferenceForm<'_> {
    fn render(&self) -> String {
        let tz = self.preferences.timezone;
        let goal = self
            .preferences
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
        let checkbox = if self.preferences.calorie_balancing_enabled {
            r#"
            <input
                class="w-6 h-6"
                type="checkbox"
                id="calorie_balancing_enabled"
                name="calorie_balancing_enabled"
                checked
            />
            "#
        } else {
            r#"
            <input
                class="w-6 h-6"
                type="checkbox"
                id="calorie_balancing_enabled"
                name="calorie_balancing_enabled"
            />
            "#
        };
        let min_calories = self
            .preferences
            .calorie_balancing_min_calories
            .map(|v| v.to_string())
            .unwrap_or("".into());
        let max_calories = self
            .preferences
            .calorie_balancing_max_calories
            .map(|v| v.to_string())
            .unwrap_or("".into());
        let script = include_str!("./interactive_checkbox.js");
        let intake_goal_err = if let Some(err) =
            self.get_field_validation_err("caloric_intake_goal")
        {
            format!(r#"<p class="text-red-500 italic text-sm">{err}</p>"#)
        } else {
            "".into()
        };
        let min_cals_err = if let Some(err) =
            self.get_field_validation_err("calorie_balancing_min_calories")
        {
            format!(r#"<p class="text-red-500 italic text-sm">{err}</p>"#)
        } else {
            "".into()
        };
        let max_cals_err = if let Some(err) =
            self.get_field_validation_err("calorie_balancing_max_calories")
        {
            format!(r#"<p class="text-red-500 italic text-sm">{err}</p>"#)
        } else {
            "".into()
        };
        format!(
            r#"
            <div class="flex flex-col items-center justify-center max-w-prose">
                <form
                    hx-post="{self_url}"
                    class="p-4 bg-slate-200 dark:bg-indigo-900 dark:text-slate-200
                    text-black rounded w-prose flex
                    flex-col gap-2"
                >
                    <h1 class="text-2xl font-extrabold">User Preferences</h1>
                    <label for="timezone">Timezone</label>
                    <select
                        id="timezone"
                        name="timezone"
                    >{options}</select>
                    <div class="rounded my-3 p-3 border-2 border-black">
                        <label for="caloric_intake_goal">Caloric Intake Goal</label>
                        <p class="text-xs">
                            (Optional setting) use an online resource like
                            <a class="link" href="https://tdeecalculator.net/">the
                            TDEE calculator</a> to calculate the perfect calorie
                            goal for you.
                        </p>
                        {intake_goal_err}
                        <input
                            type="number"
                            value="{goal}"
                            name="caloric_intake_goal"
                            id="caloric_intake_goal"
                        />
                    </div>
                    <div class="rounded my-3 p-3 border-2 border-black">
                        <label for="calorie_balancing_enabled">
                            Enable calorie balancing
                        </label>
                        <details>
                            <summary class="text-xs">Learn more</summary>
                            <p class="text-xs">
                                If enabled, excess or defecit calories from previous
                                days will be applied to future days. Combined with
                                accurate calorie counting, this can help you ensure
                                that if you eat too many or too few calories on one
                                day, you ultimately "catch-up," and continuously
                                work towards your calorie goal.
                            </p>
                            <div
                                class="bg-yellow-100 dark:text-black rounded p-1 prose
                                text-black mt-1"
                            >
                                <p class="text-xs">
                                    Warning: if you have a history of <a class="link"
                                    href="https://www.mayoclinic.org/diseases-conditions/eating-disorders/symptoms-causes/syc-20353603"
                                    >any eating disorder,</a> please consult a dietician
                                    before using this product in general, but especially
                                    related to the use of this feature, which includes a
                                    risk of worsening existing ED conditions.
                                </p>
                                <p class="text-xs">
                                    To avoid receiving unhealthy calorie goals after
                                    unhealthy eating episodes, consider setting a
                                    minimum and maximum calorie limit which is close
                                    to your overall calorie goal, so that you never
                                    receive unhealthy calorie goals.
                                </p>
                                <p class="text-xs">
                                    I am particularly concerned about building this
                                    site to support healthy eating. Please do not
                                    hesitate to reach out and share any feedback
                                    on this application with me
                                    (<a href="mailto:jdevries3133@gmail.com">jdevries3133@gmail.com</a>).
                                </p>
                            </div>
                        </details>
                        {checkbox}
                    </div>
                    <div class="rounded my-3 p-3 border-2 border-black">
                        <h2 class="text-lg">Calorie Limits</h2>
                        <details class="text-xs">
                            <summary>Learn more</summary>
                            <p>
                                Minimum and maximum limits apply to the calorie
                                goals that we set for you via calorie balancing.
                                We will never set a goal for you which is outside
                                your limits, and instead, we'll apply any excess
                                calories to a later date, allowing you to more
                                gently work back towards your calorie counting goal.
                            </p>
                        </details>
                        <label class="block" for="calorie_balancing_min_calories">
                            Minimum daily calorie limit
                        </label>
                        {min_cals_err}
                        <input
                            type="number"
                            id="calorie_balancing_min_calories"
                            name="calorie_balancing_min_calories"
                            value="{min_calories}"
                        />
                        <label class="block" for="calorie_balancing_max_calories">
                            Maximum daily calorie limit
                        </label>
                        {max_cals_err}
                        <input
                            type="number"
                            id="calorie_balancing_max_calories"
                            name="calorie_balancing_max_calories"
                            value="{max_calories}"
                        />
                    </div>
                    <button class="
                        bg-green-100 
                        hover:bg-green-200 
                        dark:bg-green-700 
                        dark:hover:bg-green-600 
                        rounded 
                        p-1
                    ">
                        Save
                    </button>
                    <a
                        class="text-center rounded border-slate-800 border-2"
                        href="{home}"
                    >Go back</a>
                </form>
            </div>
            <script>
                (() => {{
                    {script}
                }})();
            </script>
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
        let form = UserPreferenceForm {
            preferences: self.preferences,
            field_validation_error: None,
        }
        .render();
        format!(
            r#"
            {form}
            {saved}
            "#
        )
    }
}

pub async fn get_user_preference(
    db: &PgPool,
    user_id: i32,
) -> Aresult<Option<UserPreference>> {
    struct Qres {
        timezone: String,
        caloric_intake_goal: Option<i32>,
        calorie_balancing_enabled: bool,
        calorie_balancing_max_calories: Option<i32>,
        calorie_balancing_min_calories: Option<i32>,
    }
    let pref = query_as!(
        Qres,
        "select
            caloric_intake_goal,
            calorie_balancing_enabled,
            calorie_balancing_max_calories,
            calorie_balancing_min_calories,
            timezone
        from user_preference
        where user_id = $1",
        user_id
    )
    .fetch_optional(db)
    .await?;
    match pref {
        Some(pref) => Ok(Some(UserPreference {
            calorie_balancing_enabled: pref.calorie_balancing_enabled,
            calorie_balancing_max_calories: pref.calorie_balancing_max_calories,
            calorie_balancing_min_calories: pref.calorie_balancing_min_calories,
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
    user_id: i32,
    preference: &UserPreference,
) -> Aresult<()> {
    query!(
        "insert into user_preference
        (
            user_id,
            timezone,
            caloric_intake_goal,
            calorie_balancing_enabled,
            calorie_balancing_min_calories,
            calorie_balancing_max_calories
        ) values ($1, $2, $3, $4, $5, $6)
        on conflict (user_id)
        do update set
            timezone = $2,
            caloric_intake_goal = $3,
            calorie_balancing_enabled = $4,
            calorie_balancing_min_calories = $5,
            calorie_balancing_max_calories = $6
        ",
        user_id,
        preference.timezone.to_string(),
        preference.caloric_intake_goal,
        preference.calorie_balancing_enabled,
        preference.calorie_balancing_min_calories,
        preference.calorie_balancing_max_calories
    )
    .execute(db)
    .await?;

    Ok(())
}

enum CalorieBalancingLimitResult {
    /// The value is present
    Some(i32),
    /// The value is missing, but the previously saved value should be
    /// retained. This happens when the field is disabled.
    None,
    /// The property was sent from the frontend, but it was empty. This means
    /// that the field was not disabled, but the input was empty. In this
    /// case, we need to set the user preference for this value to NULL.
    Unset,
    /// There was an error parsing the value sent from the frontend
    Err(&'static str),
}

impl CalorieBalancingLimitResult {
    /// With knowledge of the existing value, we can convert this into a
    /// data-type easier to propagate.
    ///
    /// - If we have Some, we overwrite the existing value.
    /// - If we have None, we use the existing value
    /// - If we have Unset, we set the value to None,
    /// - If we have Err, we propagate it via the Result type
    fn combobulate(
        &self,
        existing_value: Option<i32>,
    ) -> Result<Option<i32>, &'static str> {
        match self {
            Self::Some(v) => Ok(Some(*v)),
            Self::None => Ok(existing_value),
            Self::Unset => Ok(None),
            Self::Err(e) => Err(e),
        }
    }
}

#[derive(Deserialize)]
pub struct UserPreferencePayload {
    timezone: Tz,
    caloric_intake_goal: String,
    /// Checkboxes will be the string, "on" if set, or the filed will be
    /// omitted if unset.
    calorie_balancing_enabled: Option<String>,
    calorie_balancing_min_calories: Option<String>,
    calorie_balancing_max_calories: Option<String>,
}
impl UserPreferencePayload {
    fn get_intake_goal(&self) -> Result<Option<i32>, &'static str> {
        if self.caloric_intake_goal.is_empty() {
            if self.calorie_balancing_enabled.is_some() {
                Err(
                    r#"A caloric intake goal is required if calorie balancing is enabled. If you want to remove this goal, also uncheck "enable calorie balancing.""#,
                )
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(self.caloric_intake_goal.parse().map_err(|_| {
                "Caloric intake cannot be parsed into a number."
            })?))
        }
    }
    fn get_calorie_balancing_enabled(&self) -> bool {
        self.calorie_balancing_enabled.is_some()
    }
    fn get_calorie_balancing_min_calories(
        &self,
    ) -> CalorieBalancingLimitResult {
        match self.calorie_balancing_min_calories {
            None => CalorieBalancingLimitResult::None,
            Some(ref str) => {
                if str.is_empty() {
                    CalorieBalancingLimitResult::Unset
                } else {
                    match str.parse() {
                        Ok(val) => {
                            match self.get_intake_goal() {
                                Ok(Some(goal)) => {
                                    if val > goal {
                                        CalorieBalancingLimitResult::Err("Minimum limit cannot be greater than your overall goal.")
                                    } else {
                                        CalorieBalancingLimitResult::Some(val)
                                    }
                                },
                                _ => CalorieBalancingLimitResult::Err("Set a caloric intake goal if you would like to use calorie balancing.")
                            }
                        }
                        Err(_) => CalorieBalancingLimitResult::Err(
                            "Minimum calories cannot be parsed into a number.",
                        ),
                    }
                }
            }
        }
    }
    fn get_calorie_balancing_max_calories(
        &self,
    ) -> CalorieBalancingLimitResult {
        match self.calorie_balancing_max_calories {
            None => CalorieBalancingLimitResult::None,
            Some(ref str) => {
                if str.is_empty() {
                    CalorieBalancingLimitResult::Unset
                } else {
                    match str.parse() {
                        Ok(val) => {
                            match self.get_intake_goal() {
                                Ok(Some(goal)) => {
                                    if val < goal {
                                        CalorieBalancingLimitResult::Err("Maximum limit cannot be less than your overall goal.")
                                    } else {
                                        CalorieBalancingLimitResult::Some(val)
                                    }
                                },
                                _ => CalorieBalancingLimitResult::Err("Set a caloric intake goal if you would like to use calorie balancing.")
                            }
                        }
                        Err(_) => CalorieBalancingLimitResult::Err(
                            "Maximum calories cannot be parsed into a number.",
                        ),
                    }
                }
            }
        }
    }
    fn get_errors(&self) -> Vec<(&'static str, &'static str)> {
        let mut errs = Vec::new();
        if let Err(e) = self.get_intake_goal() {
            errs.push(("caloric_intake_goal", e))
        };
        if let CalorieBalancingLimitResult::Err(e) =
            self.get_calorie_balancing_min_calories()
        {
            errs.push(("calorie_balancing_min_calories", e))
        };
        if let CalorieBalancingLimitResult::Err(e) =
            self.get_calorie_balancing_max_calories()
        {
            errs.push(("calorie_balancing_max_calories", e))
        };
        errs
    }
    /// Produce the best approximation of UserPreference from the unvalidated
    /// data. This is used for re-rendering previous form inputs when
    /// displaying validation errors.
    fn get_unvalidated_data(
        &self,
        existing_preferences: &UserPreference,
    ) -> UserPreference {
        UserPreference {
            timezone: existing_preferences.timezone,
            caloric_intake_goal: self
                .caloric_intake_goal
                .parse()
                .ok()
                .or(existing_preferences.caloric_intake_goal),
            calorie_balancing_enabled: self.calorie_balancing_enabled.is_some(),
            calorie_balancing_max_calories: self
                .calorie_balancing_max_calories
                .as_ref()
                .and_then(|v| v.parse().ok())
                .or(existing_preferences.calorie_balancing_max_calories),
            calorie_balancing_min_calories: self
                .calorie_balancing_min_calories
                .as_ref()
                .and_then(|v| v.parse().ok())
                .or(existing_preferences.calorie_balancing_min_calories),
        }
    }
}

pub async fn user_preference_controller(
    State(AppState { db }): State<AppState>,
    method: Method,
    headers: HeaderMap,
    preferences: Option<Form<UserPreferencePayload>>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "user preferences")?;
    let existing_preferences = session.get_preferences(&db).await?;
    match method {
        Method::GET => {
            let preferences = session.get_preferences(&db).await?;
            Ok(Page {
                title: "User Preferences",
                children: &PageContainer {
                    children: &UserPreferenceForm {
                        preferences,
                        field_validation_error: None,
                    },
                },
            }
            .render())
        }
        Method::POST => match preferences {
            Some(pref) => {
                let intake = pref.get_intake_goal();
                let min_cals = pref.get_calorie_balancing_min_calories();
                let max_cals = pref.get_calorie_balancing_max_calories();
                let min_cals = min_cals.combobulate(
                    existing_preferences.calorie_balancing_min_calories,
                );
                let max_cals = max_cals.combobulate(
                    existing_preferences.calorie_balancing_max_calories,
                );
                match (intake, min_cals, max_cals) {
                    (Ok(intake), Ok(min), Ok(max)) => {
                        let pref = UserPreference {
                            timezone: pref.timezone,
                            caloric_intake_goal: intake,
                            calorie_balancing_enabled: pref
                                .get_calorie_balancing_enabled(),
                            calorie_balancing_max_calories: max,
                            calorie_balancing_min_calories: min,
                        };
                        save_user_preference(&db, session.user_id, &pref)
                            .await?;
                        Ok(SavedPreference { preferences: pref }.render())
                    }
                    _ => Ok(UserPreferenceForm {
                        preferences: pref
                            .get_unvalidated_data(&existing_preferences),
                        field_validation_error: Some(&pref.get_errors()),
                    }
                    .render()),
                }
            }
            None => Err(ServerError::bad_request(
                "form data is missing",
                Some("form data is missing".into()),
            )),
        },
        _ => Err(ServerError::method_not_allowed()),
    }
}
