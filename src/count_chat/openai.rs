use anyhow::{Error, Result};
use openai_api_rs::v1::{
    api::Client,
    chat_completion::{
        ChatCompletionMessage, ChatCompletionRequest, FinishReason,
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
    pub fn send(&self, messages: Vec<ChatCompletionMessage>) -> Result<String> {
        let request =
            ChatCompletionRequest::new(GPT3_5_TURBO_1106.into(), messages);
        let result = &self.client.chat_completion(request)?.choices[0];
        match &result.finish_reason {
            Some(ref reason) => match reason {
                FinishReason::stop => match &result.message.content {
                    Some(c) => Ok(c.clone()),
                    None => {
                        Err(Error::msg("message does not have any content"))
                    }
                },
                reason => Err(Error::msg(format!(
                    "finish reason is {reason:?}, not 'stop'"
                ))),
            },
            None => Err(Error::msg("finish reason is missing")),
        }
    }
}
