use crate::{
    error::NexaError,
    mcp::{
        connection::{
            self, mcp_stdio_connect, MCPStdioConnection, MCPTransportReader, MCPTransportWriter,
        },
        structs::{
            Id, ListToolsResult, MCPDataPacket, MCPNotification, MCPRequest, MCPResponse, Tool,
            JSON_RPC,
        },
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
    pub(crate) name: String,
    pub(crate) version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) title: Option<String>,
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
    pub(crate) completions: Option<Value>,
    pub(crate) experimental: Option<Value>,
    pub(crate) logging: Option<Value>,
    pub(crate) prompts: Option<SimpleCapability>,
    pub(crate) resources: Option<ResourcesCapability>,
    pub(crate) tools: Option<SimpleCapability>,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ServerConfiguration {
    #[serde(rename = "_meta")]
    pub(crate) _meta: Option<Value>,
    pub(crate) capabilities: ServerCapabilities,
    pub(crate) instructions: Option<String>,
    pub(crate) protocol_version: String,
    pub(crate) server_info: Implementation,

    #[serde(flatten)]
    pub(crate) extra_fields: Value,
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
    transport_output: Mutex<Option<Box<dyn MCPTransportReader>>>,
    transport_input: Box<dyn MCPTransportWriter>,
    transport_type: Transport,
    tool_list: RwLock<HashMap<String, Tool>>,
    tool_calls_map: Arc<Mutex<HashMap<Id, oneshot::Sender<MCPResponse>>>>,
    request_id: Mutex<u64>,
    status: RwLock<MCPStatus>,

    server_config: RwLock<ServerConfiguration>,
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
            status: RwLock::new(MCPStatus::Connecting),
            transport_output: Mutex::new(Some(Box::new(stdio_reader))),
            transport_input: Box::new(stdio_writer),
            transport_type: Transport::Stdio,
            request_id: Mutex::new(1),

            tool_list: RwLock::new(HashMap::new()),
            tool_calls_map: Arc::new(Mutex::new(HashMap::new())),

            server_config: RwLock::new(ServerConfiguration::default()),
            // Cancel token for listening async task
            cancel_token: CancellationToken::new(),
        })
    }

    pub async fn get_server_config(&self) -> ServerConfiguration {
        return self.server_config.read().await.clone();
    }

    pub async fn get_tool_list(&self) -> Vec<(String, Tool)> {
        return self
            .tool_list
            .read()
            .await
            .iter()
            .map(|(k, v)| return (k.clone(), v.clone()))
            .collect();
    }

    async fn get_request_id(&self) -> u64 {
        let mut handle = self.request_id.lock().await;
        let id = *handle;
        *handle += 1;
        id
    }

    async fn initialize(&self) -> Result<(), NexaError> {
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
                    .lock()
                    .await
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

                            // Update Server Configuration
                            let server_config_received: ServerConfiguration =
                                serde_json::from_value(result)?;
                            let mut server_config_handle = self.server_config.write().await;
                            *server_config_handle = server_config_received;

                            // Update Status
                            let mut status_handle = self.status.write().await;
                            *status_handle = MCPStatus::Connected;

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

    async fn list_tools(&self) -> Result<(), NexaError> {
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
                    .lock()
                    .await
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
                        } => {
                            let tools_list_result: ListToolsResult =
                                serde_json::from_value(result)?;
                            let mut tool_list_handle = self.tool_list.write().await;
                            for tool in tools_list_result.tools.iter() {
                                tool_list_handle.insert(tool.name.clone(), tool.clone());
                            }
                            return Ok(());
                        }
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

    pub async fn start_listening(&self) -> Result<(), NexaError> {
        self.initialize().await?;
        self.list_tools().await?;

        let mut stdout_handle = self
            .transport_output
            .lock()
            .await
            .take()
            .ok_or(NexaError::MCPConnection(String::from("Missing Stdout")))?;
        let cancel_token = self.cancel_token.clone();
        let tool_calls_map = self.tool_calls_map.clone();

        task::spawn(async move {
            loop {
                select! {
                    res = cancel_token.cancelled() => {
                        dbg!("Canceled!!");
                        break;
                    },
                    result = stdout_handle.receive() => {
                        match result {
                            Ok(data_packet) => {
                                match data_packet {
                                    MCPDataPacket::Response(response) => {
                                        let mut map = tool_calls_map.lock().await;

                                        match &response {
                                            MCPResponse::Success{jsonrpc, id, result} => {
                                                dbg!(&id);
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

    pub async fn call_tool(
        &self,
        name: impl Into<String>,
        arguments: Value,
    ) -> Result<oneshot::Receiver<MCPResponse>, NexaError> {
        if !arguments.is_object() {
            return Err(NexaError::MCPToolCall(String::from(
                "Arguments should be an object",
            )));
        }

        let id = Id::NumberId(self.get_request_id().await);
        let (tx, tr) = oneshot::channel::<MCPResponse>();

        let mut tool_calls_map_handle = self.tool_calls_map.lock().await;
        tool_calls_map_handle.insert(id.clone(), tx);

        let name: String = name.into();
        let call_tool_request = MCPRequest {
            jsonrpc: JSON_RPC.to_string(),
            id: id.clone(),
            method: String::from("tools/call"),
            params: Some(match arguments.as_object().unwrap().is_empty() {
                true => {
                    json!({
                        "name": name,
                    })
                }
                false => {
                    json!({
                        "name": name,
                        "arguments": arguments
                    })
                }
            }),
        };

        self.transport_input
            .send(serde_json::to_value(call_tool_request)?)
            .await?;

        Ok(tr)
    }
}

impl Drop for MCPClient {
    fn drop(&mut self) {
        self.cancel_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use tokio::time::{sleep, Duration};

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

        assert_eq!(*client.status.read().await, MCPStatus::Connecting);

        let _ = client.start_listening().await;

        assert_eq!(*client.status.read().await, MCPStatus::Connected);

        dbg!(&client.server_config);
        dbg!(&client.tool_list);

        let output_channel_1 = client
            .call_tool(
                "get_github_summary",
                json!({
                    "after": "2025-11-15",
                    "before": "2025-12-04",
                    "github_username": "Jazzcort"
                }),
            )
            .await
            .unwrap();

        let output_channel_2 = client
            .call_tool(
                "get_github_summary",
                json!({
                    "after": "2025-11-15",
                    "before": "2025-12-04",
                    "github_username": "Jazzcort"
                }),
            )
            .await
            .unwrap();

        let output_channel_3 = client
            .call_tool(
                "get_github_summary",
                json!({
                    "after": "2025-11-15",
                    "before": "2025-12-04",
                    "github_username": "Jazzcort"
                }),
            )
            .await
            .unwrap();
        let output_channel_4 = client
            .call_tool(
                "get_github_summary",
                json!({
                    "after": "2025-11-15",
                    "before": "2025-12-04",
                    "github_username": "Jazzcort"
                }),
            )
            .await
            .unwrap();
        let output_channel_5 = client
            .call_tool(
                "create_draft_email",
                json!({
                    "content": "testing",
                    "subject": "MCP client testing",
                    "to": ["jason101011113@gmail.com"]
                }),
            )
            .await
            .unwrap();
        let output_channel_6 = client
            .call_tool("get_root_directory", json!({}))
            .await
            .unwrap();
        let output_channel_7 = client
            .call_tool("get_root_directory", json!({}))
            .await
            .unwrap();

        let start = Instant::now();

        let res = tokio::join!(
            output_channel_1,
            output_channel_2,
            output_channel_3,
            output_channel_4,
            output_channel_5,
            output_channel_6,
            output_channel_7
        );

        dbg!(start.elapsed());
        dbg!(res);

        // let a = select! {
        //     _ = sleep(Duration::from_secs(10)) => {
        //         dbg!("timeout!");
        //     }
        //     res = output_channel => {
        //         match res {
        //             Ok(tool_call_res) => {
        //                 dbg!(tool_call_res);
        //             }
        //             Err(e) => {
        //                 dbg!(e);
        //             }
        //         }
        //     }
        // }

        drop(client);

        sleep(Duration::from_secs(5)).await;
    }
}
