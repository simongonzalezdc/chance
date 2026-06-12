use super::protocol::*;
use super::tools;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

const PROTOCOL_VERSION: &str = "2024-11-05";

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin.lock());
    let mut initialized = false;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match handle_message(line, &mut initialized) {
            Some(response) => {
                let out = serde_json::to_string(&response)?;
                println!("{}", out);
                io::stdout().flush()?;
            }
            None => {}
        }
    }

    Ok(())
}

fn handle_message(line: &str, initialized: &mut bool) -> Option<JsonRpcResponse> {
    let request: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(req) => req,
        Err(e) => {
            return Some(JsonRpcResponse::parse_error(format!(
                "failed to parse JSON-RPC request: {}",
                e
            )));
        }
    };

    // Validate jsonrpc version.
    if request.jsonrpc != "2.0" {
        return Some(JsonRpcResponse::invalid_params(
            request.id.clone(),
            "jsonrpc field must be '2.0'",
        ));
    }

    let id = request.id.clone();

    match request.method.as_str() {
        "initialize" => {
            let result = initialize_result();
            *initialized = true;
            Some(JsonRpcResponse::ok(id, result))
        }
        "notifications/initialized" => {
            // No response for notifications.
            None
        }
        method if !*initialized && method != "initialize" => Some(JsonRpcResponse::invalid_params(
            id,
            "server not initialized; send initialize first",
        )),
        "tools/list" => {
            let tools = tools::all_tools();
            let result = json!({ "tools": tools });
            Some(JsonRpcResponse::ok(id, result))
        }
        "tools/call" => {
            let params: CallToolParams = match request.params {
                Some(value) => serde_json::from_value(value).unwrap_or_else(|_| CallToolParams {
                    name: String::new(),
                    arguments: None,
                }),
                None => CallToolParams {
                    name: String::new(),
                    arguments: None,
                },
            };

            if params.name.is_empty() {
                return Some(JsonRpcResponse::invalid_params(
                    id,
                    "missing tool name",
                ));
            }

            let tool_result = tools::call_tool(&params);
            let result = match serde_json::to_value(tool_result) {
                Ok(v) => v,
                Err(e) => {
                    return Some(JsonRpcResponse::internal_error(
                        id,
                        format!("failed to serialize tool result: {}", e),
                    ));
                }
            };
            Some(JsonRpcResponse::ok(id, result))
        }
        _ => Some(JsonRpcResponse::method_not_found(id, &request.method)),
    }
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": {
                "listChanged": false
            }
        },
        "serverInfo": {
            "name": "chance",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}
