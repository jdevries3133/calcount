//! The core calorie counting feature (models, components, and controllers
//! are colocated here).

use super::{llm_parse_response::ParserResult, openai::OpenAI};
use crate::{
    client_events, components::Component, errors::ServerError,
    models::AppState, routes::Route, session::Session,
};
use ammonia::clean;
use anyhow::Result as AResult;
use axum::{
    extract::{Form, State},
    headers::HeaderMap,
    response::IntoResponse,
};
use chrono::prelude::*;
use chrono_tz::Tz;
use serde::Deserialize;
use sqlx::{query, query_as, PgPool};
use std::time::Duration;

#[derive(Debug)]
pub struct Meal {
    id: i32,
    info: MealInfo,
}

#[derive(Debug, Deserialize)]
pub struct MealInfo {
    pub calories: i32,
    pub protein_grams: i32,
    pub carbohydrates_grams: i32,
    pub fat_grams: i32,
    pub meal_name: String,
    pub created_at: DateTime<Utc>,
}

pub struct Chat<'a> {
    pub meals: &'a Vec<Meal>,
}
impl Component for Chat<'_> {
    fn render(&self) -> String {
        let handler = Route::HandleChat;
        let meal_header = if self.meals.is_empty() {
            ""
        } else {
            // Pushing the top up by just 1px hides the text from revealing
            // itself behind the top of this sticky header as the user scrolls
            // through the container; weird browser behavior, weird fix.
            r#"<h2 class="sticky top-[-1px] bg-slate-200 rounded p-2
                dark:text-black text-xl font-bold">
                Saved Items</h2>"#
        };
        let mut found_meal_before_today = false;
        let meals = self.meals.iter().enumerate().fold(
            String::new(),
            |mut acc, (i, meal)| {
                if !found_meal_before_today
                    && is_before_today(
                        &meal.info.created_at,
                        chrono_tz::US::Eastern,
                    )
                    && i != self.meals.len()
                {
                    found_meal_before_today = true;
                    acc.push_str(
                        // Note: the 20rem width matches the width of
                        // `MealCard`
                        r#"
                        <div class="w-[20rem] border-b-4 border-black">
                        <p class="text-xs my-4 dark:text-black">
                            Items after this line were input yesterday, and are
                            not included in your daily totals at the top.
                        </p>
                    </div>
                    "#,
                    )
                };
                acc.push_str(
                    &MealCard {
                        info: &meal.info,
                        meal_id: Some(meal.id),
                        actions: None,
                    }
                    .render(),
                );
                acc
            },
        );
        format!(
            r#"
            <div id="cal-chat-container" class="flex items-center justify-center">
                <div class="rounded bg-slate-200 shadow m-2 p-2 md:p-4">
                    <h1 class="border-b-2 border-slate-600 mb-2 border-black prose serif font-extrabold text-3xl">Calorie Chat</h1>
                    <div class="md:flex md:gap-3 ">
                    <div>
                        <form
                            class="flex flex-col gap-2"
                            hx-post="{handler}"
                        >
                            <label for="chat">
                                <h2
                                    class="dark:text-black text-xl serif bold"
                                >Describe what you're eating</h2>
                            </label>
                            <input
                                class="rounded"
                                autocomplete="one-time-code"
                                type="text"
                                id="chat"
                                name="chat"
                                placeholder="I am eating..."
                            />
                        </form>
                    </div>
                        <div
                            class="flex flex-col gap-2 md:max-h-[80vh] md:overflow-y-scroll"
                        >
                            {meal_header}
                            {meals}
                        </div>
                    </div>
                </div>
            </div>
            "#
        )
    }
}

