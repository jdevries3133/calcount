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
        let meals = self.meals.iter().fold(String::new(), |mut acc, meal| {
            acc.push_str(&format!("<p>{meal:?}</p>"));
            acc
        });
        let current_calorie_goal = self.result.current_calorie_goal;
        let details =
            self.result
                .details
                .iter()
                .fold(String::new(), |mut acc, event| {
                    acc.push_str(&format!("<p>{event:?}</p>"));
                    acc
                });
        format!(
            r#"
            <h1 class="text-2xl font-extrabold">Calorie Balancing</h1>
            {meals}
            <div>current calorie goal: {current_calorie_goal}</div>
            <p>details: {details}</p>
            "#
        )
    }
}

pub async fn get_relevant_meals(
    db: impl PgExecutor<'_>,
    user_id: i32,
    preferences: &UserPreference,
) -> Aresult<Vec<Meal>> {
    struct Qres {
        id: i32,
        calories: i32,
        protein_grams: i32,
        carbohydrates_grams: i32,
        fat_grams: i32,
        meal_name: String,
        created_at: DateTime<Utc>,
    }
    Ok(query_as!(
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
        and user_id = $2
        order by created_at
        ",
        preferences.timezone.to_string(),
        user_id
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
    .fetch_all(db)
    .await?)
}

pub async fn get_current_goal(
    db: impl PgExecutor<'_>,
    user_id: i32,
    preferences: &UserPreference,
) -> Aresult<i32> {
    let relevant_meals = get_relevant_meals(db, user_id, preferences).await?;
    let balancing_history = compute_balancing(
        Utc::now(),
        preferences.timezone,
        preferences
            .caloric_intake_goal
            .ok_or(Error::msg("user does not have caloric intake goal"))?,
        &relevant_meals,
    );
    Ok(balancing_history.current_calorie_goal)
}

pub async fn overview(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "balancing overview")?;
    let preferences = session.get_preferences(&db).await?;
    let relevant_meals =
        get_relevant_meals(&db, session.user_id, &preferences).await?;
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
