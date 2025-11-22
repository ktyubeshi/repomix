use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

pub fn start_server() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                tracing::error!("Failed to parse JSON-RPC request: {}", e);
                continue;
            }
        };

        let response = handle_request(request);
        let response_str = serde_json::to_string(&response)?;

        writeln!(stdout, "{}", response_str)?;
        stdout.flush()?;
    }

    Ok(())
}

fn handle_request(req: JsonRpcRequest) -> JsonRpcResponse {
    let result = match req.method.as_str() {
        "initialize" => handle_initialize(req.params),
        "tools/list" => handle_tools_list(),
        "tools/call" => handle_tools_call(req.params),
        _ => Err(JsonRpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }),
    };

    match result {
        Ok(res) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(res),
            error: None,
            id: req.id,
        },
        Err(err) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(err),
            id: req.id,
        },
    }
}

fn handle_initialize(_params: Option<Value>) -> Result<Value, JsonRpcError> {
    Ok(json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "repomix",
            "version": "0.1.0"
        }
    }))
}

fn handle_tools_list() -> Result<Value, JsonRpcError> {
    Ok(json!({
        "tools": [
            {
                "name": "pack_codebase",
                "description": "Package a local code directory into a consolidated file for AI analysis. This tool analyzes the codebase structure, extracts relevant code content, and generates a comprehensive report including metrics, file tree, and formatted code content. Supports multiple output formats: XML (structured with <file> tags), Markdown (human-readable with ## headers and code blocks), JSON (machine-readable with files as key-value pairs), and Plain text (simple format with separators). Also supports Tree-sitter compression for efficient token usage.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "directory": {
                            "type": "string",
                            "description": "Directory to pack (Absolute path)"
                        },
                        "compress": {
                            "type": "boolean",
                            "description": "Enable Tree-sitter compression to extract essential code signatures and structure while removing implementation details. Reduces token usage by ~70% while preserving semantic meaning. Generally not needed since grep_repomix_output allows incremental content retrieval. Use only when you specifically need the entire codebase content for large repositories (default: false).",
                            "default": false
                        },
                        "includePatterns": {
                            "type": "string",
                            "description": "Specify files to include using fast-glob patterns. Multiple patterns can be comma-separated (e.g., \"**/*.{js,ts}\", \"src/**,docs/**\"). Only matching files will be processed. Useful for focusing on specific parts of the codebase."
                        },
                        "ignorePatterns": {
                            "type": "string",
                            "description": "Specify additional files to exclude using fast-glob patterns. Multiple patterns can be comma-separated (e.g., \"test/**,*.spec.js\", \"node_modules/**,dist/**\"). These patterns supplement .gitignore and built-in exclusions."
                        },
                        "topFilesLength": {
                            "type": "integer",
                            "description": "Number of largest files by size to display in the metrics summary for codebase analysis (default: 10)",
                            "default": 10,
                            "minimum": 1
                        },
                        "style": {
                            "type": "string",
                            "enum": ["xml", "markdown", "json", "plain"],
                            "description": "Output format style: xml (structured tags, default), markdown (human-readable with code blocks), json (machine-readable key-value), or plain (simple text with separators)",
                            "default": "xml"
                        }
                    },
                    "required": ["directory"]
                }
            }
        ]
    }))
}

fn handle_tools_call(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;

    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or(JsonRpcError {
            code: -32602,
            message: "Missing tool name".to_string(),
            data: None,
        })?;

    if name != "pack_codebase" {
        return Err(JsonRpcError {
            code: -32601,
            message: "Tool not found".to_string(),
            data: None,
        });
    }

    let args = params.get("arguments").ok_or(JsonRpcError {
        code: -32602,
        message: "Missing arguments".to_string(),
        data: None,
    })?;

    let directory = args
        .get("directory")
        .and_then(|v| v.as_str())
        .ok_or(JsonRpcError {
            code: -32602,
            message: "Missing directory argument".to_string(),
            data: None,
        })?;

    // Parse optional arguments
    let compress = args
        .get("compress")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let include_patterns = args.get("includePatterns").and_then(|v| v.as_str());
    let ignore_patterns = args.get("ignorePatterns").and_then(|v| v.as_str());
    let top_files_length = args
        .get("topFilesLength")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;
    let style_str = args.get("style").and_then(|v| v.as_str()).unwrap_or("xml");

    let style = match style_str {
        "xml" => crate::config::schema::RepomixOutputStyle::Xml,
        "markdown" => crate::config::schema::RepomixOutputStyle::Markdown,
        "json" => crate::config::schema::RepomixOutputStyle::Json,
        "plain" => crate::config::schema::RepomixOutputStyle::Plain,
        _ => crate::config::schema::RepomixOutputStyle::Xml,
    };

    // Build config
    let mut config = crate::config::schema::RepomixConfig::default();
    config.output.compress = compress;
    config.output.top_files_length = top_files_length as u32;
    config.output.style = style;
    // For MCP, we usually want security check? Node.js says `securityCheck: true`.
    config.security.enable_security_check = true;
    // For MCP output, we probably don't want to write to a file on disk unless specified?
    // Node.js creates a temp dir. Rust `pack` returns the string content in `PackResult`.
    // We can return the content directly in the tool response.
    // Node.js `runCli` writes to file.
    // Let's set `output.file_path` to None to avoid writing to current dir,
    // OR use a temp dir if `pack` requires it.
    // Rust `pack` writes to file if `config.output.file_path` is Some.
    config.output.file_path = None;
    config.output.stdout = Some(true); // To ensure logging goes to stderr? No, `stdout` flag in Rust just prints output to stdout.
                                       // Wait, `pack` returns `PackResult` containing `output` string. It writes to file only if `file_path` is set.

    if let Some(patterns) = include_patterns {
        for p in patterns.split(',') {
            if !p.trim().is_empty() {
                config.include.push(p.trim().to_string());
            }
        }
    }

    if let Some(patterns) = ignore_patterns {
        for p in patterns.split(',') {
            if !p.trim().is_empty() {
                config.ignore.custom_patterns.push(p.trim().to_string());
            }
        }
    }

    // Set CWD to the target directory so relative paths work?
    // Or `pack` handles absolute paths.
    // Node.js `runCli` takes `cwd`.
    let path = std::path::PathBuf::from(directory);

    // pack takes `&[PathBuf]`.
    let result = crate::core::pack::pack(&config, &[path]).map_err(|e| JsonRpcError {
        code: -32603,
        message: format!("Packing failed: {}", e),
        data: None,
    })?;

    // Format response similar to Node.js formatPackToolResponse
    // Node.js returns a text block with summary.
    // Here we just return the content for now, or a summary.
    // The tool description says "generates a comprehensive report ...".
    // Node.js returns the content of the file.

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": result.output
            }
        ]
    }))
}
