use anyhow::Result;
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
    choices: Vec<Response>,
}

#[derive(Deserialize)]
struct Response {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: Option<String>,
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
        usr_msg: String,
    ) -> Result<String> {
        let payload = ChatCompletionRequest {
            model: "gpt-3.5-turbo-1106".into(),
            messages: vec![
                ChatCompletionMessage {
                    role: MessageRole::system,
                    content: system_msg,
                },
                ChatCompletionMessage {
                    role: MessageRole::user,
                    content: usr_msg,
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
        let res: ChatCompletionResponse = res.json().await?;

        Ok(res.choices[0].message.content.clone().unwrap_or("".into()))
    }
}
