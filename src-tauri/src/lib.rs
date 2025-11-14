// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod api;
mod llm;
use llm::base::{get_all_ollama_chat_models, stream_chat};
use tauri_plugin_secure_storage;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_secure_storage::init())
        .invoke_handler(tauri::generate_handler![
            get_all_ollama_chat_models,
            stream_chat
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
