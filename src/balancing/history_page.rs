use super::compute_balancing::{compute_balancing, BalancedCaloriesResult};
use crate::{
    count_chat::{FoodItem, FoodItemDetails},
    prelude::*,
};

struct BalancingHistory<'a> {
    result: &'a BalancedCaloriesResult<'a>,
}
impl Component for BalancingHistory<'_> {
    fn render(&self) -> String {
        let current_calorie_goal = self.result.current_calorie_goal;
        let checkpoint = Route::BalancingCheckpoints;
        let details =
            self.result
                .details
                .iter()
                .fold(String::new(), |mut acc, event| {
                    acc.push_str(&event.render());
                    acc
                });
        format!(
            r#"
            <button
                class="dark:bg-emerald-700 dark:hover:bg-emerald-800
                bg-emerald-100 hover:bg-emerald-200 p-1 m-1 rounded"
                onclick="history.back()"
            >
                Back
            </button>
            <a href="{checkpoint}">
                <button
                    class="dark:bg-emerald-700 dark:hover:bg-emerald-800
                    bg-emerald-100 hover:bg-emerald-200 p-1 m-1 rounded"
                >
                    View or Create a Checkpoint
                </button>
            </a>
            <h1 class="text-2xl font-extrabold">Balancing History</h1>
            <p>Current Calorie Goal: {current_calorie_goal} calories</p>
            {details}
            "#
        )
    }
}

pub async fn get_relevant_food(
    db: impl PgExecutor<'_>,
    user_id: i32,
    preferences: &UserPreference,
) -> Aresult<Vec<FoodItem>> {
    struct Qres {
        id: i32,
        calories: i32,
        protein_grams: i32,
        carbohydrates_grams: i32,
        fat_grams: i32,
        food_name: String,
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
            name food_name,
            created_at
        from food
        where created_at at time zone $1 > (
            case when exists (
                select 1 from balancing_checkpoint where user_id = $2
            )
            then (
                select ignore_before
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
    .map(|row| FoodItem {
        id: row.id,
        details: FoodItemDetails {
            calories: row.calories,
            carbohydrates_grams: row.carbohydrates_grams,
            created_at: row.created_at,
            fat_grams: row.fat_grams,
            food_name: row.food_name,
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
    let relevant_food = get_relevant_food(db, user_id, preferences).await?;
    let balancing_history = compute_balancing(
        utc_now(),
        preferences.timezone,
        preferences
            .caloric_intake_goal
            .ok_or(Error::msg("user does not have caloric intake goal"))?,
        preferences.calorie_balancing_max_calories,
        preferences.calorie_balancing_min_calories,
        &relevant_food,
    );
    Ok(balancing_history.current_calorie_goal)
}

pub async fn history(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "balancing history")?;
    let preferences = session.get_preferences(&db).await?;
    let relevant_food =
        get_relevant_food(&db, session.user_id, &preferences).await?;
    let balancing_history = compute_balancing(
        utc_now(),
        preferences.timezone,
        preferences
            .caloric_intake_goal
            .expect("user has caloric intake goal"),
        preferences.calorie_balancing_max_calories,
        preferences.calorie_balancing_min_calories,
        &relevant_food,
    );

    Ok(Page {
        title: "Calorie Balancing",
        children: &PageContainer {
            children: &BalancingHistory {
                result: &balancing_history,
            },
        },
    }
    .render())
}
