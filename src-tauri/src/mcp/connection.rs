use crate::error::NexaError;
use serde_json::{self, Value};
use std::process::Stdio;
use std::sync::Arc;
use tokio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::{self, Receiver};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

pub(crate) enum MCPConnection {
    Stdio(MCPStdioConnection),
}

pub(crate) struct MCPStdioConnection {
    command: String,
    args: Vec<String>,
}

#[cfg(test)]
mod tests {
    use crate::mcp::structs::{
        Id, MCPDataPacket, MCPNotification, MCPRequest, MCPResponse, JSON_RPC,
    };

    use super::*; // Import your structs from the parent module
                  //

    #[tokio::test]
    async fn connection_test() {
        let mut child = Command::new("uvx")
            .args([
                "--env-file",
                "/Users/chihlee/temp/.env",
                "--from",
                "git+https://github.com/Jazzcort/status-report-assistant-mcp",
                "mcp-serve",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout);

        let stdin_arc = Arc::new(Mutex::new(stdin));
        dbg!(child.id());

        let (tx, mut rx) = mpsc::channel::<bool>(1);

        tokio::task::spawn(async move {
            sleep(Duration::from_secs(20)).await;
            tx.send(true).await;
        });

        // Initialize
        let initialize_request_string = r#"
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "roots": {
                        "listChanged": true
                    },
                "sampling": {},
                "elicitation": {}
                },
                "clientInfo": {
                    "name": "ExampleClient",
                    "title": "Example Client Display Name",
                    "version": "1.0.0"
                }
            }
        }
        "#;
        let initialize_request: MCPRequest =
            serde_json::from_str(initialize_request_string).unwrap();
        dbg!(&initialize_request);
        let stdin_arc_copy = stdin_arc.clone();
        tokio::task::spawn(async move {
            sleep(Duration::from_secs(3)).await;
            let json_string = serde_json::to_string(&initialize_request).unwrap();
            dbg!(&json_string);
            dbg!("initialize!!");
            let _ = stdin_arc_copy
                .lock()
                .await
                .write_all((json_string + "\n").as_bytes())
                .await;
        });

        // Ping
        let ping_request = MCPRequest {
            jsonrpc: JSON_RPC.to_string(),
            id: Id::NumberId(123),
            method: String::from("ping"),
            params: None,
        };

        let stdin_arc_copy = stdin_arc.clone();

        tokio::task::spawn(async move {
            sleep(Duration::from_secs(6)).await;

            dbg!("ping!!");

            let _ = stdin_arc_copy
                .lock()
                .await
                .write_all(serde_json::to_string(&ping_request).unwrap().as_bytes())
                .await;
        });

        let stdin_arc_copy = stdin_arc.clone();

        loop {
            tokio::select! {
                result = async {
                    let mut line = String::new();
                    let bytes = reader.read_line(&mut line).await?;

                    if bytes == 0 {
                        return Err(NexaError::MCPConnection("MCP server closed the pipe".to_string()));
                    }

                    Ok(serde_json::from_str::<MCPDataPacket>(line.trim()).unwrap())
                } => {
                    if result.is_ok() {
                        let result = result.unwrap();
                        dbg!(&result);

                        match result {
                            MCPDataPacket::Response(response) => {
                                match response {
                                    MCPResponse::Success{jsonrpc, id, result} => {
                                        if id == Id::NumberId(1) {
                                            let initialized_notification = MCPNotification {
                                                jsonrpc: JSON_RPC.to_string(),
                                                method: String::from("notifications/initialized"),
                                                params: None
                                            };

                                            let initialized_notification_string = serde_json::to_string(&initialized_notification).unwrap();

                                            let _ = stdin_arc_copy.lock().await.write_all((initialized_notification_string + "\n").as_bytes()).await;
                                        }
                                    }
                                    _ => {}


                                }

                            },
                            _ => {}
                       }
                    } else {
                        break;
                    }
                }
                result = rx.recv() => {
                    if result.is_none() {
                        break;
                    }

                    let should_break = result.unwrap();
                    if should_break {
                        break;
                    }
                }

            }
        }
        drop(stdin_arc_copy);
        drop(stdin_arc);

        dbg!("waiting!!");
        let a = child.wait().await;
        dbg!(a);
    }
}
