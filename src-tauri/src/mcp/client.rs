use crate::{
    error::NexaError,
    mcp::{
        connection::{
            self, mcp_stdio_connect, MCPStdioConnection, MCPTransportReader, MCPTransportWriter,
        },
        structs::{Id, MCPDataPacket, MCPNotification, MCPRequest, MCPResponse, JSON_RPC},
    },
};
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};
use tokio::{
    select,
    sync::{oneshot, RwLock},
    task,
};
use tokio_util::sync::CancellationToken;

use serde_json::{json, Value};

use crate::mcp::connection::MCPConnection;

pub(crate) static MCP_PROTOCOL_VERSION: &str = "2025-06-18";
pub(crate) static MCP_CLIENT_NAME: &str = "Nexa MCP Client";
pub(crate) static MCP_CLIENT_VERSION: &str = "1.0.0";

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SimpleCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    list_changed: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResourcesCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    list_changed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subscribe: Option<bool>,
}

#[derive(Serialize, Clone, PartialEq, Debug, Default)]
pub(crate) struct ClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    elicitation: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    experimental: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    roots: Option<SimpleCapability>,

    #[serde(skip_serializing_if = "Option::is_none")]
    sampling: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct Implementation {
    name: String,
    version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClientConfiguration {
    capabilities: ClientCapabilities,
    client_info: Implementation,
    protocol_version: String,
}
impl Default for ClientConfiguration {
    fn default() -> Self {
        Self {
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: MCP_CLIENT_NAME.to_string(),
                title: Some(MCP_CLIENT_NAME.to_string()),
                version: MCP_CLIENT_VERSION.to_string(),
            },
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
        }
    }
}

