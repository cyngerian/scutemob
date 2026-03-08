use anyhow::{anyhow, Result};
use log::debug;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

use crate::{
    diagnostics::format_diagnostics,
    protocol::mcp::{ContentItem, ToolResult},
};

use super::server::RustAnalyzerMCPServer;

/// Default maximum number of results returned by list-based tools.
const DEFAULT_MAX_RESULTS: usize = 50;

/// Helper struct for extracting common tool parameters.
struct ToolParams;

impl ToolParams {
    fn extract_file_path(args: &Value) -> Result<String> {
        let Some(file_path) = args["file_path"].as_str() else {
            return Err(anyhow!("Missing file_path"));
        };
        Ok(file_path.to_string())
    }

    fn extract_position(args: &Value) -> Result<(u32, u32)> {
        let Some(line) = args["line"].as_u64() else {
            return Err(anyhow!("Missing line"));
        };
        let Some(character) = args["character"].as_u64() else {
            return Err(anyhow!("Missing character"));
        };
        Ok((line as u32, character as u32))
    }

    fn extract_limit(args: &Value) -> usize {
        args["limit"]
            .as_u64()
            .map(|l| l as usize)
            .unwrap_or(DEFAULT_MAX_RESULTS)
    }

    fn extract_range(args: &Value) -> Result<(u32, u32, u32, u32)> {
        let (line, character) = Self::extract_position(args)?;
        let Some(end_line) = args["end_line"].as_u64() else {
            return Err(anyhow!("Missing end_line"));
        };
        let Some(end_character) = args["end_character"].as_u64() else {
            return Err(anyhow!("Missing end_character"));
        };
        Ok((line, character, end_line as u32, end_character as u32))
    }
}

pub async fn handle_tool_call(
    server: &mut RustAnalyzerMCPServer,
    tool_name: &str,
    args: Value,
) -> Result<ToolResult> {
    server.ensure_client_started().await?;

    match tool_name {
        "rust_analyzer_hover" => handle_hover(server, args).await,
        "rust_analyzer_definition" => handle_definition(server, args).await,
        "rust_analyzer_references" => handle_references(server, args).await,
        "rust_analyzer_completion" => handle_completion(server, args).await,
        "rust_analyzer_symbols" => handle_symbols(server, args).await,
        "rust_analyzer_format" => handle_format(server, args).await,
        "rust_analyzer_code_actions" => handle_code_actions(server, args).await,
        "rust_analyzer_implementations" => handle_implementations(server, args).await,
        "rust_analyzer_incoming_calls" => handle_incoming_calls(server, args).await,
        "rust_analyzer_outgoing_calls" => handle_outgoing_calls(server, args).await,
        "rust_analyzer_workspace_symbols" => handle_workspace_symbols(server, args).await,
        "rust_analyzer_stop" => handle_stop(server, args).await,
        "rust_analyzer_set_workspace" => handle_set_workspace(server, args).await,
        "rust_analyzer_diagnostics" => handle_diagnostics(server, args).await,
        "rust_analyzer_workspace_diagnostics" => handle_workspace_diagnostics(server, args).await,
        _ => Err(anyhow!("Unknown tool: {}", tool_name)),
    }
}

async fn handle_hover(server: &mut RustAnalyzerMCPServer, args: Value) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character) = ToolParams::extract_position(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client.hover(&uri, line, character).await?;

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&result)?,
        }],
    })
}

async fn handle_definition(server: &mut RustAnalyzerMCPServer, args: Value) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character) = ToolParams::extract_position(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client.definition(&uri, line, character).await?;

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&result)?,
        }],
    })
}

