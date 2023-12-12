//! The core calorie counting feature (models, components, and controllers
//! are colocated here).

use super::{
    llm_parse_response::{MealInfo, ParserResult},
    openai::OpenAI,
};
use crate::{components::Component, errors::ServerError, routes::Route};
use ammonia::clean;
use axum::{extract::Form, response::IntoResponse};
use openai_api_rs::v1::chat_completion::{ChatCompletionMessage, MessageRole};
use serde::Deserialize;

pub struct Chat;
impl Component for Chat {
    fn render(&self) -> String {
        let handler = Route::HandleChat;
        format!(
            r#"
            <div class="prose rounded bg-slate-200 shadow m-2 p-2">
            <h1>Calorie Chat</h1>
            <form
                class="flex flex-col gap-2"
                hx-post="{handler}"
            >
                <label for="chat">Describe what you're eating</label>
                <input autocomplete="one-time-code" type="text" id="chat" name="chat" placeholder="I am eating..." />
            </form>
            "#
        )
    }
}

pub struct CannotParse<'a> {
    parser_msg: &'a str,
    llm_responses: Vec<&'a str>,
}
impl Component for CannotParse<'_> {
    fn render(&self) -> String {
        let msg = clean(self.parser_msg);
        let llm_responses = self.llm_responses.iter().enumerate().fold(
            String::new(),
            |mut acc, (i, response)| {
                let i = i + 1;
                let sanitized = clean(response);
                acc.push_str(&format!(
                    "<p><b>Response {i}</b>: {sanitized}</p>"
                ));
                acc
            },
        );
        format!(
            r#"
            <p>{msg}</p>
            {llm_responses}
            "#
        )
    }
}

#[derive(Deserialize)]
pub struct ChatForm {
    chat: String,
}

const SYSTEM_MSG: &str = "I am overweight, and I've been trying to lose weight for a long time. My biggest problem is counting calories, and understanding the macronutrients of the food I eat. As we both know, nutrition is a somewhat inexact science. A close answer to questions about calories has a lot of value to me, and as long as many answers over time are roughly correct on average, I can finally make progress in my weight loss journey. When I ask you about the food I eat, please provide a concise and concrete estimate of the amount of calories and macronutrient breakdown of the food I describe. A macronutrient breakdown is the amount of protein, carbohydrates, and fat, each measured in grams. Always provide exactly one number each for calories, grams of protein, grams of carbohydrates, and grams of fat to ensure that I can parse your message using some simple regular expressions. Do not, for example, identify the macros of a single portion and then provide the exact macros at the end. I'll probably get confused and ignore the second set of macros. Please match this style in your response: \"The food you asked about has {} calories, {}g of protein, {}g of fat, and {}g of carbohydrates.";

pub async fn handle_chat(
    Form(ChatForm { chat }): Form<ChatForm>,
) -> Result<impl IntoResponse, ServerError> {
    let mut user_msg =
        String::from("The meal I'd like a calorie estimate for is ");
    user_msg.push_str(&chat);
    let mut messages = vec![
        ChatCompletionMessage {
            role: MessageRole::system,
            content: SYSTEM_MSG.into(),
            name: None,
            function_call: None,
        },
        ChatCompletionMessage {
            role: MessageRole::user,
            content: user_msg,
            name: None,
            function_call: None,
        },
    ];
    let response = OpenAI::from_env()?.send(messages.clone())?;
    let parse_result = MealInfo::parse(&response);
    match parse_result {
        ParserResult::Ok(meal) => Ok(meal.render()),
        ParserResult::FollowUp(msg) => {
            // On the first follow-up, we will retry.
            messages.push(ChatCompletionMessage {
                role: MessageRole::system,
                content: msg.llm_reply,
                name: None,
                function_call: None,
            });
            let response_2 = OpenAI::from_env()?.send(messages)?;
            let parse_result_2 = MealInfo::parse(&response_2);
            match parse_result_2 {
                ParserResult::Ok(meal) => Ok(meal.render()),
                ParserResult::FollowUp(err) => {
                    // On the second follow-up, we'll finally send the
                    // user-facing error.
                    let err = clean(&err.user_abort_msg);
                    Ok(CannotParse {
                        parser_msg: &err,
                        llm_responses: vec![&response, &response_2],
                    }
                    .render())
                }
            }
        }
    }
}
