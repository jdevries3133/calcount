use crate::{
    components::Component,
    errors::ServerError,
    models::{AppState, User},
    routes::Route,
    session::Session,
};
use axum::{
    extract::State, headers::HeaderMap, http::StatusCode,
    response::IntoResponse,
};
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

pub async fn get_macros(
    db: &PgPool,
    user: &User,
) -> Result<Macros, ServerError> {
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
        where user_id = $1",
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
        } => Ok(Macros {
            calories,
            protein_grams,
            fat_grams,
            carbohydrates_grams,
        }),
        _ => Err(ServerError::custom_expected_error(
            StatusCode::NOT_FOUND,
            "".into(),
        )),
    }
}

pub async fn display_macros(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "display macros")?;
    let macros = get_macros(&db, &session.user).await?;
    Ok(macros.render())
}