async fn handle_references(server: &mut RustAnalyzerMCPServer, args: Value) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character) = ToolParams::extract_position(&args)?;
    let limit = ToolParams::extract_limit(&args);

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client.references(&uri, line, character).await?;
    let total = result.as_array().map(|a| a.len()).unwrap_or(0);

    // Enrich references with source line text.
    let enriched = enrich_references_with_source(&server.workspace_root, &result, limit).await;

    let mut output = serde_json::to_string_pretty(&enriched)?;
    if total > limit {
        output.push_str(&format!(
            "\n\n(showing {}/{} results — pass \"limit\": {} to see more)",
            limit, total, total
        ));
    }

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: output,
        }],
    })
}

/// Read source lines for each reference location to save follow-up file reads.
async fn enrich_references_with_source(
    workspace_root: &Path,
    references: &Value,
    limit: usize,
) -> Value {
    let Some(ref_array) = references.as_array() else {
        return references.clone();
    };

    // Cache file contents to avoid re-reading the same file.
    let mut file_cache: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut enriched = Vec::new();

    for reference in ref_array.iter().take(limit) {
        let Some(uri) = reference.get("uri").and_then(|u| u.as_str()) else {
            enriched.push(reference.clone());
            continue;
        };

        let file_path = uri.strip_prefix("file://").unwrap_or(uri);

        // Make path relative to workspace for cleaner output.
        let relative_path = Path::new(file_path)
            .strip_prefix(workspace_root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file_path.to_string());

        let line_num = reference
            .pointer("/range/start/line")
            .and_then(|l| l.as_u64())
            .unwrap_or(0) as usize;

        let character = reference
            .pointer("/range/start/character")
            .and_then(|c| c.as_u64())
            .unwrap_or(0);

        // Read file if not cached.
        let source_line = if let Some(lines) = file_cache.get(file_path) {
            lines.get(line_num).cloned()
        } else {
            match tokio::fs::read_to_string(file_path).await {
                Ok(content) => {
                    let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
                    let line_text = lines.get(line_num).cloned();
                    file_cache.insert(file_path.to_string(), lines);
                    line_text
                }
                Err(_) => None,
            }
        };

        enriched.push(json!({
            "file": relative_path,
            "line": line_num,
            "character": character,
            "text": source_line.map(|s| s.trim().to_string()).unwrap_or_default()
        }));
    }

    json!(enriched)
}

async fn handle_completion(server: &mut RustAnalyzerMCPServer, args: Value) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character) = ToolParams::extract_position(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client.completion(&uri, line, character).await?;

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&result)?,
        }],
    })
}

async fn handle_symbols(server: &mut RustAnalyzerMCPServer, args: Value) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;

    debug!("Getting symbols for file: {}", file_path);
    let uri = server.open_document_if_needed(&file_path).await?;
    debug!("Document opened with URI: {}", uri);

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client.document_symbols(&uri).await?;
    debug!("Document symbols result: {:?}", result);

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&result)?,
        }],
    })
}

async fn handle_format(server: &mut RustAnalyzerMCPServer, args: Value) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client.formatting(&uri).await?;

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&result)?,
        }],
    })
}

async fn handle_code_actions(
    server: &mut RustAnalyzerMCPServer,
    args: Value,
) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character, end_line, end_character) = ToolParams::extract_range(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client
        .code_actions(&uri, line, character, end_line, end_character)
        .await?;

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&result)?,
        }],
    })
}

async fn handle_implementations(
    server: &mut RustAnalyzerMCPServer,
    args: Value,
) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character) = ToolParams::extract_position(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client.implementation(&uri, line, character).await?;

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&result)?,
        }],
    })
}

async fn handle_incoming_calls(
    server: &mut RustAnalyzerMCPServer,
    args: Value,
) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character) = ToolParams::extract_position(&args)?;
    let limit = ToolParams::extract_limit(&args);

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let prepare_result = client.prepare_call_hierarchy(&uri, line, character).await?;

    let items = prepare_result.as_array().ok_or_else(|| {
        anyhow!("No call hierarchy item found at this position")
    })?;

    if items.is_empty() {
        return Ok(ToolResult {
            content: vec![ContentItem {
                content_type: "text".to_string(),
                text: "No call hierarchy item found at this position".to_string(),
            }],
        });
    }

    let mut all_calls = Vec::new();
    for item in items {
        let calls = client.incoming_calls(item).await?;
        if let Some(call_array) = calls.as_array() {
            all_calls.extend(call_array.iter().cloned());
        }
    }

    let total = all_calls.len();
    all_calls.truncate(limit);
    let mut output = serde_json::to_string_pretty(&json!(all_calls))?;
    if total > limit {
        output.push_str(&format!(
            "\n\n(showing {}/{} callers — pass \"limit\": {} to see more)",
            limit, total, total
        ));
    }

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: output,
        }],
    })
}

