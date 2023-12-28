use crate::config;
use anyhow::{Error, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

pub struct OpenAI {
    client: Client,
    api_key: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatCompletionMessage>,
}

#[derive(Serialize)]
struct ChatCompletionMessage {
    role: MessageRole,
    content: String,
}

#[derive(Serialize)]
#[allow(non_camel_case_types)]
enum MessageRole {
    user,
    system,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionResponseMessage>,
    usage: Usage,
}

#[derive(Deserialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Deserialize)]
struct ChatCompletionResponseMessage {
    message: ChatCompletionResponseMessageContent,
}

#[derive(Deserialize)]
struct ChatCompletionResponseMessageContent {
    content: Option<String>,
}

pub struct Response {
    pub message: String,
    pub usage: Usage,
}

impl OpenAI {
    pub fn from_env() -> Result<Self> {
        let api_key = env::var("OPENAI_API_KEY")?;
        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }
    pub async fn send_message(
        &self,
        system_msg: String,
        meal_description: &str,
    ) -> Result<Response> {
        if meal_description.len() > config::CHAT_MAX_LEN {
            return Err(Error::msg("tried to send a chat which is too long"));
        };
        let mut user_message =
            String::from("The meal I'd like a calorie estimate for is ");
        user_message.push_str(meal_description);
        let payload = ChatCompletionRequest {
            model: "gpt-3.5-turbo-1106".into(),
            messages: vec![
                ChatCompletionMessage {
                    role: MessageRole::system,
                    content: system_msg,
                },
                ChatCompletionMessage {
                    role: MessageRole::user,
                    content: user_message,
                },
            ],
        };
        let req = self
            .client
            .post("https://api.openai.com/v1/chat/completions");
        let req =
            req.header("Authorization", format!("Bearer {}", self.api_key));
        let req = req.json(&payload);
        let res = req.send().await?;
        let text = res.text().await?;
        let mut res: ChatCompletionResponse = serde_json::from_str(&text)?;

        Ok(Response {
            message: res.choices[0].message.content.take().unwrap_or("".into()),
            usage: res.usage,
        })
    }
}
