use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(crate) static JSON_RPC: &str = "2.0";

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
#[serde(untagged)]
pub(crate) enum Id {
    NumberId(u64),
    StringId(String),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub(crate) enum MCPDataPacket {
    Request(MCPRequest),
    Response(MCPResponse),
    Notification(MCPNotification),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct MCPRequest {
    pub(crate) jsonrpc: String,
    pub(crate) id: Id,
    pub(crate) method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) params: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct MCPError {
    pub(crate) code: i32,
    pub(crate) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub(crate) enum MCPResponse {
    Success {
        jsonrpc: String,
        id: Id,
        result: Value,
    },
    Fail {
        jsonrpc: String,
        id: Id,
        error: MCPError,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EmittedMCPResponse {
    pub(crate) request_id: String,
    pub(crate) response_id: String,

    pub(crate) response: Value,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct MCPNotification {
    pub(crate) jsonrpc: String,
    pub(crate) method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) params: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) enum Role {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct Annotations {
    audience: Option<Vec<Role>>,
    last_modified: Option<String>,
    priority: Option<f64>,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct TextContent {
    #[serde(rename = "_meta")]
    _meta: Option<Value>,
    annotations: Option<Annotations>,
    text: String,
    #[serde(rename = "type")]
    data_type: String,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub(crate) enum ContentBlock {
    Text(TextContent),
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ToolAnnotations {
    destructive_hint: Option<bool>,
    idempotent_hint: Option<bool>,
    open_world_hint: Option<bool>,
    read_only_hint: Option<bool>,
    title: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub(crate) struct FunctionSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<Vec<String>>,
    #[serde(rename = "type")]
    data_type: String,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Tool {
    #[serde(rename = "_meta")]
    pub(crate) _meta: Option<Value>,
    pub(crate) annotations: Option<ToolAnnotations>,
    pub(crate) description: Option<String>,
    pub(crate) input_schema: FunctionSchema,
    pub(crate) name: String,
    pub(crate) output_schema: Option<FunctionSchema>,
    pub(crate) title: Option<String>,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListToolsResult {
    #[serde(rename = "_meta")]
    pub(crate) _meta: Option<Value>,
    pub(crate) next_cursor: Option<String>,
    pub(crate) tools: Vec<Tool>,

    #[serde(flatten)]
    pub(crate) extra_fields: Value,
}

#[cfg(test)]
mod tests {
    use super::*; // Import your structs from the parent module
    use serde_json::{self, json};

    #[test]
    fn mcp_notification_parsing_test() {
        // Without params
        let raw_json_string = r#"{
            "jsonrpc": "2.0",
            "method": "notifications/tools/list_changed"
        }"#;

        let expected_mcp_notification = MCPNotification {
            jsonrpc: JSON_RPC.to_string(),
            method: String::from("notifications/tools/list_changed"),
            params: None,
        };

        let deserialized_notification: MCPNotification =
            serde_json::from_str(raw_json_string).unwrap();
        assert_eq!(expected_mcp_notification, deserialized_notification);

        let expected_json_value: Value = serde_json::from_str(raw_json_string).unwrap();
        let serialized_value = serde_json::to_value(expected_mcp_notification).unwrap();
        assert_eq!(expected_json_value, serialized_value);

        // With params
        let raw_json_string = r#"{
            "jsonrpc": "2.0",
            "method": "notifications/cancelled",
            "params": {
                "requestId": "234",
                "reason": "User requested cancellation"
            }
        }"#;

        let expected_mcp_notification = MCPNotification {
            jsonrpc: JSON_RPC.to_string(),
            method: String::from("notifications/cancelled"),
            params: Some(json!({
                "requestId": "234",
                "reason": "User requested cancellation"
            })),
        };

        let deserialized_notification: MCPNotification =
            serde_json::from_str(raw_json_string).unwrap();
        assert_eq!(expected_mcp_notification, deserialized_notification);

        let expected_json_value: Value = serde_json::from_str(raw_json_string).unwrap();
        let serialized_value = serde_json::to_value(expected_mcp_notification).unwrap();
        assert_eq!(expected_json_value, serialized_value);
    }

    #[test]
    fn mcp_response_parsing_test() {
        // Success response
        let raw_json_string = r#"{
            "jsonrpc": "2.0",
            "id": 1234,
            "result": {
                "temperature": "33",
                "area": [
                    "Cambridge", "Allston", "Medford"
                ]
            }
        }"#;

        let expected_mcp_response = MCPResponse::Success {
            jsonrpc: JSON_RPC.to_string(),
            id: Id::NumberId(1234),
            result: json!({
                "temperature": "33",
                "area": [
                    "Cambridge", "Allston", "Medford"
                ]
            }),
        };

        let deserialized_response: MCPResponse = serde_json::from_str(raw_json_string).unwrap();
        assert_eq!(expected_mcp_response, deserialized_response);

        let expected_json_value: Value = serde_json::from_str(raw_json_string).unwrap();
        let serialized_value = serde_json::to_value(expected_mcp_response).unwrap();
        assert_eq!(expected_json_value, serialized_value);

        // Failed response
        let raw_json_string = r#"{
            "jsonrpc": "2.0",
            "id": "WEATHER-1234",
            "error": {
                "code": -32700,
                "message": "Parsing error",
                "data": {
                    "arguments": {
                        "locations": ["Boston", "New York"]
                    }
                }
            }
        }"#;

        let expected_mcp_response = MCPResponse::Fail {
            jsonrpc: JSON_RPC.to_string(),
            id: Id::StringId(String::from("WEATHER-1234")),
            error: MCPError {
                code: -32700,
                message: String::from("Parsing error"),
                data: Some(json!({
                    "arguments": {
                        "locations": ["Boston", "New York"]
                    }
                }
                )),
            },
        };

        let deserialized_response: MCPResponse = serde_json::from_str(raw_json_string).unwrap();
        assert_eq!(expected_mcp_response, deserialized_response);

        let expected_json_value: Value = serde_json::from_str(raw_json_string).unwrap();
        let serialized_value = serde_json::to_value(expected_mcp_response).unwrap();
        assert_eq!(expected_json_value, serialized_value);
    }

    #[test]
    fn mcp_request_parsing_test() {
        // Test 1
        let raw_json_string = r#"{
                "jsonrpc": "2.0",
                "id": "23",
                "method": "tool/list"
            }"#;

        let expected_mcp_request = MCPRequest {
            jsonrpc: JSON_RPC.to_string(),
            id: Id::StringId(String::from("23")),
            method: String::from("tool/list"),
            params: None,
        };

        let deserialized_request: MCPRequest = serde_json::from_str(raw_json_string).unwrap();
        assert_eq!(expected_mcp_request, deserialized_request);

        let expected_json_value: Value = serde_json::from_str(raw_json_string).unwrap();
        let serialized_value = serde_json::to_value(expected_mcp_request).unwrap();
        assert_eq!(expected_json_value, serialized_value);

        // Test 2
        let raw_json_string = r#"{
                "jsonrpc": "2.0",
                "id": 14233,
                "method": "tool/call",
                "params": {
                    "data": "Required data"
                }
            }"#;

        let expected_mcp_request = MCPRequest {
            jsonrpc: JSON_RPC.to_string(),
            id: Id::NumberId(14233),
            method: String::from("tool/call"),
            params: Some(json!({
                "data": "Required data"
            })),
        };

        let serialized_request: MCPRequest = serde_json::from_str(raw_json_string).unwrap();
        assert_eq!(expected_mcp_request, serialized_request);

        let expected_json_value: Value = serde_json::from_str(raw_json_string).unwrap();
        let deserialized_value = serde_json::to_value(expected_mcp_request).unwrap();
        assert_eq!(expected_json_value, deserialized_value);
    }

    #[test]
    fn id_parsing_test() {
        // Number Id
        let raw_id = 12352333;
        let parsed_id: Id = serde_json::from_value(json!(raw_id)).unwrap();
        assert_eq!(parsed_id, Id::NumberId(raw_id));

        // String Id
        let raw_id = "5463878";
        let parsed_id: Id = serde_json::from_value(json!(raw_id)).unwrap();
        dbg!(&parsed_id);
        assert_eq!(parsed_id, Id::StringId(raw_id.to_string()));
    }
}