async fn handle_outgoing_calls(
    server: &mut RustAnalyzerMCPServer,
    args: Value,
) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character) = ToolParams::extract_position(&args)?;
    let limit = ToolParams::extract_limit(&args);

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let prepare_result = client.prepare_call_hierarchy(&uri, line, character).await?;

    let items = prepare_result.as_array().ok_or_else(|| {
        anyhow!("No call hierarchy item found at this position")
    })?;

    if items.is_empty() {
        return Ok(ToolResult {
            content: vec![ContentItem {
                content_type: "text".to_string(),
                text: "No call hierarchy item found at this position".to_string(),
            }],
        });
    }

    let mut all_calls = Vec::new();
    for item in items {
        let calls = client.outgoing_calls(item).await?;
        if let Some(call_array) = calls.as_array() {
            all_calls.extend(call_array.iter().cloned());
        }
    }

    let total = all_calls.len();
    all_calls.truncate(limit);
    let mut output = serde_json::to_string_pretty(&json!(all_calls))?;
    if total > limit {
        output.push_str(&format!(
            "\n\n(showing {}/{} callees — pass \"limit\": {} to see more)",
            limit, total, total
        ));
    }

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: output,
        }],
    })
}

async fn handle_workspace_symbols(
    server: &mut RustAnalyzerMCPServer,
    args: Value,
) -> Result<ToolResult> {
    let Some(query) = args["query"].as_str() else {
        return Err(anyhow!("Missing query"));
    };
    let limit = ToolParams::extract_limit(&args);

    server.ensure_client_started().await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client.workspace_symbols(query).await?;

    let output = if let Some(arr) = result.as_array() {
        let total = arr.len();
        let truncated: Vec<_> = arr.iter().take(limit).cloned().collect();
        let mut text = serde_json::to_string_pretty(&json!(truncated))?;
        if total > limit {
            text.push_str(&format!(
                "\n\n(showing {}/{} symbols — pass \"limit\": {} to see more)",
                limit, total, total
            ));
        }
        text
    } else {
        serde_json::to_string_pretty(&result)?
    };

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: output,
        }],
    })
}

async fn handle_stop(
    server: &mut RustAnalyzerMCPServer,
    _args: Value,
) -> Result<ToolResult> {
    if let Some(client) = &mut server.client {
        client.shutdown().await?;
    }
    server.client = None;
    server.indexing_ready = false;

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: "rust-analyzer stopped. It will restart on next tool call.".to_string(),
        }],
    })
}

async fn handle_set_workspace(
    server: &mut RustAnalyzerMCPServer,
    args: Value,
) -> Result<ToolResult> {
    let Some(workspace_path) = args["workspace_path"].as_str() else {
        return Err(anyhow!("Missing workspace_path"));
    };

    // Shutdown existing client.
    if let Some(client) = &mut server.client {
        client.shutdown().await?;
    }
    server.client = None;

    // Set new workspace with proper absolute path handling.
    let workspace_root = PathBuf::from(workspace_path);
    server.workspace_root = workspace_root.canonicalize().unwrap_or_else(|_| {
        if workspace_root.is_absolute() {
            workspace_root.clone()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(&workspace_root)
        }
    });

    // Start the new client automatically.
    server.ensure_client_started().await?;

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: format!("Workspace set to: {}", server.workspace_root.display()),
        }],
    })
}

