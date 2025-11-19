use thiserror::Error;

#[derive(Error, Debug)]
pub enum NexaError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] tauri_plugin_http::reqwest::Error),
    #[error("Gemini Error: {0}")]
    Gemini(String),
}
