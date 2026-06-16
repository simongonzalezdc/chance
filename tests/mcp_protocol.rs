// MCP protocol edge-case tests: drive the real `chance mcp` subprocess over
// its stdio JSON-RPC transport with adversarial protocol inputs — out-of-order
// initialization, wrong protocol version, malformed JSON, oversized payloads,
// unknown methods. Verifies correct JSON-RPC error responses with no crash.

use std::io::Write;
use std::process::{Command, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_chance");

/// Spawn `chance mcp`, feed the given newline-delimited requests on stdin,
/// close stdin, wait for exit, and return all stdout output.
fn mcp_exchange(requests: &[&str]) -> String {
    let mut child = Command::new(BIN)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn chance mcp");
    {
        let mut stdin = child.stdin.take().expect("no stdin handle");
        for line in requests {
            writeln!(stdin, "{}", line).expect("write to mcp stdin failed");
        }
        // stdin dropped here → EOF, server drains and exits
    }
    let output = child
        .wait_with_output()
        .expect("failed to wait on chance mcp");
    assert!(
        output.status.success(),
        "chance mcp exited with {:?}: stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn parse_responses(output: &str) -> Vec<serde_json::Value> {
    output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .collect()
}

#[test]
fn tools_list_before_initialize_is_rejected() {
    let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
    let out = mcp_exchange(&[req]);
    let responses = parse_responses(&out);
    assert_eq!(responses.len(), 1, "expected exactly one response: {}", out);
    let resp = &responses[0];
    assert!(resp.get("error").is_some(), "expected error response: {}", resp);
    let msg = resp["error"]["message"]
        .as_str()
        .unwrap_or("")
        .to_lowercase();
    assert!(
        msg.contains("not initialized") || msg.contains("initializ"),
        "unexpected error message for out-of-order call: {}",
        msg
    );
}

#[test]
fn wrong_jsonrpc_version_is_rejected() {
    let req = r#"{"jsonrpc":"1.0","id":2,"method":"initialize"}"#;
    let out = mcp_exchange(&[req]);
    let responses = parse_responses(&out);
    assert_eq!(responses.len(), 1, "expected one response: {}", out);
    let resp = &responses[0];
    assert!(resp.get("error").is_some(), "expected error response: {}", resp);
    let msg = resp["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("2.0"),
        "unexpected version-rejection message: {}",
        msg
    );
}

#[test]
fn malformed_json_returns_parse_error() {
    let out = mcp_exchange(&["{this is not valid json"]);
    let responses = parse_responses(&out);
    assert_eq!(responses.len(), 1, "expected one parse-error response: {}", out);
    let resp = &responses[0];
    assert!(resp.get("error").is_some(), "expected error envelope: {}", resp);
    let code = resp["error"]["code"].as_i64().unwrap_or(0);
    // -32700 is the standard JSON-RPC parse error code.
    assert_eq!(code, -32700, "expected parse error code -32700: {}", resp);
}

#[test]
fn unknown_method_returns_method_not_found() {
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#;
    let unknown = r#"{"jsonrpc":"2.0","id":2,"method":"frobnicate"}"#;
    let out = mcp_exchange(&[init, unknown]);
    let responses = parse_responses(&out);
    assert!(responses.len() >= 2, "expected >=2 responses: {}", out);
    let last = responses.last().unwrap();
    assert!(last.get("error").is_some(), "expected error for unknown method: {}", last);
    let code = last["error"]["code"].as_i64().unwrap_or(0);
    // -32601 is the standard JSON-RPC "method not found" code.
    assert_eq!(code, -32601, "expected method-not-found code -32601: {}", last);
}

#[test]
fn oversized_payload_handled_gracefully() {
    // A 100 KB garbage notation: the parser must reject it cleanly and the
    // server must return a structured response, not crash or hang.
    let garbage = "z".repeat(100_000);
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#;
    let call = format!(
        r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"chance_roll","arguments":{{"notation":"{}"}}}}}}"#,
        garbage
    );
    let out = mcp_exchange(&[init, &call]);
    let responses = parse_responses(&out);
    assert!(
        !responses.is_empty(),
        "no responses returned for oversized payload"
    );
    let last = responses.last().unwrap();
    // Must be a well-formed envelope (result or error), proving no crash.
    assert!(
        last.get("result").is_some() || last.get("error").is_some(),
        "oversized payload produced a malformed response: {}",
        last
    );
}

#[test]
fn full_handshake_then_tool_call_roundtrip() {
    // Positive control: initialize → tools/list → tools/call all succeed,
    // confirming the protocol layer works end-to-end before the edge cases.
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#;
    let list = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
    let call = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"chance_flip","arguments":{"times":1}}}"#;
    let out = mcp_exchange(&[init, list, call]);
    let responses = parse_responses(&out);
    assert!(responses.len() >= 3, "expected >=3 responses: {}", out);
    // Every response should carry its id and be a success (has "result").
    for r in &responses {
        assert!(r.get("id").is_some(), "response missing id: {}", r);
        assert!(r.get("result").is_some(), "expected success result: {}", r);
    }
}