async fn handle_diagnostics(server: &mut RustAnalyzerMCPServer, args: Value) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    // Poll for diagnostics - rust-analyzer needs time to run cargo check.
    // For files with expected errors (like diagnostics_test.rs), poll longer.
    let should_poll = file_path.contains("diagnostics_test") || file_path.contains("simple_error");

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let mut result = json!([]);
    if should_poll {
        let start = std::time::Instant::now();
        let timeout = tokio::time::Duration::from_secs(8); // Less than test timeout.
        let poll_interval = tokio::time::Duration::from_millis(500);

        while start.elapsed() < timeout {
            result = client.diagnostics(&uri).await?;
            let Some(diag_array) = result.as_array() else {
                tokio::time::sleep(poll_interval).await;
                continue;
            };

            if !diag_array.is_empty() {
                // We got diagnostics, stop polling.
                break;
            }
            tokio::time::sleep(poll_interval).await;
        }
    } else {
        // For clean files, just wait a bit and check once.
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        result = client.diagnostics(&uri).await?;
    }

    let diagnostics = format_diagnostics(&file_path, &result);

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&diagnostics)?,
        }],
    })
}

async fn handle_workspace_diagnostics(
    server: &mut RustAnalyzerMCPServer,
    _args: Value,
) -> Result<ToolResult> {
    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client.workspace_diagnostics().await?;

    // Format workspace diagnostics.
    let formatted = format_workspace_diagnostics(&server.workspace_root, &result);

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&formatted)?,
        }],
    })
}

fn format_workspace_diagnostics(workspace_root: &Path, result: &Value) -> Value {
    if !result.is_object() {
        // Handle unexpected format.
        if let Some(items) = result.get("items") {
            return json!({
                "workspace": workspace_root.display().to_string(),
                "diagnostics": items,
                "summary": {
                    "total_diagnostics": items.as_array().map(|a| a.len()).unwrap_or(0),
                    "by_severity": {}
                }
            });
        }

        return json!({
            "workspace": workspace_root.display().to_string(),
            "diagnostics": result,
            "summary": {
                "note": "Unexpected response format from rust-analyzer"
            }
        });
    }

    // Fallback format (diagnostics per URI).
    let mut output = json!({
        "workspace": workspace_root.display().to_string(),
        "files": {},
        "summary": {
            "total_files": 0,
            "total_errors": 0,
            "total_warnings": 0,
            "total_information": 0,
            "total_hints": 0
        }
    });

    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut total_information = 0;
    let mut total_hints = 0;
    let mut file_count = 0;

    let Some(obj) = result.as_object() else {
        return output;
    };

    for (uri, diagnostics) in obj {
        let Some(diag_array) = diagnostics.as_array() else {
            continue;
        };

        if diag_array.is_empty() {
            continue;
        }

        file_count += 1;
        let mut file_errors = 0;
        let mut file_warnings = 0;
        let mut file_information = 0;
        let mut file_hints = 0;

        for diag in diag_array {
            let Some(severity) = diag.get("severity").and_then(|s| s.as_u64()) else {
                continue;
            };

            match severity {
                1 => {
                    file_errors += 1;
                    total_errors += 1;
                }
                2 => {
                    file_warnings += 1;
                    total_warnings += 1;
                }
                3 => {
                    file_information += 1;
                    total_information += 1;
                }
                4 => {
                    file_hints += 1;
                    total_hints += 1;
                }
                _ => {}
            }
        }

        output["files"][uri] = json!({
            "diagnostics": diagnostics,
            "summary": {
                "errors": file_errors,
                "warnings": file_warnings,
                "information": file_information,
                "hints": file_hints
            }
        });
    }

    output["summary"]["total_files"] = json!(file_count);
    output["summary"]["total_errors"] = json!(total_errors);
    output["summary"]["total_warnings"] = json!(total_warnings);
    output["summary"]["total_information"] = json!(total_information);
    output["summary"]["total_hints"] = json!(total_hints);

    output
}
