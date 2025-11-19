use crate::api::gemini::gemini_chat;
use crate::api::gemini::{Content, GeminiPart, GeminiPartData, Tool, ToolConfig};
use crate::error::NexaError;
use crate::llm::base::{ChatHistory, ChatMessage, EmittedChatMessage, Role, LLM};
use futures::stream;
use futures::StreamExt;
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
    ) -> Result<impl Stream<Item = EmittedChatMessage>, NexaError> {
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
        let response = gemini_chat(
            contents,
            self.tools.clone(),
            self.model_id.clone(),
            self.api_key.clone(),
            self.tool_config.clone(),
        )
        .await?;

        // let response = task::spawn_blocking(async {
        //     return gemini_chat(
        //         contents,
        //         self.tools.clone(),
        //         self.model_id.clone(),
        //         self.api_key.clone(),
        //         self.tool_config.clone(),
        //     )
        //     .await;
        // });
        //
        dbg!(&response);

        if response.candidates.len() < 1 {
            return Err(NexaError::Gemini(
                "No candidate in the response".to_string(),
            ));
        }

        let first_candidate = response.candidates.first().clone().unwrap();

        let mut text_output = String::new();
        let parts = first_candidate.content.parts.iter();
        for part in parts {
            match &part.data {
                GeminiPartData::Text(text) => text_output += text,
                (_) => {}
            }
        }

        let mut text_output_vec: Vec<char> = text_output.chars().collect();

        let initial_state: usize = 0;

        Ok(stream::unfold(
            (initial_state, text_output_vec, id),
            |(mut start, text_output_vec, id)| async move {
                sleep(Duration::from_millis(50)).await;
                if start >= text_output_vec.len() {
                    return None;
                }

                let end = (start + TEXT_OUTPUT_STEPS).min(text_output_vec.len());
                let chunk = &text_output_vec[start..end];

                let yielded_item = EmittedChatMessage {
                    id: id.clone(),
                    message: ChatMessage {
                        role: Role::Assistant,
                        content: chunk.iter().collect(),
                        images: None,
                    },
                    done: if end == text_output_vec.len() {
                        true
                    } else {
                        false
                    },
                };
                start = end;
                Some((yielded_item, (start, text_output_vec, id)))
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