struct NewMealOptions<'a> {
    info: &'a MealInfo,
}
impl Component for NewMealOptions<'_> {
    fn render(&self) -> String {
        let retry_route = Route::ChatForm;
        let save_route = Route::SaveMeal;
        let calories = self.info.calories;
        let protein = self.info.protein_grams;
        let carbs = self.info.carbohydrates_grams;
        let fat = self.info.fat_grams;
        let created_at = self.info.created_at;
        let meal_name = clean(&self.info.meal_name);
        format!(
            r##"
            <form hx-post="{save_route}" hx-target="#cal-chat-container">
                <input type="hidden" value="{meal_name}" name="meal_name" />
                <input type="hidden" value="{calories}" name="calories" />
                <input type="hidden" value="{protein}" name="protein_grams" />
                <input type="hidden" value="{carbs}" name="carbohydrates_grams" />
                <input type="hidden" value="{fat}" name="fat_grams" />
                <input type="hidden" value="{created_at}" name="created_at" />
                <button
                    class="bg-blue-100 p-1 rounded shadow hover:bg-blue-200"
                >Save</button>
                <button
                    hx-get="{retry_route}"
                    hx-target="#cal-chat-container"
                    class="bg-red-100 p-1 rounded shadow hover:bg-red-200"
                >Try Again</button>
            </form>
            "##
        )
    }
}

impl Component for MealInfo {
    fn render(&self) -> String {
        MealCard {
            info: self,
            meal_id: None,
            actions: Some(&NewMealOptions { info: self }),
        }
        .render()
    }
}

pub struct MealCard<'a> {
    pub info: &'a MealInfo,
    pub meal_id: Option<i32>,
    pub actions: Option<&'a dyn Component>,
}
impl Component for MealCard<'_> {
    fn render(&self) -> String {
        let meal_name = clean(&self.info.meal_name);
        let calories = self.info.calories;
        let protein = self.info.protein_grams;
        let carbs = self.info.carbohydrates_grams;
        let fat = self.info.fat_grams;
        let actions = match &self.actions {
            Some(action) => action.render(),
            None => match self.meal_id {
                Some(id) => {
                    let href = Route::DeleteMeal(Some(id));
                    format!(
                        r#"<button
                            hx-delete="{href}"
                            hx-target="closest div[data-name='meal-card']"
                            class="align-self-right bg-red-100 hover:bg-red-200 rounded p-1"
                        >
                        Delete
                    </button>"#
                    )
                }
                None => "".into(),
            },
        };
        let background_style = if is_before_today(
            &self.info.created_at,
            Tz::US__Eastern,
        ) {
            "border-4 border-black"
        } else {
            "bg-gradient-to-tr from-violet-200 border-t-4 border-l-4 border-slate-300"
        };
        format!(
            r##"
            <div
                class="dark:text-black to-sky-100 rounded p-2 shadow sm:w-[20rem] mr-4
                {background_style}
                "
                data-name="meal-card"
            >
                <h1 class="text-2xl bold serif">{meal_name}</h1>
                <p class="text-lg"><b>Calories:</b> {calories} kcal</p>
                <p class="text-sm"><b>Protein:</b> {protein} grams</p>
                <p class="text-sm"><b>Carbs:</b> {carbs} grams</p>
                <p class="text-sm"><b>Fat:</b> {fat} grams</p>
                <div class="flex justify-end">
                    {actions}
                </div>
            </div>
            "##
        )
    }
}

pub struct CannotParse<'a> {
    parser_msg: &'a str,
    llm_response: &'a str,
}
impl Component for CannotParse<'_> {
    fn render(&self) -> String {
        let parser_msg = clean(self.parser_msg);
        let llm_response = clean(self.llm_response);
        format!(
            r##"
            <div class="prose">
                <p><b>LLM response:</b> {llm_response}</p>
                <p
                    class="text-sm text-slate-600"
                ><b>Error parsing LLM Response:</b> {parser_msg}</p>
                    <button
                        hx-get="/chat-form"
                        hx-target="#cal-chat-container"
                        class="bg-blue-100 p-1 rounded shadow hover:bg-blue-200"
                    >Try Again</button>
            </div>
            "##
        )
    }
}

#[derive(Deserialize)]
pub struct ChatPayload {
    chat: String,
}

