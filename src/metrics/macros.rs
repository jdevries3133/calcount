use crate::{
    chrono_utils::is_before_today,
    components::Component,
    count_chat::MealInfo,
    errors::ServerError,
    models::{AppState, User},
    routes::Route,
    session::Session,
};
use anyhow::Result as Aresult;
use axum::{extract::State, headers::HeaderMap, response::IntoResponse};
use chrono_tz::Tz;
use sqlx::{query_as, PgPool};

/// For now, these are implicitly an aggregation of all meals during the
/// current day, but we could imagine adding explicit time constraints to
/// this data structure.
#[derive(Default)]
pub struct Macros {
    calories: i32,
    protein_grams: i32,
    fat_grams: i32,
    carbohydrates_grams: i32,
}
impl Macros {
    pub fn is_empty(&self) -> bool {
        self.calories > 0
    }
}

impl Component for Macros {
    fn render(&self) -> String {
        let calories = self.calories;
        let protein = self.protein_grams;
        let fat = self.fat_grams;
        let carbs = self.carbohydrates_grams;
        let macros = Route::DisplayMacros;
        format!(
            r#"<p hx-get="{macros}" hx-trigger="reload-macros from:body">
                In total, you've eaten {calories} calories, {protein} grams of
                protein, {fat} grams of fat, and {carbs} carbs today.
            </p>"#
        )
    }
}

pub struct MacroPlaceholder;
impl Component for MacroPlaceholder {
    fn render(&self) -> String {
        let macros = Route::DisplayMacros;
        format!(
            r#"
            <p hx-get="{macros}" hx-trigger="reload-macros from:body">
                Enter some food to get macro information.
            </p>
            "#
        )
    }
}

pub async fn get_macros(db: &PgPool, user: &User) -> Aresult<Macros> {
    let result = query_as!(
        MealInfo,
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
        user.id
    )
    .fetch_all(db)
    .await?;

    // Now, we'll more precisely filter down to the meals that that are in the
    // user's current day, having just selected the meals from the last two
    // UTC days.
    Ok(result
        .iter()
        .filter(|m| !is_before_today(&m.created_at, Tz::US__Eastern))
        .fold(Macros::default(), |mut macros, meal| {
            macros.calories += meal.calories;
            macros.carbohydrates_grams += meal.carbohydrates_grams;
            macros.protein_grams += meal.protein_grams;
            macros.fat_grams += meal.fat_grams;
            macros
        }))
}

pub async fn display_macros(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "display macros")?;
    let macros = get_macros(&db, &session.user).await?;
    if macros.is_empty() {
        Ok(macros.render())
    } else {
        Ok(MacroPlaceholder {}.render())
    }
}
