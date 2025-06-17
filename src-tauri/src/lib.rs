mod llm;
use llm::base::{emit_events, get_all_ollama_chat_models, stream_chat};
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            emit_events,
            stream_chat,
            get_all_ollama_chat_models
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
