// Integration tests for the MCP surface: tool listing and dispatch.
// Tests call_tool and all_tools directly, covering happy paths
// and adversarial inputs (unknown tools, missing required params,
// invalid arguments, malformed JSON payloads).

use chance::mcp::tools::{all_tools, call_tool};
use chance::mcp::protocol::CallToolParams;
use serde_json::json;

// ── Tool Listing ─────────────────────────────────────────────────────────

#[test]
fn all_tools_returns_expected_count() {
    let tools = all_tools();
    // There should be at least 21 tools (roll, flip, draw, pick, shuffle,
    // integer, bytes, uuid, password, runes, iching, tarot, dominoes,
    // roulette, lottery, knucklebones, teetotum, cowrie, lots, sources, health)
    assert!(tools.len() >= 21);
}

#[test]
fn all_tools_have_names_and_schemas() {
    for tool in &all_tools() {
        assert!(!tool.name.is_empty(), "tool name must not be empty");
        assert!(
            tool.input_schema.is_object(),
            "tool {} must have an object input_schema",
            tool.name
        );
    }
}

#[test]
fn expected_tool_names_present() {
    let names: Vec<String> = all_tools().iter().map(|t| t.name.clone()).collect();
    for expected in &[
        "chance_roll",
        "chance_flip",
        "chance_draw",
        "chance_pick",
        "chance_shuffle",
        "chance_integer",
        "chance_bytes",
        "chance_uuid",
        "chance_password",
        "chance_sources",
        "chance_health",
    ] {
        assert!(
            names.iter().any(|n| n == *expected),
            "expected tool '{}' not found in tools list",
            expected
        );
    }
}

// ── Happy Path Tool Calls ────────────────────────────────────────────────

#[test]
fn call_roll_d20_succeeds() {
    let params = CallToolParams {
        name: "chance_roll".into(),
        arguments: Some(json!({"notation": "d20"})),
    };
    let result = call_tool(&params);
    assert!(!result.is_error, "roll should not error");
    assert!(!result.content.is_empty());
}

#[test]
fn call_roll_default_notation_succeeds() {
    let params = CallToolParams {
        name: "chance_roll".into(),
        arguments: Some(json!({})),
    };
    let result = call_tool(&params);
    assert!(!result.is_error, "roll with default notation should succeed");
}

#[test]
fn call_flip_succeeds() {
    let params = CallToolParams {
        name: "chance_flip".into(),
        arguments: Some(json!({"times": 3})),
    };
    let result = call_tool(&params);
    assert!(!result.is_error);
}

#[test]
fn call_pick_succeeds() {
    let params = CallToolParams {
        name: "chance_pick".into(),
        arguments: Some(json!({"items": ["alice", "bob", "carol"], "count": 1})),
    };
    let result = call_tool(&params);
    assert!(!result.is_error);
}

#[test]
fn call_sources_succeeds() {
    let params = CallToolParams {
        name: "chance_sources".into(),
        arguments: None,
    };
    let result = call_tool(&params);
    assert!(!result.is_error);
}

#[test]
fn call_health_succeeds() {
    let params = CallToolParams {
        name: "chance_health".into(),
        arguments: None,
    };
    let result = call_tool(&params);
    assert!(!result.is_error);
}

#[test]
fn call_integer_succeeds() {
    let params = CallToolParams {
        name: "chance_integer".into(),
        arguments: Some(json!({"min": 1, "max": 100})),
    };
    let result = call_tool(&params);
    assert!(!result.is_error);
}

// ── Adversarial Tool Calls ───────────────────────────────────────────────

#[test]
fn call_unknown_tool_errors() {
    let params = CallToolParams {
        name: "chance_nonexistent".into(),
        arguments: None,
    };
    let result = call_tool(&params);
    assert!(result.is_error, "unknown tool should set is_error=true");
}

#[test]
fn call_empty_tool_name_errors() {
    let params = CallToolParams {
        name: "".into(),
        arguments: None,
    };
    let result = call_tool(&params);
    assert!(result.is_error);
}

#[test]
fn call_roll_bad_notation_errors() {
    let params = CallToolParams {
        name: "chance_roll".into(),
        arguments: Some(json!({"notation": "xyzzy"})),
    };
    let result = call_tool(&params);
    assert!(result.is_error, "bad dice notation should error");
}

#[test]
fn call_roll_d0_errors() {
    let params = CallToolParams {
        name: "chance_roll".into(),
        arguments: Some(json!({"notation": "d0"})),
    };
    let result = call_tool(&params);
    assert!(result.is_error, "d0 should error");
}

#[test]
fn call_pick_empty_items_errors() {
    let params = CallToolParams {
        name: "chance_pick".into(),
        arguments: Some(json!({"items": [], "count": 1})),
    };
    let result = call_tool(&params);
    assert!(result.is_error, "picking from empty list should error");
}

#[test]
fn call_pick_count_exceeds_items_errors() {
    let params = CallToolParams {
        name: "chance_pick".into(),
        arguments: Some(json!({"items": ["a"], "count": 10})),
    };
    let result = call_tool(&params);
    assert!(result.is_error, "picking more than available should error");
}

#[test]
fn call_integer_inverted_range_errors() {
    let params = CallToolParams {
        name: "chance_integer".into(),
        arguments: Some(json!({"min": 100, "max": 1})),
    };
    let result = call_tool(&params);
    assert!(result.is_error, "inverted range should error");
}

#[test]
fn call_roll_unsupported_source_errors() {
    let params = CallToolParams {
        name: "chance_roll".into(),
        arguments: Some(json!({"notation": "d20", "source": "fake-source"})),
    };
    let result = call_tool(&params);
    assert!(result.is_error, "unsupported source should error");
}

#[test]
fn call_roll_with_invalid_argument_type_errors() {
    // notation should be a string; passing a number should fail deserialization
    let params = CallToolParams {
        name: "chance_roll".into(),
        arguments: Some(json!({"notation": 12345})),
    };
    let result = call_tool(&params);
    assert!(result.is_error, "wrong argument type should error");
}

#[test]
fn call_roll_with_null_arguments() {
    // Null arguments should use defaults (empty object → default notation d20)
    let params = CallToolParams {
        name: "chance_roll".into(),
        arguments: None,
    };
    let result = call_tool(&params);
    assert!(!result.is_error, "null arguments should use defaults");
}