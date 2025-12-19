use crate::error::NexaError;
use crate::mcp::structs::{Id, MCPDataPacket};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{self, json, Value};
use std::process::Stdio;
use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_shell::{
    process::{CommandChild, CommandEvent},
    ShellExt,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::mpsc::{self, Receiver};
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};
use tokio::{self, select};

#[async_trait]
pub trait MCPTransportWriter: Send + Sync {
    async fn send(&self, data: Value) -> Result<(), NexaError>;
}

#[async_trait]
pub trait MCPTransportReader: Send + Sync {
    async fn receive(&mut self) -> Result<MCPDataPacket, NexaError>;
}

pub(crate) struct StdioWriter {
    stdin: Mutex<ChildStdin>,

    _child: Child,
}

#[async_trait]
impl MCPTransportWriter for StdioWriter {
    async fn send(&self, data: Value) -> Result<(), NexaError> {
        let mut stdin_handle = self.stdin.lock().await;

        let mut bytes = serde_json::to_vec(&data)?;
        bytes.push(b'\n');
        stdin_handle.write_all(&bytes).await?;

        Ok(stdin_handle.flush().await?)
    }
}

pub(crate) struct StdioWriter2 {
    stdin: Mutex<CommandChild>,
}

#[async_trait]
impl MCPTransportWriter for StdioWriter2 {
    async fn send(&self, data: Value) -> Result<(), NexaError> {
        let mut stdin_handle = self.stdin.lock().await;

        let mut bytes = serde_json::to_vec(&data)?;
        bytes.push(b'\n');
        Ok(stdin_handle.write(&bytes)?)
    }
}

pub(crate) struct StdioReader {
    stdout: BufReader<ChildStdout>,
}

#[async_trait]
impl MCPTransportReader for StdioReader {
    async fn receive(&mut self) -> Result<MCPDataPacket, NexaError> {
        let mut line = String::new();
        self.stdout.read_line(&mut line).await?;

        Ok(serde_json::from_str(line.trim())?)
    }
}

pub(crate) struct StdioReader2 {
    stdout: Receiver<CommandEvent>,
}

#[async_trait]
impl MCPTransportReader for StdioReader2 {
    async fn receive(&mut self) -> Result<MCPDataPacket, NexaError> {
        let event = self
            .stdout
            .recv()
            .await
            .ok_or(NexaError::MCPConnection(String::from(
                "Lost stdout connection",
            )))?;

        match event {
            CommandEvent::Stdout(bytes) => Ok(serde_json::from_slice(bytes.trim_ascii())?),
            CommandEvent::Stderr(bytes) => {
                dbg!(String::from_utf8_lossy(&bytes));
                Err(NexaError::MCPConnection(
                    String::from_utf8_lossy(&bytes).to_string(),
                ))
            }
            a => {
                dbg!(a);
                Err(NexaError::MCPConnection(String::from(
                    "Read error over stdin",
                )))
            }
        }
    }
}

pub(crate) enum MCPConnection {
    Stdio(MCPStdioConnection),
}

pub(crate) struct MCPStdioConnection {
    command: String,
    args: Vec<String>,
    stdin: Mutex<ChildStdin>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,

    _child_process: Child,
}

pub(crate) fn mcp_stdio_connect<S, IS, IP>(
    command: S,
    args: IS,
    envs: IP,
) -> Result<(StdioWriter, StdioReader), NexaError>
where
    S: Into<String>,
    IS: IntoIterator<Item = S>,
    IP: IntoIterator<Item = (S, S)>,
{
    let mut cmd = Command::new(command.into());
    cmd.args(args.into_iter().map(|s| s.into()))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

    for (k, v) in envs.into_iter() {
        cmd.env(k.into(), v.into());
    }

    let mut child = cmd.spawn()?;

    let stdin = child
        .stdin
        .take()
        .ok_or(NexaError::MCPConnection(String::from(
            "Missing stdin for stdio type MCP",
        )))?;
    let stdout = child
        .stdout
        .take()
        .ok_or(NexaError::MCPConnection(String::from(
            "Missing stdout for stdio type MCP",
        )))?;

    let reader = BufReader::new(stdout);

    Ok((
        StdioWriter {
            stdin: Mutex::new(stdin),
            _child: child,
        },
        StdioReader { stdout: reader },
    ))
}

