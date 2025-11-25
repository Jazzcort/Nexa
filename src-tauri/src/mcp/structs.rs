use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(crate) static JSON_RPC: &str = "2.0";

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub(crate) enum Id {
    NumberId(u64),
    StringId(String),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct MCPRequest {
    jsonrpc: String,
    id: Id,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct MCPError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub(crate) enum MCPResponse {
    Success {
        jsonrpc: String,
        id: Id,
        result: HashMap<String, Value>,
    },
    Fail {
        jsonrpc: String,
        id: Id,
        error: MCPError,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct MCPNotification {
    jsonrpc: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<HashMap<String, Value>>,
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
            params: Some(HashMap::from([
                (String::from("requestId"), json!(String::from("234"))),
                (
                    String::from("reason"),
                    json!(String::from("User requested cancellation")),
                ),
            ])),
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
            result: HashMap::from([
                (String::from("temperature"), json!(String::from("33"))),
                (
                    String::from("area"),
                    json!(vec![
                        String::from("Cambridge"),
                        String::from("Allston"),
                        String::from("Medford"),
                    ]),
                ),
            ]),
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
                data: Some(json!(HashMap::from([(
                    String::from("arguments"),
                    HashMap::from([(
                        String::from("locations"),
                        vec![String::from("Boston"), String::from("New York")]
                    )])
                )]))),
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
            params: Some(HashMap::from([(
                String::from("data"),
                json!(String::from("Required data")),
            )])),
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
