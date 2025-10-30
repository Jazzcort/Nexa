use crate::llm::base::ChatMessage;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct OllamaChatResponse {
    pub(crate) model: String,
    pub(crate) created_at: String,
    pub(crate) message: ChatMessage,
    pub(crate) done: bool,
    pub(crate) total_duration: Option<u64>,
    pub(crate) load_duration: Option<u64>,
    pub(crate) prompt_evel_count: Option<u64>,
    pub(crate) prompt_evel_duration: Option<u64>,
    pub(crate) eval_count: Option<u64>,
    pub(crate) eval_duration: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct OllamaTagsResponse {
    pub(crate) models: Vec<OllamaModelTag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct OllamaModelTag {
    pub(crate) name: String,
    pub(crate) model: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct OllamaModelInfo {
    pub(crate) capabilities: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct OllamaChatRequest {
    pub(crate) model: String,
    pub(crate) messages: Vec<ChatMessage>,
}

pub(crate) struct Ollama {
    pub(crate) model: String,
}
