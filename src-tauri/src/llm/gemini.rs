use crate::api::gemini::{gemini_chat, GeminiPartMetadata};
use crate::api::gemini::{Content, GeminiPart, GeminiPartData, Tool, ToolConfig};
use crate::error::NexaError;
use crate::llm::base::{
    ChatHistory, ChatMessage, ChatMessageContent, EmittedChatMessage, Role, LLM,
};
use futures::stream;
use futures::StreamExt;
use futures_util::Stream;
use serde_json::{json, Value};

pub struct Gemini {
    pub model_id: String,
    pub tools: Vec<Tool>,
    pub api_key: String,
    pub tool_config: Option<ToolConfig>,
}

pub static GEMINI_META: &str = "x-gemini-meta";

impl LLM for Gemini {
    async fn stream_chat(
        &self,
        mut history: ChatHistory,
    ) -> Result<impl Stream<Item = Result<EmittedChatMessage, NexaError>>, NexaError> {
        let last_message = history
            .messages
            .pop()
            .ok_or(NexaError::Gemini("Empty chat history".to_string()))?;
        let id = last_message.id.clone();

        let mut combined_contents: Vec<Content> = vec![];
        let mut cur_role = Role::User;
        let mut cur_content = Content {
            parts: vec![],
            role: Some(get_gemini_role(cur_role.clone())),
        };

        for msg in history.messages.into_iter() {
            // let (mut thought, mut thought_signature, mut metadata, mut part_metadata) =
            //     (None, None, None, None);

            let part = match msg.content {
                ChatMessageContent::Text { text, _meta } => {
                    let (thought, thought_signature, metadata, part_metadata) =
                        get_gemini_meta_fields(_meta);

                    GeminiPart {
                        thought,
                        thought_signature,
                        data: GeminiPartData::Text(text),
                        metadata,
                        part_metadata,
                    }
                }
                ChatMessageContent::FunctionCallRequest {
                    id,
                    name,
                    args,
                    _meta,
                } => {
                    let (thought, thought_signature, metadata, part_metadata) =
                        get_gemini_meta_fields(_meta);

                    GeminiPart {
                        thought,
                        thought_signature,
                        data: GeminiPartData::FunctionCall { id, name, args },
                        metadata,
                        part_metadata,
                    }
                }
                ChatMessageContent::FunctionCallResponse {
                    id,
                    name,
                    response,
                    _meta,
                } => {
                    let (thought, thought_signature, metadata, part_metadata) =
                        get_gemini_meta_fields(_meta);

                    GeminiPart {
                        thought,
                        thought_signature,
                        data: GeminiPartData::FunctionResponse {
                            id,
                            name,
                            response,
                            parts: None,
                            will_continue: None,
                            scheduling: None,
                        },
                        metadata,
                        part_metadata,
                    }
                }
            };

            if msg.role == cur_role {
                cur_content.parts.push(part);
            } else {
                if !cur_content.parts.is_empty() {
                    combined_contents.push(cur_content);
                }

                cur_role = msg.role.clone();
                cur_content = Content {
                    parts: vec![part],
                    role: Some(get_gemini_role(msg.role.clone())),
                };
            }
        }

        combined_contents.push(cur_content);

        // let contents: Vec<Content> = history
        //     .messages
        //     .into_iter()
        //     .map(|message| {
        //         let mut content = Content {
        //             parts: vec![],
        //             role: Some(get_gemini_role(message.role.clone())),
        //         };
        //
        //         match message.content {
        //             ChatMessageContent::Text { text, _meta } => {
        //                 let (thought, thought_signature, metadata, part_metadata) =
        //                     get_gemini_meta_fields(_meta);
        //
        //                 content.parts.push(GeminiPart {
        //                     thought,
        //                     thought_signature,
        //                     data: GeminiPartData::Text(text),
        //                     metadata,
        //                     part_metadata,
        //                 });
        //                 content
        //             }
        //             ChatMessageContent::FunctionCallRequest {
        //                 id,
        //                 name,
        //                 args,
        //                 _meta,
        //             } => {
        //                 let (thought, thought_signature, metadata, part_metadata) =
        //                     get_gemini_meta_fields(_meta);
        //
        //                 content.parts.push(GeminiPart {
        //                     thought,
        //                     thought_signature,
        //                     data: GeminiPartData::FunctionCall { id, name, args },
        //                     metadata,
        //                     part_metadata,
        //                 });
        //                 content
        //             }
        //             ChatMessageContent::FunctionCallResponse {
        //                 id,
        //                 name,
        //                 response,
        //                 _meta,
        //             } => {
        //                 let (thought, thought_signature, metadata, part_metadata) =
        //                     get_gemini_meta_fields(_meta);
        //
        //                 content.parts.push(GeminiPart {
        //                     thought,
        //                     thought_signature,
        //                     data: GeminiPartData::FunctionResponse {
        //                         id,
        //                         name,
        //                         response,
        //                         parts: None,
        //                         will_continue: None,
        //                         scheduling: None,
        //                     },
        //                     metadata,
        //                     part_metadata,
        //                 });
        //                 content
        //             }
        //         }
        //     })
        //     .collect();

        let stream = gemini_chat(
            combined_contents,
            self.tools.clone(),
            self.model_id.clone(),
            self.api_key.clone(),
            self.tool_config.clone(),
        )
        .await?;

        let should_terminate_stream = false;
        let boxed_stream = Box::pin(stream);

        Ok(stream::unfold(
            (boxed_stream, should_terminate_stream, id),
            |(mut stream, mut should_terminate_stream, id)| async move {
                let mut yielded_item = EmittedChatMessage {
                    id: id.clone(),
                    message: vec![],
                    done: false,
                };

                if should_terminate_stream {
                    return None;
                }

                if let Some(item) = stream.next().await {
                    match item {
                        Ok(gemini_response) => {
                            if gemini_response.candidates.len() < 1 {
                                return Some((
                                    Err(NexaError::Gemini(
                                        "No candidate in the response".to_string(),
                                    )),
                                    (stream, should_terminate_stream, id),
                                ));
                            }

                            let first_candidate =
                                gemini_response.candidates.first().unwrap().clone();

                            for part in first_candidate.content.parts {
                                let _meta = Some(generate_gemini_part_meta_value(&part));

                                match part.data {
                                    GeminiPartData::Text(msg) => {
                                        yielded_item.message.push(ChatMessage {
                                            role: Role::Assistant,
                                            content: ChatMessageContent::Text { text: msg, _meta },
                                            images: None,
                                        })
                                    }
                                    GeminiPartData::FunctionCall { id, name, args } => {
                                        yielded_item.message.push(ChatMessage {
                                            role: Role::Assistant,
                                            content: ChatMessageContent::FunctionCallRequest {
                                                id: id,
                                                name,
                                                args,
                                                _meta,
                                            },
                                            images: None,
                                        })
                                    }
                                    _ => {}
                                }
                            }

                            Some((Ok(yielded_item), (stream, should_terminate_stream, id)))
                        }
                        Err(e) => Some((Err(e), (stream, should_terminate_stream, id))),
                    }
                } else {
                    yielded_item.done = true;
                    should_terminate_stream = true;
                    Some((Ok(yielded_item), (stream, should_terminate_stream, id)))
                }
            },
        ))
    }
}

