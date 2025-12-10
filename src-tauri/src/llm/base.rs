use crate::error::NexaError;
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    Function,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: ChatMessageContent,
    pub images: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "content")]
#[serde(rename_all = "camelCase")]
pub enum ChatMessageContent {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "_meta")]
        _meta: Option<Value>,
    },
    FunctionCallRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "_meta")]
        _meta: Option<Value>,
    },
    FunctionCallResponse {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        name: String,
        response: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "_meta")]
        _meta: Option<Value>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessageWithId {
    pub id: String,
    pub role: Role,
    pub content: ChatMessageContent,
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
    pub message: Vec<ChatMessage>,
    pub done: bool,
}