#[derive(Deserialize, Clone, PartialEq, Debug, Default)]
pub(crate) struct ServerCapabilities {
    completions: Option<Value>,
    experimental: Option<Value>,
    logging: Option<Value>,
    prompts: Option<SimpleCapability>,
    resources: Option<ResourcesCapability>,
    tools: Option<SimpleCapability>,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ServerConfiguration {
    #[serde(rename = "_meta")]
    _meta: Option<Value>,
    capabilities: ServerCapabilities,
    instructions: Option<String>,
    protocol_version: String,
    server_info: Implementation,

    #[serde(flatten)]
    extra_fields: Value,
}
impl Default for ServerConfiguration {
    fn default() -> Self {
        Self {
            _meta: None,
            capabilities: ServerCapabilities::default(),
            instructions: None,
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            server_info: Implementation {
                name: String::from("Default Server"),
                version: String::from("1.0.0"),
                title: None,
            },

            extra_fields: json!({}),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum MCPStatus {
    Connected,
    Disconnected,
    Connecting,
}

pub(crate) enum Transport {
    Stdio,
    Http,
}

pub(crate) struct MCPClient {
    configuration: ClientConfiguration,
    transport_output: Option<Box<dyn MCPTransportReader>>,
    transport_input: Box<dyn MCPTransportWriter>,
    transport_type: Transport,
    tool_calls_map: Arc<Mutex<HashMap<Id, oneshot::Sender<MCPResponse>>>>,
    request_id: Mutex<u64>,
    status: MCPStatus,

    server_config: ServerConfiguration,
    cancel_token: CancellationToken,
}

impl MCPClient {
    pub fn new_stdio_client<S, I>(command: S, args: I) -> Result<Self, NexaError>
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        let (stdio_writer, stdio_reader) = mcp_stdio_connect(command, args)?;

        Ok(MCPClient {
            configuration: ClientConfiguration::default(),
            status: MCPStatus::Connecting,
            transport_output: Some(Box::new(stdio_reader)),
            transport_input: Box::new(stdio_writer),
            transport_type: Transport::Stdio,
            request_id: Mutex::new(1),

            tool_calls_map: Arc::new(Mutex::new(HashMap::new())),

            server_config: ServerConfiguration::default(),
            cancel_token: CancellationToken::new(),
        })
    }

    async fn get_request_id(&self) -> u64 {
        let mut handle = self.request_id.lock().await;
        let id = *handle;
        *handle += 1;
        id
    }

    async fn initialize(&mut self) -> Result<(), NexaError> {
        match &self.transport_type {
            Transport::Stdio => {
                let default_client_config = self.configuration.clone();
                let default_client_config_value = serde_json::to_value(default_client_config)?;
                let request_id = Id::NumberId(self.get_request_id().await);

                let initialize_request = MCPRequest {
                    jsonrpc: JSON_RPC.to_string(),
                    id: request_id.clone(),
                    method: String::from("initialize"),
                    params: Some(default_client_config_value),
                };

                self.transport_input
                    .send(serde_json::to_value(initialize_request)?)
                    .await?;
                let response = self
                    .transport_output
                    .as_mut()
                    .ok_or(NexaError::MCPConnection(String::from("Missing Stdout")))?
                    .receive()
                    .await?;

                if let MCPDataPacket::Response(initialize_response) = response {
                    match initialize_response {
                        MCPResponse::Success {
                            jsonrpc,
                            id,
                            result,
                        } => {
                            if jsonrpc != JSON_RPC {
                                return Err(NexaError::MCPConnection(String::from(
                                    "Incorrect JSON RPC version",
                                )));
                            }

                            if id != request_id {
                                return Err(NexaError::MCPConnection(String::from(
                                    "Incorrect response id",
                                )));
                            }

                            let server_config: ServerConfiguration =
                                serde_json::from_value(result)?;
                            self.server_config = server_config;
                            self.status = MCPStatus::Connected;

                            let initialized_notification = MCPNotification {
                                jsonrpc: JSON_RPC.to_string(),
                                method: String::from("notifications/initialized"),
                                params: None,
                            };

                            self.transport_input
                                .send(serde_json::to_value(initialized_notification)?)
                                .await?;

                            return Ok(());
                        }
                        MCPResponse::Fail { jsonrpc, id, error } => {
                            return Err(NexaError::MCPConnection(format!(
                                "Initialization Failed: {}",
                                error.message
                            )))
                        }
                    }
                }

                Err(NexaError::MCPConnection(String::from(
                    "Incorrect server response during initialization phase",
                )))
            }
            Transport::Http => Err(NexaError::MCPConnection(String::from("Unimplemented"))),
        }
    }

    async fn list_tools(&mut self) -> Result<(), NexaError> {
        match &self.transport_type {
            Transport::Stdio => {
                let tools_list_request = MCPRequest {
                    jsonrpc: JSON_RPC.to_string(),
                    id: Id::NumberId(self.get_request_id().await),
                    method: String::from("tools/list"),
                    params: None,
                };

                let _ = self
                    .transport_input
                    .send(serde_json::to_value(tools_list_request)?)
                    .await?;

                let response = self
                    .transport_output
                    .as_mut()
                    .ok_or(NexaError::MCPConnection(String::from("Missing Stdout")))?
                    .receive()
                    .await?;

                if let MCPDataPacket::Response(tools_list_response) = response {
                    match tools_list_response {
                        MCPResponse::Success {
                            jsonrpc,
                            id,
                            result,
                        } => return Ok(()),
                        MCPResponse::Fail { jsonrpc, id, error } => {
                            return Err(NexaError::MCPConnection(String::from(format!(
                                "Tools List Failed: {}",
                                error.message
                            ))))
                        }
                    }
                }

                Err(NexaError::MCPConnection(String::from(
                    "Incorrect server response during initialization phase",
                )))
            }
            Transport::Http => Err(NexaError::MCPConnection(String::from("Unimplemented"))),
        }
    }

    pub async fn start_listening(&mut self) -> Result<(), NexaError> {
        self.initialize().await?;

        let mut stdout_handle = self
            .transport_output
            .take()
            .ok_or(NexaError::MCPConnection(String::from("Missing Stdout")))?;
        let cancel_token = self.cancel_token.clone();
        let tool_calls_map = self.tool_calls_map.clone();

        task::spawn(async move {
            loop {
                select! {
                    res = cancel_token.cancelled() => {break},
                    result = stdout_handle.receive() => {
                        match result {
                            Ok(data_packet) => {
                                match data_packet {
                                    MCPDataPacket::Response(response) => {
                                        let mut map = tool_calls_map.lock().await;

                                        match &response {
                                            MCPResponse::Success{jsonrpc, id, result} => {
                                                if let Some(response_pipe) = map.remove(id) {
                                                    let _ = response_pipe.send(response);
                                                }
                                            }
                                            MCPResponse::Fail{jsonrpc, id, error} => {
                                                if let Some(response_pipe) = map.remove(id) {
                                                    let _ = response_pipe.send(response);
                                                }
                                            }
                                        }
                                    }
                                    MCPDataPacket::Request(request) => {}
                                    MCPDataPacket::Notification(notification) => {}
                                }
                            }
                            Err(e) => {
                                break
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    // pub async fn call_tool()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn client_test() {
        let mut client = MCPClient::new_stdio_client(
            "uvx",
            [
                "--env-file",
                "/Users/chihlee/temp/.env",
                "--from",
                "git+https://github.com/Jazzcort/status-report-assistant-mcp",
                "mcp-serve",
            ],
        )
        .unwrap();

        assert_eq!(client.status, MCPStatus::Connecting);

        let _ = client.initialize().await;

        assert_eq!(client.status, MCPStatus::Connected);
        dbg!(&client.server_config);
    }
}
