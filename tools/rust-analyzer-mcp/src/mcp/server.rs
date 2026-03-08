use anyhow::Result;
use log::{debug, error, info};
use serde_json::json;
use std::{path::PathBuf, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    sync::Mutex,
};

use crate::{
    lsp::RustAnalyzerClient,
    protocol::mcp::{MCPError, MCPRequest, MCPResponse},
};

pub struct RustAnalyzerMCPServer {
    pub(super) client: Option<RustAnalyzerClient>,
    pub(super) workspace_root: PathBuf,
    indexing_ready: bool,
}

impl Default for RustAnalyzerMCPServer {
    fn default() -> Self {
        Self::new()
    }
}

impl RustAnalyzerMCPServer {
    pub fn new() -> Self {
        Self {
            client: None,
            workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            indexing_ready: false,
        }
    }

    pub fn with_workspace(workspace_root: PathBuf) -> Self {
        // Ensure the workspace root is absolute.
        let workspace_root = workspace_root.canonicalize().unwrap_or_else(|_| {
            // If canonicalize fails, try to make it absolute.
            if workspace_root.is_absolute() {
                workspace_root.clone()
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(&workspace_root)
            }
        });

        Self {
            client: None,
            workspace_root,
            indexing_ready: false,
        }
    }

    pub(super) async fn ensure_client_started(&mut self) -> Result<()> {
        if self.client.is_none() {
            let mut client = RustAnalyzerClient::new(self.workspace_root.clone());
            client.start().await?;
            self.client = Some(client);
            self.indexing_ready = false;
        }

        // Wait for rust-analyzer to finish indexing (once per session).
        if !self.indexing_ready {
            self.wait_for_indexing().await?;
            self.indexing_ready = true;
        }
        Ok(())
    }

    /// Poll rust-analyzer until it can resolve a hover request on a real symbol.
    /// This is the most reliable readiness indicator — documentSymbol returns
    /// quickly for simple files (mod declarations) before full analysis is done.
    async fn wait_for_indexing(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(180);
        let poll_interval = std::time::Duration::from_secs(5);

        info!("Waiting for rust-analyzer to finish indexing...");

        // Find a probe file with a known symbol to hover on.
        let probe = self.find_probe_file().await;
        let Some((probe_path, probe_line, probe_char)) = probe else {
            info!("No probe file found, waiting 30s as fallback");
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            return Ok(());
        };

        let absolute_path = self.workspace_root.join(&probe_path);
        let absolute_path = absolute_path
            .canonicalize()
            .unwrap_or_else(|_| absolute_path.clone());
        let uri = format!("file://{}", absolute_path.display());

        loop {
            if start.elapsed() > timeout {
                info!("Indexing wait timed out after {:?}, proceeding anyway", timeout);
                return Ok(());
            }

            let Some(client) = &mut self.client else {
                return Err(anyhow::anyhow!("Client not initialized"));
            };

            if let Ok(content) = tokio::fs::read_to_string(&absolute_path).await {
                let _ = client.open_document(&uri, &content).await;

                // Try hover — returns null while RA is still analyzing.
                match client.hover(&uri, probe_line, probe_char).await {
                    Ok(result) => {
                        if !result.is_null() {
                            info!(
                                "rust-analyzer ready after {:?} (hover succeeded on {}:{})",
                                start.elapsed(),
                                probe_path,
                                probe_line
                            );
                            return Ok(());
                        }
                    }
                    Err(_) => {}
                }
            }

            info!(
                "rust-analyzer still indexing... ({:.0}s elapsed)",
                start.elapsed().as_secs_f64()
            );
            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Find a Rust source file with a known symbol position to use as a readiness probe.
    /// Returns (file_path, line, character) where line/character point to a `pub fn` or `pub struct`.
    async fn find_probe_file(&self) -> Option<(String, u32, u32)> {
        // Try known files with stable symbol positions.
        let candidates = [
            ("crates/engine/src/lib.rs", 0, 0),
            ("src/lib.rs", 0, 0),
            ("src/main.rs", 0, 0),
        ];

        for (path, _, _) in &candidates {
            let full = self.workspace_root.join(path);
            if let Ok(content) = tokio::fs::read_to_string(&full).await {
                // Find the first `pub fn` or `pub struct` line for a reliable hover target.
                for (i, line) in content.lines().enumerate() {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with("pub fn ") || trimmed.starts_with("pub struct ") {
                        let offset = line.len() - trimmed.len();
                        let char_pos = offset as u32 + 4; // "pub " = 4 chars, land on fn/struct name
                        if trimmed.starts_with("pub struct ") {
                            return Some((path.to_string(), i as u32, offset as u32 + 11));
                        }
                        return Some((path.to_string(), i as u32, char_pos + 3)); // "pub fn " = 7
                    }
                }
                // File exists but no pub fn/struct — use position 0,0.
                return Some((path.to_string(), 0, 0));
            }
        }
        None
    }

    pub(super) async fn open_document_if_needed(&mut self, file_path: &str) -> Result<String> {
        let absolute_path = self.workspace_root.join(file_path);
        // Ensure we have an absolute path for the URI.
        let absolute_path = absolute_path
            .canonicalize()
            .unwrap_or_else(|_| absolute_path.clone());
        let uri = format!("file://{}", absolute_path.display());
        let content = tokio::fs::read_to_string(&absolute_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path, e))?;

        let Some(client) = &mut self.client else {
            return Err(anyhow::anyhow!("Client not initialized"));
        };

        client.open_document(&uri, &content).await?;
        Ok(uri)
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting rust-analyzer MCP server");

        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut writer = BufWriter::new(stdout);

        // Handle shutdown signals.
        let running = Arc::new(Mutex::new(true));
        let running_clone = Arc::clone(&running);

        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            info!("Received shutdown signal");
            *running_clone.lock().await = false;
        });

