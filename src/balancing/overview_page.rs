use super::compute_balancing::{compute_balancing, BalancedCaloriesResult};
use crate::{
    count_chat::{Meal, MealInfo},
    prelude::*,
};

struct BalancingOverview<'a> {
    meals: &'a [Meal],
    result: &'a BalancedCaloriesResult<'a>,
}
impl Component for BalancingOverview<'_> {
    fn render(&self) -> String {
        let meals = self.meals;
        let result = self.result;
        format!(
            r#"
            <h1 class="text-2xl font-extrabold">Calorie Balancing</h1>
            {meals:?}
            {result:?}
            "#
        )
    }
}

pub async fn overview(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "balancing overview")?;
    let preferences = session.get_preferences(&db).await?;
    struct Qres {
        id: i32,
        calories: i32,
        protein_grams: i32,
        carbohydrates_grams: i32,
        fat_grams: i32,
        meal_name: String,
        created_at: DateTime<Utc>,
    }
    let relevant_meals = query_as!(
        Qres,
        "select
            id,
            calories,
            protein protein_grams,
            carbohydrates carbohydrates_grams,
            fat fat_grams,
            name meal_name,
            created_at
        from meal
        where created_at > (
            case when exists (
                select 1 from balancing_checkpoint where user_id = $2
            )
            then (
                select ignore_before at time zone $1
                from balancing_checkpoint
                where user_id = $2
                order by ignore_before desc
                limit 1
            ) else date('01-01-0')
            end
        )
        and user_id = $2",
        preferences.timezone.to_string(),
        session.user_id
    )
    .map(|row| Meal {
        id: row.id,
        info: MealInfo {
            calories: row.calories,
            carbohydrates_grams: row.carbohydrates_grams,
            created_at: row.created_at,
            fat_grams: row.fat_grams,
            meal_name: row.meal_name,
            protein_grams: row.protein_grams,
        },
    })
    .fetch_all(&db)
    .await?;

    let balancing_history = compute_balancing(
        Utc::now(),
        preferences.timezone,
        preferences
            .caloric_intake_goal
            .expect("user has caloric intake goal"),
        &relevant_meals,
    );

    Ok(Page {
        title: "Calorie Balancing",
        children: &PageContainer {
            children: &BalancingOverview {
                meals: &relevant_meals,
                result: &balancing_history,
            },
        },
    }
    .render())
}
