use crate::api::gemini::gemini_chat;
use crate::api::gemini::{Content, GeminiPart, GeminiPartData, Tool, ToolConfig};
use crate::error::NexaError;
use crate::llm::base::{ChatHistory, ChatMessage, EmittedChatMessage, Role, LLM};
use futures::StreamExt;
use futures::{pin_mut, stream};
use futures_util::Stream;
use tokio::time::{sleep, Duration};

static TEXT_OUTPUT_STEPS: usize = 10;

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

        let mut stream = gemini_chat(
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

        // if response.candidates.len() < 1 {
        //     return Err(NexaError::Gemini(
        //         "No candidate in the response".to_string(),
        //     ));
        // }
        //
        // let first_candidate = response.candidates.first().clone().unwrap();
        //
        // let mut text_output = String::new();
        // let parts = first_candidate.content.parts.iter();
        // for part in parts {
        //     match &part.data {
        //         GeminiPartData::Text(text) => text_output += text,
        //         (_) => {}
        //     }
        // }
        //
        // let mut text_output_vec: Vec<char> = text_output.chars().collect();
        // let initial_state: usize = 0;
        // let should_terminate_stream = false;
        //
        // Ok(stream::unfold(
        //     (initial_state, text_output_vec, id, should_terminate_stream),
        //     |(mut start, text_output_vec, id, mut should_terminate_stream)| async move {
        //         // Simulate stream generation
        //         sleep(Duration::from_millis(50)).await;
        //
        //         // Default item
        //         let mut yielded_item = EmittedChatMessage {
        //             id: id.clone(),
        //             message: ChatMessage {
        //                 role: Role::Assistant,
        //                 content: String::new(),
        //                 images: None,
        //             },
        //             done: false,
        //         };
        //
        //         // Terminate the stream
        //         if should_terminate_stream {
        //             return None;
        //         }
        //
        //         // Hit the end of the test ouput
        //         if start >= text_output_vec.len() {
        //             yielded_item.done = true;
        //             should_terminate_stream = true;
        //
        //             return Some((
        //                 yielded_item,
        //                 (start, text_output_vec, id, should_terminate_stream),
        //             ));
        //         }
        //
        //         // Streaming
        //         let end = (start + TEXT_OUTPUT_STEPS).min(text_output_vec.len());
        //         let chunk = &text_output_vec[start..end];
        //         yielded_item.message.content = chunk.iter().collect();
        //         start = end;
        //         Some((
        //             yielded_item,
        //             (start, text_output_vec, id, should_terminate_stream),
        //         ))
        //     },
        // ))
    }
}

fn get_gemini_role(role: Role) -> String {
    match role {
        Role::User => String::from("user"),
        Role::Assistant => String::from("model"),
        Role::System => String::from("system"),
    }
}
