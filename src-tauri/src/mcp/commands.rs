use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::{error::NexaError, mcp::client::MCPClient, AppData};

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
