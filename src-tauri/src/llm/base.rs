use bytes::Bytes;
use futures_util::StreamExt;
// use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::from_utf8;
use tauri::{AppHandle, Emitter};
use tauri_plugin_http::reqwest;
use tokio::io::{AsyncReadExt, Result};
use tokio_util::io::StreamReader;

pub trait LLM {
    // fn stream_chat(history: ChatHistory) -> AsyncGenerator;
}

pub struct ChatHistory {
    messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    role: Role,
    content: String,
    images: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessageWithId {
    id: String,
    role: Role,
    content: String,
    images: Option<String>,
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

#[derive(Serialize, Deserialize)]
struct OllamaChatResponse {
    model: String,
    created_at: String,
    message: ChatMessage,
    done: bool,
    total_duration: Option<u64>,
    load_duration: Option<u64>,
    prompt_evel_count: Option<u64>,
    prompt_evel_duration: Option<u64>,
    eval_count: Option<u64>,
    eval_duration: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelTag>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OllamaModelTag {
    name: String,
    model: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OllamaModelInfo {
    capabilities: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Clone)]
struct EmittedChatMessage {
    id: String,
    message: ChatMessage,
    done: bool,
}

#[tauri::command]
pub async fn stream_chat(app: AppHandle, messages: Vec<ChatMessageWithId>, model: String) {
    let client = reqwest::Client::new();
    // let messages = vec![ChatMessage {
    //     role: Role::User,
    //     content: "My name is Tao.".to_string(),
    //     images: None,
    // }];
    if messages.len() == 0 {
        return;
    }

    let user_input = messages.last();
    if user_input.is_none() {
        return;
    }

    let id = user_input.unwrap().id.clone();
    let messages_without_id: Vec<ChatMessage> =
        messages.into_iter().map(|msg| msg.strip_id()).collect();

    let req = OllamaChatRequest {
        model,
        messages: messages_without_id,
    };
    let res = client
        .post("http://localhost:11434/api/chat")
        .body(serde_json::to_string(&req).unwrap())
        .send()
        .await
        .unwrap();

    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        match item {
            Ok(byte) => {
                let msg = from_utf8(&byte).unwrap();
                let stream_response: OllamaChatResponse = serde_json::from_str(msg).unwrap();
                let emitted_message = EmittedChatMessage {
                    id: id.clone(),
                    message: stream_response.message,
                    done: stream_response.done,
                };
                _ = app.emit("stream_chat", emitted_message);
            }
            Err(e) => {
                dbg!(e);
                break;
            }
        }
    }

    // dbg!(res);
}

#[tauri::command]
pub async fn get_all_ollama_chat_models() -> Vec<String> {
    let client = reqwest::Client::new();

    let tags = get_all_ollama_models().await;
    let mut chat_models = vec![];

    for tag in tags {
        let json_obj = json!({
            "model": tag.name
        });
        let model_info_res = client
            .post("http://localhost:11434/api/show")
            .body(json_obj.to_string())
            .send()
            .await
            .unwrap();

        let bytes = model_info_res.bytes().await.unwrap();
        let model_info: OllamaModelInfo = serde_json::from_slice(&bytes).unwrap();
        if model_info.capabilities.iter().any(|ca| ca == "completion") {
            chat_models.push(tag.name);
        }
    }

    chat_models
}

async fn get_all_ollama_models() -> Vec<OllamaModelTag> {
    let client = reqwest::Client::new();
    let res = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .unwrap();

    let bytes = res.bytes().await.unwrap();
    let ollama_tags: OllamaTagsResponse = serde_json::from_slice(&bytes).unwrap();
    ollama_tags.models
}

#[tauri::command]
pub fn emit_events(app: AppHandle) {
    let msg_vec = vec![
        "Hello, ", "I", "'", "m ", "Ne", "x", "a. ", "Ho", "w ", "c", "an ", "I ", "h", "el", "p ",
        "y", "ou ", "w", "i", "t", "h?",
    ];

    for s in msg_vec {
        _ = app.emit("testing", s);
    }
}
