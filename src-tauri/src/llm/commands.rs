use crate::api::gemini::{FunctionDeclaration, Tool};
use crate::error::NexaError;
use crate::llm::base::{
    ChatHistory, ChatMessage, ChatMessageContent, EmittedChatMessage, Provider, LLM,
};
use crate::llm::constants::GEMINI_KETRING_KEY;
use crate::llm::gemini::Gemini;
use crate::llm::ollama::{
    OllamaChatMessage, OllamaChatRequest, OllamaChatResponse, OllamaModelInfo, OllamaModelTag,
    OllamaTagsResponse,
};
use crate::AppData;
use futures::pin_mut;
use futures_util::StreamExt;
use keyring::Entry;
use serde_json::json;
use std::collections::HashMap;
use std::str::from_utf8;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_http::reqwest;

#[tauri::command]
pub async fn stream_chat(
    app: AppHandle,
    state: State<'_, AppData>,
    history: ChatHistory,
    model: String,
    provider: Provider,
) -> Result<(), NexaError> {
    let client = reqwest::Client::new();

    dbg!(&history);

    if history.messages.len() == 0 {
        return Err(NexaError::Command(String::from(
            "Stream chat command without chat history",
        )));
    }

    let user_input = history.messages.last();
    if user_input.is_none() {
        return Err(NexaError::Command(String::from(
            "Stream chat command without user input",
        )));
    }

    match provider {
        Provider::Gemini => {
            let product_name = app.config().product_name.clone();
            if product_name.is_none() {
                return Err(NexaError::Command(String::from("Product name is none")));
            }

            // Need to move this to setup maybe
            let product_name = product_name.unwrap();
            let entry = Entry::new(&product_name, GEMINI_KETRING_KEY).expect("Keyring Error");
            let api_key = entry.get_password().expect("Keychain Error");

            let mcp_clients = state.mcp_clients.read().await;
            let mut tools: Vec<Tool> = vec![];

            for (name, mcp_client) in mcp_clients.iter() {
                let tool_list = mcp_client.get_tool_list().await;
                let function_decorations: Vec<FunctionDeclaration> = tool_list
                    .iter()
                    .map(|(name, tool)| FunctionDeclaration {
                        name: name.clone(),
                        description: tool.description.clone().unwrap_or_default(),
                        parameters: Some(serde_json::to_value(&tool.input_schema).unwrap()),
                        extra_fields: json!({}),
                    })
                    .collect();

                tools.push(Tool {
                    function_declarations: Some(function_decorations),

                    extra_fields: json!({}),
                });
            }

            let gemini = Gemini {
                model_id: model,
                tools: tools,
                api_key: api_key,
                tool_config: None,
            };

            let stream = gemini.stream_chat(history).await;
            if stream.is_err() {
                return Err(NexaError::Gemini(String::from(format!(
                    "Stream chat error: {}",
                    stream.err().unwrap()
                ))));
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

            Ok(())
        }
        Provider::Ollama => {
            // Ollama section
            let id = user_input.unwrap().id.clone();
            let mut messages: Vec<OllamaChatMessage> = vec![];

            for msg in history.messages.into_iter() {
                if let ChatMessageContent::Text(text) = msg.content {
                    messages.push(OllamaChatMessage {
                        role: msg.role,
                        content: text,
                        images: msg.images,
                    });
                }
            }

            let req = OllamaChatRequest {
                model,
                messages: messages,
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
                            message: vec![ChatMessage {
                                role: stream_response.message.role,
                                images: stream_response.message.images,
                                content: ChatMessageContent::Text(stream_response.message.content),
                            }],
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

            Ok(())

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