fn get_gemini_role(role: Role) -> String {
    match role {
        Role::User => String::from("user"),
        Role::Assistant => String::from("model"),
        Role::System => String::from("system"),
        Role::Function => String::from("model"),
    }
}

fn generate_gemini_part_meta_value(part_ref: &GeminiPart) -> Value {
    let mut inner_data = json!({});
    // Thought
    if let Some(thought) = &part_ref.thought {
        inner_data["thought"] = json!(*thought);
    }

    // Thought signature
    if let Some(thought_signature) = &part_ref.thought_signature {
        inner_data["thoughtSignature"] = json!(thought_signature.clone());
    }

    // Metadata
    if let Some(metadata) = &part_ref.metadata {
        inner_data["metadata"] = json!(metadata.clone());
    }

    // Part Metadata
    if let Some(part_metadata) = &part_ref.part_metadata {
        inner_data["partMetadata"] = part_metadata.clone();
    }

    let mut meta = json!({});
    meta[GEMINI_META] = inner_data;

    meta
}

fn get_gemini_meta_fields(
    meta: Option<Value>,
) -> (
    Option<bool>,
    Option<String>,
    Option<GeminiPartMetadata>,
    Option<Value>,
) {
    let mut thought = None;
    let mut thought_signature = None;
    let mut metadata = None;
    let mut part_metadata = None;

    if let Some(meta) = meta {
        // Thought
        if let Some(new_thought) = meta.pointer(&format!("/{GEMINI_META}/thought")) {
            if let Value::Bool(boolean_val) = new_thought {
                thought = Some(*boolean_val);
            }
        }
        // Thought signature
        if let Some(new_thought_signature) =
            meta.pointer(&format!("/{GEMINI_META}/thoughtSignature"))
        {
            if let Value::String(string_val) = new_thought_signature {
                thought_signature = Some(string_val.clone());
            }
        }
        // Metadata
        if let Some(new_metadata) = meta.pointer(&format!("/{GEMINI_META}/metadata")) {
            if let Ok(parsed_metadata) =
                serde_json::from_value::<GeminiPartMetadata>(new_metadata.clone())
            {
                metadata = Some(parsed_metadata);
            }
        }
        //Part metadata
        if let Some(new_part_metadata) = meta.pointer(&format!("/{GEMINI_META}/partMetadata")) {
            part_metadata = Some(new_part_metadata.clone());
        }
    }

    (thought, thought_signature, metadata, part_metadata)
}
