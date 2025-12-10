use std::sync::Arc;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::RwLock;

use crate::{
    error::NexaError,
    mcp::{client::MCPClient, structs::EmittedMCPResponse},
    AppData,
};

#[tauri::command]
pub async fn initialize_mcp_client(state: State<'_, AppData>) -> Result<(), NexaError> {
    let mut client = MCPClient::new_stdio_client(
        "uvx",
        [
            "--env-file",
            "/Users/chihlee/temp/.env",
            "--from",
            "git+https://github.com/Jazzcort/status-report-assistant-mcp",
            "mcp-serve",
        ],
    )?;

    let _ = client.start_listening().await;
    let server_config = client.get_server_config().await;

    let mut mcp_clients_map = state.mcp_clients.write().await;

    mcp_clients_map.insert(server_config.server_info.name, Arc::new(client));

    dbg!("successfully initialized mcps");

    Ok(())
}

#[tauri::command]
pub async fn call_tool(
    app: AppHandle,
    state: State<'_, AppData>,
    server_name: String,
    function_name: String,
    request_id: String,
    response_id: String,
    arguments: Value,
) -> Result<(), NexaError> {
    dbg!("called!");

    dbg!(&arguments);

    let mcp_client_map = state.mcp_clients.read().await;
    let mcp_client = mcp_client_map
        .get(&server_name)
        .ok_or(NexaError::MCPToolCall(String::from(
            "Can't find the MCP Server with the given name",
        )))?;

    let mut receiver = mcp_client.call_tool(function_name, arguments).await?;

    // Need to install timeout mechanism

    let response = receiver.await;

    if let Ok(mcp_response) = response {
        dbg!(&mcp_response);
        let _ = app.emit(
            "mcp_response",
            EmittedMCPResponse {
                request_id,
                response_id,
                response: serde_json::to_value(mcp_response)?,
            },
        );
    } else {
        let _ = app.emit(
            "mcp_response",
            EmittedMCPResponse {
                request_id,
                response_id,
                response: json!("MCP Error: internal server error"),
            },
        );
    }

    Ok(())
}
