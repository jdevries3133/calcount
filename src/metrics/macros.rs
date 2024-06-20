use crate::{
    balancing, chrono_utils::is_before_today, count_chat::MealInfo, prelude::*,
};

/// For now, these are implicitly an aggregation of all meals during the
/// current day, but we could imagine adding explicit time constraints to
/// this data structure.
pub struct Macros {
    calories: i32,
    protein_grams: i32,
    fat_grams: i32,
    carbohydrates_grams: i32,
    user_preferences: UserPreference,
}
impl Macros {
    pub fn is_empty(&self) -> bool {
        self.calories == 0
    }
    pub fn render_status(&self, caloric_intake_goal: Option<i32>) -> String {
        MacroStatus {
            macros: self,
            caloric_intake_goal,
            user_preferences: &self.user_preferences,
        }
        .render()
    }
}

pub struct MacroStatus<'a> {
    macros: &'a Macros,
    caloric_intake_goal: Option<i32>,
    user_preferences: &'a UserPreference,
}
impl Component for MacroStatus<'_> {
    fn render(&self) -> String {
        let calories = self.macros.calories;
        let protein = self.macros.protein_grams;
        let fat = self.macros.fat_grams;
        let carbs = self.macros.carbohydrates_grams;
        let macros = Route::DisplayMacros;
        let calories_remaining = match self.caloric_intake_goal {
            Some(goal) => {
                let computed_goal =
                    if self.user_preferences.calorie_balancing_enabled {
                        let history = Route::BalancingHistory;
                        format!(
                            r#"
                            <p>Your computed goal is {goal} calories.
                                <a href="{history}">
                                    <button
                                        class="
                                            bg-emerald-100 
                                            hover:bg-emerald-200 
                                            dark:bg-emerald-700 
                                            dark:hover:bg-emerald-600 
                                            rounded 
                                            m-1 
                                            p-1
                                    ">
                                        Balancing History
                                    </button>
                                </a>
                            </p>
                            "#
                        )
                    } else {
                        "".into()
                    };
                let diff = goal - calories;
                format!("<p>You have {diff} calories left to eat today.</p>{computed_goal}")
            }
            None => "".into(),
        };
        format!(
            r#"<div hx-get="{macros}" hx-trigger="reload-macros from:body">
                {calories_remaining}
                <p>
                    In total, you've eaten {calories} calories, {protein} grams
                    of protein, {fat} grams of fat, and {carbs} carbs today.
                </p>
            </div>"#
        )
    }
}

pub struct MacroPlaceholder;
impl Component for MacroPlaceholder {
    fn render(&self) -> String {
        let macros = Route::DisplayMacros;
        format!(
            r#"
            <p hx-get="{macros}" hx-trigger="reload-macros from:body"></p>
            "#
        )
    }
}

/// Macros depend on the user's timezone, because we need to aggregate meals
/// which happened "today," in the user's local timezone. At eactly midnight
/// for the user, meals should rollover into the previous day, and they should
/// get a clean slate for macros.
pub async fn get_macros(
    db: &PgPool,
    user_id: i32,
    user_preferences: &UserPreference,
) -> Aresult<Macros> {
    struct Qres {
        calories: i32,
        protein_grams: i32,
        carbohydrates_grams: i32,
        fat_grams: i32,
        meal_name: String,
        created_at: DateTime<Utc>,
    }
    let result = query_as!(
        Qres,
        "select
            name meal_name,
            calories calories,
            protein protein_grams,
            fat fat_grams,
            carbohydrates carbohydrates_grams,
            created_at
        from meal
        where
            user_id = $1
            and date_trunc('day', created_at) >= CURRENT_DATE - INTERVAL '1 day'
        ",
        user_id
    )
    .map(|row| MealInfo {
        calories: row.calories,
        protein_grams: row.protein_grams,
        fat_grams: row.fat_grams,
        carbohydrates_grams: row.carbohydrates_grams,
        meal_name: row.meal_name,
        created_at: row.created_at,
    })
    .fetch_all(db)
    .await?;

    // Now, we'll more precisely filter down to the meals that that are in the
    // user's current day, having just selected the meals from the last two
    // UTC days.
    Ok(result
        .iter()
        .filter(|m| !is_before_today(&m.created_at, user_preferences.timezone))
        .fold(
            Macros {
                calories: 0,
                protein_grams: 0,
                fat_grams: 0,
                carbohydrates_grams: 0,
                user_preferences: *user_preferences,
            },
            |mut macros, meal| {
                macros.calories += meal.calories;
                macros.carbohydrates_grams += meal.carbohydrates_grams;
                macros.protein_grams += meal.protein_grams;
                macros.fat_grams += meal.fat_grams;
                macros
            },
        ))
}

pub async fn display_macros(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "display macros")?;
    let preferences = session.get_preferences(&db).await?;
    let macros = get_macros(&db, session.user_id, &preferences).await?;
    let caloric_intake_goal = if preferences.calorie_balancing_enabled {
        Some(
            balancing::get_current_goal(&db, session.user_id, &preferences)
                .await?,
        )
    } else {
        preferences.caloric_intake_goal
    };
    if macros.is_empty() && !preferences.calorie_balancing_enabled {
        Ok(MacroPlaceholder {}.render())
    } else {
        Ok(MacroStatus {
            macros: &macros,
            caloric_intake_goal,
            user_preferences: &preferences,
        }
        .render())
    }
}