const SYSTEM_MSG: &str = "I am overweight, and I've been trying to lose weight for a long time. My biggest problem is counting calories, and understanding the macronutrients of the food I eat. As we both know, nutrition is a somewhat inexact science. A close answer to questions about calories has a lot of value to me, and as long as many answers over time are roughly correct on average, I can finally make progress in my weight loss journey. When I ask you about the food I eat, please provide a concise and concrete estimate of the amount of calories and macronutrient breakdown of the food I describe. A macronutrient breakdown is the amount of protein, carbohydrates, and fat, each measured in grams. Always provide exactly one number each for calories, grams of protein, grams of carbohydrates, and grams of fat to ensure that I can parse your message using some simple regular expressions. Do not, for example, identify the macros of a single portion and then provide the exact macros at the end. I'll probably get confused and ignore the second set of macros. Please match this style in your response: \"The food you asked about has {} calories, {}g of protein, {}g of fat, and {}g of carbohydrates.";

pub async fn handle_chat(
    Form(ChatPayload { chat }): Form<ChatPayload>,
) -> Result<impl IntoResponse, ServerError> {
    let mut msg = String::from("The meal I'd like a calorie estimate for is ");
    msg.push_str(&chat);
    let response = OpenAI::from_env()?.send_message(SYSTEM_MSG.into(), msg)?;
    let parse_result = MealInfo::parse(&response, &chat);
    match parse_result {
        ParserResult::Ok(meal) => Ok(meal.render()),
        ParserResult::FollowUp(msg) => {
            let msg = clean(&msg.parsing_error);
            Ok(CannotParse {
                parser_msg: &msg,
                llm_response: &response,
            }
            .render())
        }
    }
}

pub async fn chat_form(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers(&headers)
        .ok_or_else(|| ServerError::forbidden("chat form"))?;
    let meals = get_meals(&db, session.user.id).await?;
    let chat = Chat { meals: &meals };
    let content = chat.render();
    Ok(content)
}

pub async fn get_meals(db: &PgPool, user_id: i32) -> AResult<Vec<Meal>> {
    struct Qres {
        id: i32,
        meal_name: String,
        calories: i32,
        fat_grams: i32,
        protein_grams: i32,
        carbohydrates_grams: i32,
        created_at: DateTime<Utc>,
    }
    let mut res = query_as!(
        Qres,
        "select
            id,
            name meal_name,
            calories,
            fat fat_grams,
            protein protein_grams,
            carbohydrates carbohydrates_grams,
            created_at
        from meal
        where user_id = $1
        order by created_at desc
        ",
        user_id
    )
    .fetch_all(db)
    .await?;

    Ok(res
        .drain(..)
        .map(|r| Meal {
            id: r.id,
            info: MealInfo {
                meal_name: r.meal_name,
                calories: r.calories,
                carbohydrates_grams: r.carbohydrates_grams,
                fat_grams: r.fat_grams,
                protein_grams: r.protein_grams,
                created_at: r.created_at,
            },
        })
        .collect::<Vec<Meal>>())
}

pub async fn handle_save_meal(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    Form(meal): Form<MealInfo>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers(&headers)
        .ok_or_else(|| ServerError::forbidden("handle save meal"))?;
    query!(
        "insert into meal (user_id, name, calories, fat, protein, carbohydrates)
        values ($1, $2, $3, $4, $5, $6)",
        session.user.id,
        meal.meal_name,
        meal.calories,
        meal.fat_grams,
        meal.protein_grams,
        meal.carbohydrates_grams
    )
    .execute(&db)
    .await?;
    let response_headers = client_events::reload_macros(HeaderMap::new());
    let meals = get_meals(&db, session.user.id).await?;
    Ok((response_headers, Chat { meals: &meals }.render()))
}

/// Check if `time` is yesterday or before.
fn is_before_today(
    datetime: &DateTime<Utc>,
    user_timezone: chrono_tz::Tz,
) -> bool {
    let local_dt = datetime.with_timezone(&user_timezone);
    let date = local_dt.date_naive();
    let yesterday = (Utc::now() - Duration::from_secs(60 * 60 * 24))
        .with_timezone(&user_timezone)
        .date_naive();
    date <= yesterday
}