        loop {
            // Check if we should stop.
            if !*running.lock().await {
                break;
            }

            let mut line = String::new();
            let bytes_read = match reader.read_line(&mut line).await {
                Ok(n) => n,
                Err(e) => {
                    error!("Error reading from stdin: {}", e);
                    break;
                }
            };

            if bytes_read == 0 {
                break; // EOF
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let Ok(request) = serde_json::from_str::<MCPRequest>(line) else {
                debug!("Failed to parse request: {}", line);
                continue;
            };

            debug!("Received request: {}", request.method);
            let response = self.handle_request(request).await;
            let response_json = serde_json::to_string(&response)?;
            writer.write_all(response_json.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
        }

        // Cleanup.
        info!("Shutting down");
        if let Some(client) = &mut self.client {
            let _ = client.shutdown().await;
        }

        Ok(())
    }

    async fn handle_request(&mut self, request: MCPRequest) -> MCPResponse {
        match request.method.as_str() {
            "initialize" => MCPResponse::Success {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "rust-analyzer-mcp",
                        "version": "0.1.0"
                    },
                    "capabilities": {
                        "tools": {}
                    }
                }),
            },
            "tools/list" => MCPResponse::Success {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: json!({
                    "tools": super::tools::get_tools()
                }),
            },
            "tools/call" => {
                let Some(params) = request.params else {
                    return MCPResponse::Error {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        error: MCPError {
                            code: -32602,
                            message: "Invalid params".to_string(),
                            data: None,
                        },
                    };
                };

                let Some(tool_name) = params["name"].as_str() else {
                    return MCPResponse::Error {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        error: MCPError {
                            code: -32602,
                            message: "Missing tool name".to_string(),
                            data: None,
                        },
                    };
                };

                let args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));

                match super::handlers::handle_tool_call(self, tool_name, args).await {
                    Ok(result) => MCPResponse::Success {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: serde_json::to_value(result).unwrap(),
                    },
                    Err(e) => {
                        error!("Tool call error: {}", e);
                        MCPResponse::Error {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            error: MCPError {
                                code: -1,
                                message: e.to_string(),
                                data: None,
                            },
                        }
                    }
                }
            }
            _ => MCPResponse::Error {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                error: MCPError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                },
            },
        }
    }
}
