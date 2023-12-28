use super::{counter, llm_parse_response, openai};
use crate::prelude::*;

struct DemoMealOptions;
impl Component for DemoMealOptions {
    fn render(&self) -> String {
        let demo = Route::ChatDemo;
        format!(
            r##"
            <button
                class="bg-green-100 p-1 rounded shadow hover:bg-green-200"
                hx-target="#cal-chat-container"
                hx-get="{demo}"
            >Reset</button>
            "##
        )
    }
}

pub struct ChatDemo<'a> {
    pub prefill_prompt: Option<&'a str>,
}
impl Component for ChatDemo<'_> {
    fn render(&self) -> String {
        counter::ChatUI {
            post_request_handler: &Route::ChatDemo,
            prefill_prompt: self.prefill_prompt,
            children: None,
        }
        .render()
    }
}

pub async fn get_demo_ui() -> impl IntoResponse {
    ChatDemo {
        prefill_prompt: None,
    }
    .render()
}

#[derive(Deserialize)]
pub struct RetryPayload {
    meal_name: String,
}

pub async fn handle_retry(Form(form): Form<RetryPayload>) -> impl IntoResponse {
    ChatDemo {
        prefill_prompt: Some(&form.meal_name),
    }
    .render()
}

pub async fn handle_chat(
    State(AppState { db }): State<AppState>,
    Form(counter::ChatPayload { chat }): Form<counter::ChatPayload>,
) -> Result<impl IntoResponse, ServerError> {
    let response = openai::OpenAI::from_env()?
        .send_message(counter::SYSTEM_MSG.into(), &chat)
        .await?;
    query!(
        "insert into openai_usage (prompt_tokens, completion_tokens, total_tokens)
        values ($1, $2, $3)",
        response.usage.prompt_tokens,
        response.usage.completion_tokens,
        response.usage.total_tokens
    ).execute(&db).await?;
    let parse_result = counter::MealInfo::parse(&response.message, &chat);
    match parse_result {
        llm_parse_response::ParserResult::Ok(meal) => Ok(counter::MealCard {
            info: &meal,
            meal_id: None,
            actions: Some(&DemoMealOptions {}),
            // We don't know where the user is, but it also doesn't really
            // matter. We just want to ensure that this meal card renders with
            // the current-day style, and not the "yesterday" style. The meal's
            // creation time is basically "now" so as long as we choose a
            // timezone with a very negative offset, the timezone-aware
            // date comparison will end up determining that the meal created
            // "now" UTC is not yesterday or before. If anything, the meal
            // might end up being in the future from the users' real
            // perspective, but again for this specific use-case
            // that is OK.
            user_timezone: Tz::US__Samoa,
        }
        .render()),
        llm_parse_response::ParserResult::FollowUp(msg) => {
            let msg = clean(&msg.parsing_error);
            Ok(counter::CannotParse {
                parser_msg: &msg,
                llm_response: &response.message,
                original_user_prompt: &chat,
                retry_route: Route::ChatDemoRetry,
            }
            .render())
        }
    }
}