pub(crate) async fn mcp_stdio_connect2<S, IS, IP>(
    app: AppHandle,
    command: S,
    args: IS,
    envs: IP,
) -> Result<(StdioWriter2, StdioReader2), NexaError>
where
    S: Into<String>,
    IS: IntoIterator<Item = S>,
    IP: IntoIterator<Item = (S, S)>,
{
    let shell = app.shell().sidecar(command.into())?;
    let (mut rx, child) = shell
        .args(args.into_iter().map(|arg| arg.into()))
        .envs(envs.into_iter().map(|(k, v)| (k.into(), v.into())))
        .spawn()?;

    // Cleaning pipe
    rx = clean_pipe(rx).await;
    // let mut counts = 30;
    // while counts > 0 {
    //     std::thread::sleep(std::time::Duration::from_secs(1));
    //     counts -= 1;
    //     let packet = rx.try_recv();
    //
    //     if let Ok(event) = packet {
    //         match event {
    //             CommandEvent::Stderr(bytes) => {
    //                 dbg!(String::from_utf8_lossy(&bytes));
    //             }
    //             _ => {
    //                 dbg!("not stderr");
    //             }
    //         }
    //     }
    // }

    Ok((
        StdioWriter2 {
            stdin: Mutex::new(child),
        },
        StdioReader2 { stdout: rx },
    ))
}

async fn clean_pipe(mut rx: Receiver<CommandEvent>) -> Receiver<CommandEvent> {
    loop {
        select! {
            _ = sleep(Duration::from_secs(3)) => {
            break;
        }
            result = rx.recv() => {
                if let Some(event) = result {
                    match event {
                        CommandEvent::Stderr(bytes) => {
                            dbg!(String::from_utf8_lossy(&bytes));
                        }
                        _ => {
                            dbg!("Not stderr");
                        }
                    }
                }
            }
        }
    }

    rx
}

impl MCPStdioConnection {
    pub fn connect(
        command: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, NexaError> {
        let args: Vec<String> = args.into_iter().map(|s| s.into()).collect();
        let command: String = command.into();

        let mut child = Command::new(command.clone())
            .args(args.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = child
            .stdin
            .take()
            .ok_or(NexaError::MCPConnection(String::from(
                "Missing stdin for stdio type MCP",
            )))?;
        let stdout = child
            .stdout
            .take()
            .ok_or(NexaError::MCPConnection(String::from(
                "Missing stdout for stdio type MCP",
            )))?;

        let reader = BufReader::new(stdout);

        Ok(MCPStdioConnection {
            command: command,
            args,
            stdin: Mutex::new(stdin),
            stdout: Arc::new(Mutex::new(reader)),
            _child_process: child,
        })
    }

    fn raise_for_output_borrowed(&self) -> Result<(), NexaError> {
        if Arc::strong_count(&self.stdout) > 1 {
            return Err(NexaError::MCPConnection(String::from(
                "Stdout is borrowed by another process",
            )));
        }

        Ok(())
    }

    pub async fn send<S: Serialize>(&self, data: S) -> Result<(), NexaError> {
        let mut stdin_handle = self.stdin.lock().await;

        let mut bytes = serde_json::to_vec(&data)?;
        bytes.push(b'\n');
        stdin_handle.write_all(&bytes).await?;

        Ok(stdin_handle.flush().await?)
    }

    pub async fn receive(&self) -> Result<MCPDataPacket, NexaError> {
        self.raise_for_output_borrowed()?;

        let mut line = String::new();
        let mut stdout_handle = self.stdout.lock().await;
        stdout_handle.read_line(&mut line).await?;

        Ok(serde_json::from_str(line.trim())?)
    }

    pub async fn borrow_output(&self) -> Result<Arc<Mutex<BufReader<ChildStdout>>>, NexaError> {
        self.raise_for_output_borrowed()?;
        Ok(self.stdout.clone())
    }
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
                "capabilities": {},
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
            let _ = stdin_arc_copy
                .lock()
                .await
                .write_all((json_string + "\n").as_bytes())
                .await;

            dbg!("initialize!!");
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
            sleep(Duration::from_secs(9)).await;

            let _ = stdin_arc_copy
                .lock()
                .await
                .write_all((serde_json::to_string(&ping_request).unwrap() + "\n").as_bytes())
                .await;

            dbg!("ping!!");
        });

        // Tool listing
        let tools_listing_request = MCPRequest {
            jsonrpc: JSON_RPC.to_string(),
            id: Id::NumberId(2),
            method: String::from("tools/list"),
            params: Some(json!({"cursor": "optional-cursor-value"})),
        };

        let stdin_arc_copy = stdin_arc.clone();
        tokio::task::spawn(async move {
            sleep(Duration::from_secs(6)).await;

            let _ = stdin_arc_copy
                .lock()
                .await
                .write_all(
                    (serde_json::to_string(&tools_listing_request).unwrap() + "\n").as_bytes(),
                )
                .await;

            dbg!("tools listing!!")
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

                                            dbg!("notified!!");
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
