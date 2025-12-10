// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod api;
mod error;
mod llm;
mod mcp;
use std::{collections::HashMap, sync::Arc};

use llm::commands::{get_all_ollama_chat_models, stream_chat};
use mcp::client::MCPClient;
use mcp::commands::{call_tool, initialize_mcp_client};
use tauri::Manager;
use tauri_plugin_secure_storage;
use tokio::sync::RwLock;

struct AppData {
    mcp_clients: RwLock<HashMap<String, Arc<MCPClient>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppData {
                mcp_clients: RwLock::new(HashMap::new()),
            });

            Ok(())
        })
        .plugin(tauri_plugin_secure_storage::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_secure_storage::init())
        .invoke_handler(tauri::generate_handler![
            get_all_ollama_chat_models,
            stream_chat,
            initialize_mcp_client,
            call_tool,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
