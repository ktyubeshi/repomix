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
                "name": "repomix_pack",
                "description": "Pack a repository into a single file",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "directory": {
                            "type": "string",
                            "description": "Path to the directory to pack"
                        },
                        "options": {
                            "type": "object",
                            "description": "Repomix configuration options"
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

    if name != "repomix_pack" {
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

    // Load default config
    // TODO: Allow options to override config
    let config = crate::config::schema::RepomixConfig::default();

    let path = std::path::PathBuf::from(directory);
    let result = crate::core::pack::pack(&config, &[path]).map_err(|e| JsonRpcError {
        code: -32603,
        message: format!("Packing failed: {}", e),
        data: None,
    })?;

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": result.output
            }
        ]
    }))
}
