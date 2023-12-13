//! The core calorie counting feature (models, components, and controllers
//! are colocated here).

use super::{
    llm_parse_response::{MealCard, MealInfo, ParserResult},
    openai::OpenAI,
};
use crate::{
    components::Component, errors::ServerError, models::AppState,
    routes::Route, session::Session,
};
use ammonia::clean;
use anyhow::Result as AResult;
use axum::{
    extract::{Form, State},
    headers::HeaderMap,
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::{query, query_as, PgPool};

pub struct Chat<'a> {
    pub meals: &'a Vec<MealInfo>,
}
impl Component for Chat<'_> {
    fn render(&self) -> String {
        let handler = Route::HandleChat;
        let meals = self.meals.iter().fold(String::new(), |mut acc, meal| {
            acc.push_str(&MealCard { info: meal }.render());
            acc
        });
        format!(
            r#"
            <div id="cal-chat-container" class="prose rounded bg-slate-200 shadow m-2 p-2">
            <h1>Calorie Chat</h1>
            <form
                class="flex flex-col gap-2"
                hx-post="{handler}"
            >
                <label for="chat">Describe what you're eating</label>
                <input autocomplete="one-time-code" type="text" id="chat" name="chat" placeholder="I am eating..." />
            </form>
            {meals}
            </div>
            "#
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
            <p><b>LLM response:</b> {llm_response}</p>
            <p class="text-sm text-slate-600"><b>Error parsing LLM Response:</b> {parser_msg}</p>
                <button
                    hx-get="/chat-form"
                    hx-target="#cal-chat-container"
                    class="bg-blue-100 p-1 rounded shadow hover:bg-blue-200"
                >Try Again</button>
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

pub async fn get_meals(db: &PgPool, user_id: i32) -> AResult<Vec<MealInfo>> {
    Ok(query_as!(
        MealInfo,
        "select name meal_name, calories, fat fat_grams, protein protein_grams, carbohydrates carbohydrates_grams
        from meal
        where user_id = $1
        order by id desc
        ",
        user_id
    ).fetch_all(db).await?)
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
    let meals = get_meals(&db, session.user.id).await?;
    Ok(Chat { meals: &meals }.render())
}
