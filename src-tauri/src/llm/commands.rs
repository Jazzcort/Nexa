use crate::llm::base::{ChatHistory, ChatMessage, EmittedChatMessage, Provider, LLM};
use crate::llm::constants::GEMINI_KETRING_KEY;
use crate::llm::gemini::Gemini;
use crate::llm::ollama::{
    OllamaChatRequest, OllamaChatResponse, OllamaModelInfo, OllamaModelTag, OllamaTagsResponse,
};
use futures::pin_mut;
use futures_util::{Stream, StreamExt};
use keyring::Entry;
use serde_json::json;
use std::pin::pin;
use std::str::from_utf8;
use tauri::{AppHandle, Emitter};
use tauri_plugin_http::reqwest;

#[tauri::command]
pub async fn stream_chat(app: AppHandle, history: ChatHistory, model: String, provider: Provider) {
    let client = reqwest::Client::new();
    // let messages = vec![ChatMessage {
    //     role: Role::User,
    //     content: "My name is Tao.".to_string(),
    //     images: None,
    // }];
    dbg!(&history);
    if history.messages.len() == 0 {
        return;
    }

    let user_input = history.messages.last();
    if user_input.is_none() {
        return;
    }

    match provider {
        Provider::Gemini => {
            let product_name = app.config().product_name.clone();
            if product_name.is_none() {
                return;
            }

            let product_name = product_name.unwrap();
            let entry = Entry::new(&product_name, GEMINI_KETRING_KEY).expect("Keyring Error");
            let api_key = entry.get_password().expect("Keychain Error");

            let gemini = Gemini {
                model_id: model,
                tools: vec![],
                api_key: api_key,
                tool_config: None,
            };

            let stream = gemini.stream_chat(history).await;
            if stream.is_err() {
                return;
            }

            let stream = stream.unwrap();
            pin_mut!(stream);

            while let Some(item) = stream.next().await {
                match item {
                    Ok(message) => {
                        _ = app.emit("stream_chat", message);
                    }
                    Err(e) => {
                        dbg!(e);
                    }
                }
            }
        }
        Provider::Ollama => {
            // Ollama section
            let id = user_input.unwrap().id.clone();
            let messages_without_id: Vec<ChatMessage> = history
                .messages
                .into_iter()
                .map(|msg| msg.strip_id())
                .collect();

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
                        let stream_response: OllamaChatResponse =
                            serde_json::from_str(msg).unwrap();
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
    }
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

    dbg!(&chat_models);

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
