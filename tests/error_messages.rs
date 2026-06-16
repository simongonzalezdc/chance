// Error message quality tests: assert that error responses carry meaningful,
// specific human-readable messages rather than opaque codes. Covers the
// service dispatch layer and the MCP tool layer.

use chance::core::SourceError;
use chance::mcp::protocol::{CallToolParams, CallToolResult, ToolContent};
use chance::mcp::tools::call_tool;
use chance::services;
use chance::services::dto::*;
use chance::sources::create_source;
use serde_json::json;

#[test]
fn roll_d0_error_message_is_descriptive() {
    let req = RollRequest {
        source: SourceRequest::default(),
        notation: "d0".to_string(),
    };
    let err = services::roll(&req).unwrap_err();
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("parse") || msg.contains("sides") || msg.contains("zero"),
        "d0 error message not descriptive: {}",
        msg
    );
}

#[test]
fn invalid_source_error_message_names_the_source() {
    let err = match create_source("totally-bogus-source", None) {
        Ok(_) => panic!("expected error for bogus source name"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(
        msg.contains("totally-bogus-source"),
        "error message should name the bad source: {}",
        msg
    );
    assert!(
        matches!(err, SourceError::InvalidSource(_)),
        "expected InvalidSource variant, got {:?}",
        err
    );
}

#[test]
fn pick_empty_error_message_is_descriptive() {
    let req = ListRequest {
        source: SourceRequest::default(),
        items: vec![],
        count: 1,
    };
    let err = services::pick(&req).unwrap_err();
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("cannot pick") || msg.contains("available"),
        "empty-pick error not descriptive: {}",
        msg
    );
}

#[test]
fn integer_inverted_range_error_is_descriptive() {
    let req = IntRequest {
        source: SourceRequest::default(),
        min: 100,
        max: 1,
    };
    let err = services::integer(&req).unwrap_err();
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("min must be") || msg.contains("<="),
        "inverted-range error not descriptive: {}",
        msg
    );
}

#[test]
fn mcp_unknown_tool_error_message_names_tool() {
    let params = CallToolParams {
        name: "chance_nope".to_string(),
        arguments: None,
    };
    let result = call_tool(&params);
    assert!(result.is_error, "unknown tool should surface as error");
    let text = extract_text(&result);
    assert!(
        text.contains("chance_nope") || text.to_lowercase().contains("unknown"),
        "unknown-tool error should name the tool: {}",
        text
    );
}

#[test]
fn mcp_roll_bad_notation_error_is_descriptive() {
    let params = CallToolParams {
        name: "chance_roll".to_string(),
        arguments: Some(json!({"notation": "xyzzy"})),
    };
    let result = call_tool(&params);
    assert!(result.is_error, "bad notation should surface as error");
    let text = extract_text(&result).to_lowercase();
    assert!(
        text.contains("parse") || text.contains("dice"),
        "bad-notation error not descriptive: {}",
        text
    );
}

#[test]
fn mcp_pick_oversized_error_is_descriptive() {
    let params = CallToolParams {
        name: "chance_pick".to_string(),
        arguments: Some(json!({"items": ["a"], "count": 99})),
    };
    let result = call_tool(&params);
    assert!(result.is_error, "oversized pick should surface as error");
    let text = extract_text(&result).to_lowercase();
    assert!(
        text.contains("cannot pick") || text.contains("available"),
        "oversized-pick error not descriptive: {}",
        text
    );
}

fn extract_text(result: &CallToolResult) -> String {
    match result.content.first() {
        Some(ToolContent::Text { text }) => text.clone(),
        _ => String::new(),
    }
}
