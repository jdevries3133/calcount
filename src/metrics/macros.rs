use crate::{
    components::Component,
    errors::ServerError,
    models::{AppState, User},
    routes::Route,
    session::Session,
};
use anyhow::Result as Aresult;
use axum::{extract::State, headers::HeaderMap, response::IntoResponse};
use sqlx::{query_as, PgPool};

/// For now, these are implicitly an aggregation of all meals during the
/// current day, but we could imagine adding explicit time constraints to
/// this data structure.
pub struct Macros {
    calories: i64,
    protein_grams: i64,
    fat_grams: i64,
    carbohydrates_grams: i64,
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

pub async fn get_macros(db: &PgPool, user: &User) -> Aresult<Option<Macros>> {
    struct Qres {
        calories: Option<i64>,
        protein_grams: Option<i64>,
        fat_grams: Option<i64>,
        carbohydrates_grams: Option<i64>,
    }
    let result = query_as!(
        Qres,
        "select
            sum(calories) calories,
            sum(protein) protein_grams,
            sum(fat) fat_grams,
            sum(carbohydrates) carbohydrates_grams
        from meal
        where
            user_id = $1
            and date_trunc('day', created_at) = CURRENT_DATE
        ",
        user.id
    )
    .fetch_one(db)
    .await?;

    match result {
        Qres {
            calories: Some(calories),
            protein_grams: Some(protein_grams),
            fat_grams: Some(fat_grams),
            carbohydrates_grams: Some(carbohydrates_grams),
        } => Ok(Some(Macros {
            calories,
            protein_grams,
            fat_grams,
            carbohydrates_grams,
        })),
        _ => Ok(None),
    }
}

pub async fn display_macros(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "display macros")?;
    let macros = get_macros(&db, &session.user).await?;
    if let Some(macros) = macros {
        Ok(macros.render())
    } else {
        Ok(MacroPlaceholder {}.render())
    }
}
