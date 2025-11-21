use crate::api::gemini::gemini_chat;
use crate::api::gemini::{Content, GeminiPart, GeminiPartData, Tool, ToolConfig};
use crate::error::NexaError;
use crate::llm::base::{ChatHistory, ChatMessage, EmittedChatMessage, Role, LLM};
use futures::stream;
use futures::StreamExt;
use futures_util::Stream;

pub struct Gemini {
    pub model_id: String,
    pub tools: Vec<Tool>,
    pub api_key: String,
    pub tool_config: Option<ToolConfig>,
}

impl LLM for Gemini {
    async fn stream_chat(
        &self,
        history: ChatHistory,
    ) -> Result<impl Stream<Item = Result<EmittedChatMessage, NexaError>>, NexaError> {
        let last_message = history
            .messages
            .last()
            .ok_or(NexaError::Gemini("Empty chat history".to_string()))?;
        let id = last_message.id.clone();

        let contents: Vec<Content> = history
            .messages
            .iter()
            .map(|message| Content {
                parts: vec![GeminiPart {
                    thought: None,
                    thought_signature: None,
                    metadata: None,
                    part_metadata: None,
                    data: GeminiPartData::Text(message.content.clone()),
                }],
                role: Some(get_gemini_role(message.role.clone())),
            })
            .collect();

        let stream = gemini_chat(
            contents,
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
                    message: ChatMessage {
                        role: Role::Assistant,
                        content: String::new(),
                        images: None,
                    },
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

                            let mut text_output = String::new();
                            for part in first_candidate.content.parts {
                                match part.data {
                                    GeminiPartData::Text(msg) => text_output += &msg,
                                    _ => {}
                                }
                            }

                            yielded_item.message.content = text_output;

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
    }
}
