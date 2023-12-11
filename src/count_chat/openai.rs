use anyhow::Result;
use openai_api_rs::v1::{
    api::Client,
    chat_completion::{
        ChatCompletionMessage, ChatCompletionRequest, MessageRole,
    },
    common::GPT3_5_TURBO_1106,
};
use std::env;

pub struct OpenAI {
    client: Client,
}

impl OpenAI {
    pub fn from_env() -> Result<Self> {
        let api_key = env::var("OPENAI_API_KEY")?;
        Ok(Self {
            client: Client::new(api_key),
        })
    }
    pub fn send_message(
        &self,
        system_msg: String,
        usr_msg: String,
    ) -> Result<String> {
        let request = ChatCompletionRequest::new(
            GPT3_5_TURBO_1106.into(),
            vec![
                ChatCompletionMessage {
                    role: MessageRole::system,
                    content: system_msg,
                    name: None,
                    function_call: None,
                },
                ChatCompletionMessage {
                    role: MessageRole::user,
                    content: usr_msg,
                    name: None,
                    function_call: None,
                },
            ],
        );
        let result = self.client.chat_completion(request)?;
        Ok(result.choices[0]
            .message
            .content
            .clone()
            .unwrap_or("".into()))
    }
}
