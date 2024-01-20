use super::{counter, llm_parse_response, openai};
use crate::{config, prelude::*};
use rand::random;

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
        let options = [
            "5 second squeeze of honey",
            "hummus on brioche bread",
            "gigantic cheese burger",
            "half a dunkin boston cream",
            "3 handfuls of chex mix",
            "a greasy cheese burger",
            "a frozen chicken cutlet",
            "really big diner breakfast (traditional American)",
            "caesar salad & 10 stolen fries",
        ];
        let i = random::<usize>() % options.len();
        counter::ChatUI {
            post_request_handler: &Route::ChatDemo,
            prefill_prompt: self.prefill_prompt.or(Some(options[i])),
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
    if chat.len() > config::CHAT_MAX_LEN {
        return Ok(counter::InputTooLong {}.render());
    };
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
            rendering_behavior: counter::RenderingBehavior::RenderAsToday,
            show_ai_warning: true,
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
