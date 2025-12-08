use crate::error::NexaError;
use futures::stream::{self, StreamExt};
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use serde_json::{self, json, Value};
use std::collections::HashMap;
use std::str::from_utf8;
use tauri_plugin_http::reqwest;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought_signature: Option<String>,

    #[serde(flatten)]
    pub data: GeminiPartData,

    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<GeminiPartMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_metadata: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum GeminiPartData {
    Text(String),

    #[serde(rename_all = "camelCase")]
    InlineData {
        mime_type: String,
        data: String,
        display_name: String,
    },

    #[serde(rename_all = "camelCase")]
    FileData {
        mime_type: String,
        file_uri: String,
        display_name: String,
    },

    FunctionCall {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<Value>,
    },

    #[serde(rename_all = "camelCase")]
    FunctionResponse {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        name: String,
        response: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        parts: Option<Vec<FunctionResponsePart>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        will_continue: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        scheduling: Option<Scheduling>,
    },

    ExecutableCode {
        language: Language,
        code: String,
    },

    CodeExecutionResult {
        outcome: CodeExecutionOutcome,
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum GeminiPartMetadata {
    #[serde(rename_all = "camelCase")]
    VideoMetadata {
        start_offset: String,
        end_offset: String,
        fps: f32,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum FunctionResponsePart {
    InlineData(FunctionResponseBlob),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FunctionResponseBlob {
    mime_type: String,
    data: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FunctionResponseFileData {
    mime_type: String,
    file_uri: String,
    display_name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Language {
    #[serde(rename = "PYTHON")]
    Python,
    #[serde(rename = "LANGUAGE_UNSPECIFIED")]
    LanguageUnspecified,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum CodeExecutionOutcome {
    #[serde(rename = "OUTCOME_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "OUTCOME_OK")]
    Ok,
    #[serde(rename = "OUTCOME_FAILED")]
    Failed,
    #[serde(rename = "OUTCOME_DEADLINE_EXCEEDED")]
    DeadlineExceeded,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Scheduling {
    #[serde(rename = "SCHEDULING_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "SILENT")]
    Silent,
    #[serde(rename = "WHEN_IDLE")]
    WhenIdle,
    #[serde(rename = "INTERRUPT")]
    Interrupt,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GeminiGenerateContentResponse {
    pub candidates: Vec<Candidate>,

    #[serde(flatten)]
    pub extra_fields: Value,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Candidate {
    pub content: Content,

    #[serde(flatten)]
    pub extra_fields: Value,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GeminiGenerateContentRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_config: Option<ToolConfig>,

    #[serde(flatten)]
    extra_fields: Value,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Content {
    pub parts: Vec<GeminiPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Tool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) function_declarations: Option<Vec<FunctionDeclaration>>,

    #[serde(flatten)]
    pub(crate) extra_fields: Value,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub(crate) struct FunctionDeclaration {
    pub(crate) name: String,
    pub(crate) description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parameters: Option<Value>,

    #[serde(flatten)]
    pub(crate) extra_fields: Value,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Schema {
    #[serde(rename = "type")]
    data_type: Type,

    #[serde(flatten)]
    type_specified_fields: Value,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Type {
    #[serde(rename = "TYPE_UNSPECIFIED")]
    TypeUnspecified,
    #[serde(rename = "STRING")]
    String,
    #[serde(rename = "NUMBER")]
    Number,
    #[serde(rename = "INTEGER")]
    Integer,
    #[serde(rename = "BOOLEAN")]
    Boolean,
    #[serde(rename = "ARRAY")]
    Array,
    #[serde(rename = "OBJECT")]
    Object,
    #[serde(rename = "NULL")]
    Null,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    function_calling_config: Option<FunctionCallingConfig>,

    #[serde(flatten)]
    extra_fields: Value,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCallingConfig {
    mode: FunctionCallingMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_function_names: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum FunctionCallingMode {
    #[serde(rename = "MODE_UNSPECIFIED")]
    ModeUnspecified,
    #[serde(rename = "AUTO")]
    Auto,
    #[serde(rename = "ANY")]
    Any,
    #[serde(rename = "NONE")]
    None,
    #[serde(rename = "VALIDATED")]
    Validated,
}

pub async fn gemini_chat(
    chat_history: Vec<Content>,
    tools: Vec<Tool>,
    model_id: String,
    api_key: String,
    tool_config: Option<ToolConfig>,
) -> Result<impl Stream<Item = Result<GeminiGenerateContentResponse, NexaError>>, NexaError> {
    let client = reqwest::Client::new();

    let gemini_request = GeminiGenerateContentRequest {
        contents: chat_history,
        tools: match tools.len() {
            0 => None,
            _ => Some(tools),
        },
        tool_config: tool_config,
        extra_fields: json!({}),
    };

    let response = client
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse",
            model_id
        ))
        .header("x-goog-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&gemini_request)
        .send()
        .await?;

    let result = response.error_for_status();

    if let Err(e) = result {
        return Err(NexaError::Reqwest(e));
    }

    let result = result.unwrap();
    let stream = result.bytes_stream();

    let stream = stream::unfold(stream, |mut stream| async move {
        if let Some(item) = stream.next().await {
            return match item {
                Ok(bytes) => {
                    let msg = from_utf8(&bytes).unwrap();
                    let trimed_msg = msg.trim().trim_start_matches("data: ");
                    if trimed_msg.is_empty() {
                        return None;
                    }

                    let res = serde_json::from_str::<GeminiGenerateContentResponse>(trimed_msg);
                    match res {
                        Ok(response) => Some((Ok(response), stream)),
                        Err(e) => {
                            dbg!(trimed_msg);
                            Some((Err(NexaError::SerdeJson(e)), stream))
                        }
                    }
                }
                Err(e) => Some((Err(NexaError::Reqwest(e)), stream)),
            };
        }
        None
    });

    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::pin_mut;
    // Import your structs from the parent module
    use serde_json::json; // Use the json! macro for easy Value creation
    use std::env;

    // Just a handy test for the Gemini API
    // #[tokio::test]
    async fn test_gemini_toolcall_response() {
        dotenv::dotenv().ok();

        let chat_history = vec![Content {
            parts: vec![GeminiPart {
                thought: None,
                thought_signature: None,
                data: GeminiPartData::Text("Schedule a meeting with Bob and Alice for 03/27/2025 at 10:00 AM about the Q3 planning.".to_string()),
                metadata: None,
                part_metadata: None,
            }],
            role: Some("user".to_string()),
        }];

        let schedule_meeting_properties = json!({
            "type": "object",
            "required": ["attendees", "date", "time", "topic"],
            "properties": {
                "attendees": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "List of people attending the meeting."
                },
                "date": {
                    "type": "string",
                    "description": "Date of the meeting (e.g., '2024-07-29')"
                },
                "time": {
                    "type": "string",
                    "description": "Time of the meeting (e.g., '15:00')"
                },
                "topic": {
                    "type": "string",
                    "description": "The subject or topic of the meeting."
                }
            }
        });

        let tools = vec![Tool {
            function_declarations: Some(vec![
                FunctionDeclaration {
                    name: "schedule_meeting".to_string(),
                    description:
                        "Schedules a meeting with specified attendees at a given time and date."
                            .to_string(),
                    parameters: Some(schedule_meeting_properties),
                    extra_fields: json!({}),
                },
                FunctionDeclaration {
                    name: "get_weather".to_string(),
                    description: "Get a current weather report for a given location.".to_string(),
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "location": {
                                "type": "string",
                                "description": "The location for the current weather report."
                            }
                        },
                        "required": ["location"]
                    })),
                    extra_fields: json!({}),
                },
            ]),
            extra_fields: json!({}),
        }];

        let stream = gemini_chat(
            chat_history,
            tools,
            "gemini-2.5-pro".to_string(),
            env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set for this test."),
            None,
        )
        .await
        .unwrap();

        pin_mut!(stream);

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(response) => {
                    dbg!(response);
                }
                Err(e) => {
                    dbg!("Error!!!!");
                    dbg!(e);
                }
            }
        }
    }

    #[test]
    fn test_gemini_part_serde() {
        test_function_call_part_serde();
        test_function_response_part_serde();
    }

    fn test_function_call_part_serde() {
        // 1. The input JSON string from your example
        let fake_signature = "CsMGAdHtim8x32E6rhrr86dUMLThOmH0MHzweDPuZh3CX0aTAS4vNR0KG369E0HYIniZW/2ZY4suy7JleM1ZAwYccBmdRv9UqVEOzOto+L7iINnuoocfj8aRjSa6rZRI+aL1MWLrXnAuWiwGLssNeos/eDRe7LV8DgPKgs59vf/31lXce7TpWBo+RLTXpdWnZv1U33WufGlhzBfmt07wLzkTx1QzZ7Bfw/fsKpT2njPGLaosmRH0Vs/ovknjGwh05x4tGls+39S4uum/GTE5vJcyYFr6gmjtvi+lAkQzafBIC9+3HpsKLYvEFPRvtxL/QoetFs/i12MgFJFkMGr01efvgkkVtkbFDM7cdqqFntbyy8R/MwvlR+3kKuXEa56WP9vFVCFfbO4dsFSQowau1nCg1T0e/kFBbNl6qZrkxw853xo4xe5ojXwn3v7xRCO/QA7SCCwo1zH4ARNW8k+9O71lXXIHEgLW+HxXir0MCMHJwwtHryMHXyK2moxEXuc2a47H4cYHnGcFi0ZF3p2s69U3aGDq0OgXBMp+Tm9othix53Dn5KMm5eSaDcUGMzXzCS2YoLQ7j+688ml4VMHL8REbpm+QFkbBOS5MP2usz2O5rb1Mbo1YHAoiNkYDjF2ybXz2exb+//N2Kj01gW66Kc7ljU2E6bpXgsu4rBzE7Pe2UrIgtlnBc/CgEW25bhzjN9STrMBoGLwEbfwvtIzfJ/BRaY7dRi04ICs5oB9LZsuB0Hs9msjfpgBBOvnI2bKRCpZc6T6qsI6gpqoyG+Y86nQETXQxADc6eA267SbyShfnzzDyCbUZZ5TSf9IWyVfJS0wUS5jW3MbsBh6qtj9O4/SecLdcVk/O+zZvUbsDsnGq3++c/AztQXf1bLs6JjgPCHCh1lKXULXjxWZpzqBVRR6ML4hsGEjvzEFrbDvT0Pok7CdyUMECNznHND+7OISno+Acl2rYM5JKiI6KiBHddRiAGXL/hRln9BcOo3zXTXbpQW7RoEsk8vw2bhpyI3LVNYATnAyVNjeUOx73cV81n4N7GVTZp/5hfULzw190BscOIqNG3TRfN9bV2kzO/pQALJUdWaQHEPyMeczp3ioTHs1B5OCcRQ==";

        let json_data = format!(
            r#"
        {{
          "functionCall": {{
            "id": "jazzcort1993",
            "name": "schedule_meeting",
            "args": {{
              "topic": "Q3 planning",
              "date": "2025-03-27",
              "time": "10:00",
              "attendees": [
                "Bob",
                "Alice"
              ]
            }}
          }},
          "thoughtSignature": "{}"
        }}
        "#,
            fake_signature
        );

        let expected_part = GeminiPart {
            thought: None,
            thought_signature: Some(fake_signature.to_string()),
            data: GeminiPartData::FunctionCall {
                id: Some("jazzcort1993".to_string()),
                name: "schedule_meeting".to_string(),
                args: Some(json!({
                    "topic": "Q3 planning",
                    "date": "2025-03-27",
                    "time": "10:00",
                    "attendees": ["Bob", "Alice"]
                })),
            },
            metadata: None,
            part_metadata: None,
        };

        // --- TEST 1: DESERIALIZATION (JSON -> RUST) ---

        let deserialized_part: GeminiPart =
            serde_json::from_str(&json_data).expect("Failed to deserialize JSON");

        // Compare the deserialized struct with our expected struct
        assert_eq!(deserialized_part, expected_part);

        // --- TEST 2: SERIALIZATION (RUST -> JSON) ---

        // Serialize our Rust struct to a serde_json::Value
        let serialized_value =
            serde_json::to_value(&expected_part).expect("Failed to serialize struct");

        // Parse the original JSON string to a serde_json::Value
        let original_json_value: serde_json::Value =
            serde_json::from_str(&json_data).expect("Failed to parse original JSON as Value");

        // Compare the two JSON Values. This is better than comparing strings
        // because it ignores key order and whitespace differences.
        assert_eq!(serialized_value, original_json_value);
    }

    fn test_function_response_part_serde() {
        // Test 1
        let json_data = r#"{
            "functionResponse": {
                "id": "12345",
                "name": "get_weather_response",
                "response": {
                  "location": "Boston",
                  "temperature": "40",
                  "weather": "sunny"
                },
                "parts": [
                    { 
                        "inlineData": { 
                            "mimeType": "image/png", 
                            "data": "000111000111" 
                        } 
                    }
                ],
                "willContinue": true,
                "scheduling": "SILENT"
            }
        }"#;

        let deserialized_part: GeminiPartData =
            serde_json::from_str(json_data).expect("Failed to deserialize JSON");

        let expected_response = json!({
            "location": "Boston",
            "temperature": "40",
            "weather": "sunny"
        });

        let expected_part = GeminiPartData::FunctionResponse {
            id: Some("12345".to_string()),
            name: "get_weather_response".to_string(),
            response: expected_response,
            parts: Some(vec![FunctionResponsePart::InlineData(
                FunctionResponseBlob {
                    mime_type: "image/png".to_string(),
                    data: "000111000111".to_string(),
                },
            )]),
            will_continue: Some(true),
            scheduling: Some(Scheduling::Silent),
        };

        assert_eq!(deserialized_part, expected_part);

        let serialized_value =
            serde_json::to_value(&expected_part).expect("Failed to serialize struct");

        let original_json_value: serde_json::Value =
            serde_json::from_str(&json_data).expect("Failed to parse original JSON to Value");

        assert_eq!(serialized_value, original_json_value);

        // Test 2
        let json_data = r#"{
            "functionResponse": {
                "name": "get_weather_response",
                "response": {
                  "location": "Boston",
                  "temperature": "40",
                  "weather": "sunny"
                }
            }
        }"#;

        let deserialized_part: GeminiPartData =
            serde_json::from_str(json_data).expect("Failed to deserialize JSON");

        let expected_response = json!({
            "location": "Boston",
            "temperature": "40",
            "weather": "sunny"
        });

        let expected_part = GeminiPartData::FunctionResponse {
            id: None,
            name: "get_weather_response".to_string(),
            response: expected_response,
            parts: None,
            will_continue: None,
            scheduling: None,
        };

        assert_eq!(deserialized_part, expected_part);

        let serialized_value =
            serde_json::to_value(&expected_part).expect("Failed to serialize struct");

        let original_json_value: serde_json::Value =
            serde_json::from_str(&json_data).expect("Failed to parse original JSON to Value");

        assert_eq!(serialized_value, original_json_value);
    }
}
