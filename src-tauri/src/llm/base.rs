use futures::Future;
use futures_util::pin_mut;
use futures_util::stream::Stream;
use futures_util::StreamExt;
// use futures_util::StreamExt;
use crate::error::NexaError;
use crate::llm::ollama::{
    OllamaChatRequest, OllamaChatResponse, OllamaModelInfo, OllamaModelTag, OllamaTagsResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::from_utf8;
use tauri::{AppHandle, Emitter};
use tauri_plugin_http::reqwest;
// use tokio::io::{AsyncReadExt, Result};
// use tokio_util::io::StreamReader;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Ollama,
    Gemini,
}

pub trait LLM {
    async fn stream_chat(
        &self,
        history: ChatHistory,
    ) -> Result<impl Stream<Item = Result<EmittedChatMessage, NexaError>>, NexaError>;
}

#[derive(Deserialize, Debug)]
pub struct ChatHistory {
    pub messages: Vec<ChatMessageWithId>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub images: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessageWithId {
    pub id: String,
    pub role: Role,
    pub content: String,
    pub images: Option<String>,
}

impl ChatMessageWithId {
    pub fn strip_id(&self) -> ChatMessage {
        ChatMessage {
            role: self.role.clone(),
            content: self.content.clone(),
            images: self.images.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EmittedChatMessage {
    pub id: String,
    pub message: ChatMessage,
    pub done: bool,
}
