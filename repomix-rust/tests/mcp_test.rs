use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::TempDir;
use serde_json::{json, Value};

#[test]
fn test_mcp_server_pack_codebase() {
    // 1. Setup test environment
    let dir = TempDir::new().unwrap();
    let input_dir = dir.path().join("input");
    fs::create_dir(&input_dir).unwrap();
    let file_path = input_dir.join("test.txt");
    fs::write(&file_path, "Hello MCP").unwrap();

    // 2. Build binary if needed (assumes debug build exists/is fresh enough from cargo test)
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bin_path = repo_root.join("target/debug/repomix-rs");

    // 3. Start server process
    let mut child = Command::new(&bin_path)
        .arg("--server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) // Capture stderr to avoid polluting test output
        .spawn()
        .expect("Failed to start repomix-rs server");

    let child_stdin = child.stdin.as_mut().unwrap();
    let child_stdout = child.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(child_stdout);

    // 4. Helper to send request and read response
    let mut send_request = |method: &str, params: Option<Value>| -> Value {
        let req = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        let req_str = serde_json::to_string(&req).unwrap();
        writeln!(child_stdin, "{}", req_str).unwrap();
        child_stdin.flush().unwrap();

        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        
        if line.is_empty() {
            panic!("Server closed connection unexpectedly");
        }

        let res: Value = serde_json::from_str(&line).expect("Failed to parse response");
        res
    };

    // 5. Test initialize
    let init_res = send_request("initialize", None);
    assert!(init_res.get("result").is_some());
    assert_eq!(init_res["result"]["serverInfo"]["name"], "repomix");

    // 6. Test tools/list
    let list_res = send_request("tools/list", None);
    let tools = list_res["result"]["tools"].as_array().unwrap();
    assert!(tools.iter().any(|t| t["name"] == "pack_codebase"));

    // 7. Test tools/call pack_codebase
    let call_res = send_request("tools/call", Some(json!({
        "name": "pack_codebase",
        "arguments": {
            "directory": input_dir.to_str().unwrap(),
            "style": "plain"
        }
    })));

    if let Some(err) = call_res.get("error") {
        if !err.is_null() {
            panic!("Tool call failed: {:?}", err);
        }
    }
    
    let content = &call_res["result"]["content"][0]["text"];
    assert!(content.is_string(), "Content text not found or not a string: {:?}", call_res);
    let text = content.as_str().unwrap();
    
    assert!(text.contains("Hello MCP"));
    assert!(text.contains("test.txt"));

    // 8. Cleanup
    let _ = child.kill();
}
