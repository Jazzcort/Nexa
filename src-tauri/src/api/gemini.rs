use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::collections::HashMap;
use tauri_plugin_http::reqwest;

pub struct GeminiChatHistory {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    thought: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    thought_signature: Option<String>,

    #[serde(flatten)]
    data: GeminiPartData,

    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<GeminiPartMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    part_metadata: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
        args: Option<HashMap<String, Value>>,
    },

    #[serde(rename_all = "camelCase")]
    FunctionResponse {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        name: String,
        response: HashMap<String, Value>,
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GeminiPartMetadata {
    #[serde(rename_all = "camelCase")]
    VideoMetadata {
        start_offset: String,
        end_offset: String,
        fps: f32,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum FunctionResponsePart {
    InlineData(FunctionResponseBlob),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Language {
    #[serde(rename = "PYTHON")]
    Python,
    #[serde(rename = "LANGUAGE_UNSPECIFIED")]
    LanguageUnspecified,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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

// fn gemini_chat(project_id: String, location: String, model_id: String, api_key: String) {
//     let client = reqwest::Client::new();
//     client.post(format!("https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent", location, project_id, location, model_id))
//         .header("Authorization", format!("Bearer {}", api_key))
//         .header("Content-Type", "application/json")
//         .json();
// }

// Put this at the bottom of your .rs file
#[cfg(test)]
mod tests {
    use super::*; // Import your structs from the parent module
    use serde_json::json; // Use the json! macro for easy Value creation

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

        // 2. Define the equivalent "ground truth" Rust struct
        let expected_args = HashMap::from([
            ("topic".to_string(), json!("Q3 planning")),
            ("date".to_string(), json!("2025-03-27")),
            ("time".to_string(), json!("10:00")),
            (
                "attendees".to_string(),
                json!(vec!["Bob".to_string(), "Alice".to_string()]),
            ),
        ]);

        let expected_part = GeminiPart {
            thought: None,
            thought_signature: Some(fake_signature.to_string()),
            data: GeminiPartData::FunctionCall {
                id: Some("jazzcort1993".to_string()),
                name: "schedule_meeting".to_string(),
                args: Some(expected_args),
            },
            metadata: None,
            part_metadata: None,
        };
        // NOTE: You MUST copy the full, long thoughtSignature string into the line above.

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

        let expected_response = HashMap::from([
            ("location".to_string(), json!("Boston")),
            ("temperature".to_string(), json!("40")),
            ("weather".to_string(), json!("sunny")),
        ]);

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

        let expected_response = HashMap::from([
            ("location".to_string(), json!("Boston")),
            ("temperature".to_string(), json!("40")),
            ("weather".to_string(), json!("sunny")),
        ]);

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
