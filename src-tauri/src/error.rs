use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NexaError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] tauri_plugin_http::reqwest::Error),
    #[error("Serde Json Error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Gemini Error: {0}")]
    Gemini(String),
    #[error("MCP Connection Error: {0}")]
    MCPConnection(String),
    #[error("MCP Tool Call Error: {0}")]
    MCPToolCall(String),
    #[error("Command Error: {0}")]
    Command(String),
}

impl Serialize for NexaError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}
