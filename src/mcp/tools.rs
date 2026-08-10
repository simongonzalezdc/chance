use crate::services;
use crate::services::dto::*;
use serde_json::{json, Map, Value};

use super::protocol::{CallToolParams, CallToolResult, Tool};

fn source_properties() -> Value {
    json!({
        "source": {
            "type": "string",
            "description": "Randomness source to use (e.g. os-csprng, chacha20, xoshiro256**, mix:os-csprng,drand). Defaults to os-csprng."
        },
        "seed": {
            "type": "string",
            "description": "Optional seed for deterministic sources."
        }
    })
}

fn schema(required: &[&str], properties: Value) -> Value {
    let mut props = source_properties().as_object().cloned().unwrap_or_default();
    if let Some(extra) = properties.as_object() {
        props.extend(extra.clone());
    }
    json!({
        "type": "object",
        "properties": props,
        "required": required,
    })
}

fn tool(name: &str, description: &str, required: &[&str], properties: Value) -> Tool {
    let req: Vec<String> = required.iter().map(|s| s.to_string()).collect();
    // Source/seed are always optional, so not added to required.
    Tool {
        name: name.to_string(),
        description: Some(description.to_string()),
        input_schema: schema(
            &req.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            properties,
        ),
    }
}

pub fn all_tools() -> Vec<Tool> {
    let mut tools = vec![
        tool(
            "chance_roll",
            "Roll dice using standard RPG notation (e.g. d20, 4d6kh3, 2d20adv). Returns the roll total, individual die values, and modifier breakdown. Use when a game or RPG session needs random dice results. Pass notation directly from the user's roll request.",
            &["notation"],
            json!({
                "notation": {
                    "type": "string",
                    "description": "Dice notation expression.",
                    "minLength": 1,
                    "maxLength": 256,
                    "default": "d20"
                }
            }),
        ),
        tool(
            "chance_flip",
            "Flip one or more coins. Returns each flip result (heads/tails) and aggregate counts. Use when generating a binary random decision or coin toss. Pass times from the user request for how many flips.",
            &[],
            json!({
                "times": {
                    "type": "integer",
                    "description": "Number of flips.",
                    "minimum": 1,
                    "maximum": 1000000,
                    "default": 1
                }
            }),
        ),
        tool(
            "chance_draw",
            "Draw cards from a shuffled standard 52-card deck. Returns each card's rank and suit. Use when dealing a hand of cards for a game. Pass count from the user request for how many cards to draw.",
            &[],
            json!({
                "count": {
                    "type": "integer",
                    "description": "Number of cards to draw.",
                    "minimum": 1,
                    "maximum": 52,
                    "default": 5
                }
            }),
        ),
        tool(
            "chance_pick",
            "Pick one or more distinct winners from a list without replacement. Returns the selected items. Use when choosing random winners or samples from a candidate set. Pass items from the user request and count for how many to pick.",
            &["items"],
            json!({
                "items": {
                    "type": "array",
                    "description": "Items to choose from.",
                    "items": { "type": "string" },
                    "minItems": 1,
                    "maxItems": 100000
                },
                "count": {
                    "type": "integer",
                    "description": "Number of distinct items to pick.",
                    "minimum": 1,
                    "default": 1
                }
            }),
        ),
        tool(
            "chance_shuffle",
            "Shuffle a list of items into a uniformly random order. Returns the full list reordered. Use when randomizing the ordering of known items. Pass items from the user request.",
            &["items"],
            json!({
                "items": {
                    "type": "array",
                    "description": "Items to shuffle.",
                    "items": { "type": "string" },
                    "minItems": 0,
                    "maxItems": 100000
                }
            }),
        ),
        tool(
            "chance_integer",
            "Generate a random integer in an inclusive range. Returns the generated integer value. Use when picking a number within bounds. Pass min and max from the user request to set the range.",
            &[],
            json!({
                "min": {
                    "type": "integer",
                    "description": "Minimum value (inclusive).",
                    "default": 1
                },
                "max": {
                    "type": "integer",
                    "description": "Maximum value (inclusive).",
                    "default": 100
                }
            }),
        ),
        tool(
            "chance_bytes",
            "Generate cryptographically random bytes encoded as hex or base64. Returns the encoded byte string and count. Use when generating tokens, salts, or raw entropy. Pass count and encoding from the user request.",
            &[],
            json!({
                "count": {
                    "type": "integer",
                    "description": "Number of bytes.",
                    "minimum": 1,
                    "maximum": 1048576,
                    "default": 16
                },
                "encoding": {
                    "type": "string",
                    "description": "Output encoding.",
                    "enum": ["hex", "base64"],
                    "default": "hex"
                }
            }),
        ),
        tool(
            "chance_uuid",
            "Generate a random UUID (v4 or v7). Returns the UUID string. Use when creating unique identifiers for records or resources. Pass version from the user request (4 or 7).",
            &[],
            json!({
                "version": {
                    "type": "integer",
                    "description": "UUID version.",
                    "enum": [4, 7],
                    "default": 4
                }
            }),
        ),
        tool(
            "chance_password",
            "Generate a random password. Returns the generated password string. Use when creating credentials or secrets. Pass length and symbols from the user request.",
            &[],
            json!({
                "length": {
                    "type": "integer",
                    "description": "Password length.",
                    "minimum": 1,
                    "maximum": 1024,
                    "default": 16
                },
                "symbols": {
                    "type": "boolean",
                    "description": "Include symbols.",
                    "default": true
                }
            }),
        ),
        tool(
            "chance_runes",
            "Draw Elder Futhark runes. Returns each rune's name, meaning, and upright or merk (reversed) orientation. Use when performing Norse runic divination. Pass count from the user request for how many runes to draw.",
            &[],
            json!({
                "count": {
                    "type": "integer",
                    "description": "Number of runes to draw.",
                    "minimum": 1,
                    "maximum": 24,
                    "default": 1
                }
            }),
        ),
        tool(
            "chance_iching",
            "Cast an I Ching hexagram using the coin or yarrow method. Returns the primary hexagram number, changing lines, and the resulting transformed hexagram. Use when performing I Ching divination. Pass method from the user request (coin or yarrow).",
            &[],
            json!({
                "method": {
                    "type": "string",
                    "description": "Divination method.",
                    "enum": ["coin", "yarrow"],
                    "default": "coin"
                }
            }),
        ),
        tool(
            "chance_tarot",
            "Draw Tarot cards from the full 78-card deck (Major and Minor Arcana). Returns each card's name, arcana, suit, number, and upright/reversed orientation. Use when performing tarot divination. Pass count from the user request for how many cards to draw.",
            &[],
            json!({
                "count": {
                    "type": "integer",
                    "description": "Number of cards to draw.",
                    "minimum": 1,
                    "maximum": 78,
                    "default": 1
                }
            }),
        ),
        tool(
            "chance_dominoes",
            "Draw dominoes from a double-n set. Returns each tile's two pip-counts. Use when generating a domino hand for tabletop play. Pass set and count from the user request.",
            &[],
            json!({
                "set": {
                    "type": "integer",
                    "description": "Double-n set size (e.g. 6 for double-six).",
                    "minimum": 0,
                    "maximum": 18,
                    "default": 6
                },
                "count": {
                    "type": "integer",
                    "description": "Number of dominoes to draw.",
                    "minimum": 1,
                    "maximum": 1000,
                    "default": 1
                }
            }),
        ),
        tool(
            "chance_roulette",
            "Spin a roulette wheel. Returns the winning number, its color, and the wheel variant. Use when simulating casino roulette. Pass variant from the user request (european or american).",
            &[],
            json!({
                "variant": {
                    "type": "string",
                    "description": "Roulette variant.",
                    "enum": ["european", "american"],
                    "default": "european"
                }
            }),
        ),
        tool(
            "chance_lottery",
            "Draw lottery numbers from a numbered pool. Returns the drawn numbers sorted ascending, plus optional bonus balls from a separate pool. Use when simulating a lottery draw. Pass pool, pick, and optional bonus_pool from the user request.",
            &[],
            json!({
                "pool": {
                    "type": "integer",
                    "description": "Highest numbered ball in the pool.",
                    "minimum": 1,
                    "maximum": 99,
                    "default": 49
                },
                "pick": {
                    "type": "integer",
                    "description": "How many numbers to draw.",
                    "minimum": 1,
                    "maximum": 20,
                    "default": 6
                },
                "bonus_pool": {
                    "type": ["integer", "null"],
                    "description": "Optional separate bonus ball pool size.",
                    "minimum": 1,
                    "maximum": 99
                }
            }),
        ),
        tool(
            "chance_knucklebones",
            "Cast knucklebones or astragali (ancient bone dice). Returns each bone's landed face and numeric value. Use when simulating ancient bone-dice divination or games. Pass count from the user request for how many bones to cast.",
            &[],
            json!({
                "count": {
                    "type": "integer",
                    "description": "Number of bones to cast.",
                    "minimum": 1,
                    "maximum": 1000,
                    "default": 4
                }
            }),
        ),
        tool(
            "chance_teetotum",
            "Spin a teetotum or dreidel. Returns the landed face and its meaning. Use when simulating a spinning-top game. Pass dreidel from the user request to select Hebrew dreidel faces over Latin teetotum faces.",
            &[],
            json!({
                "dreidel": {
                    "type": "boolean",
                    "description": "Use Hebrew dreidel faces instead of Latin teetotum faces.",
                    "default": false
                }
            }),
        ),
        tool(
            "chance_cowrie",
            "Cast cowrie shells for Santería or Ifá divination. Returns each shell's orientation (open/closed) and the resulting odu signature. Use when performing Afro-Cuban religious divination. Pass shells from the user request (traditionally 4 or 16).",
            &[],
            json!({
                "shells": {
                    "type": "integer",
                    "description": "Number of shells (traditionally 4 or 16).",
                    "minimum": 1,
                    "maximum": 64,
                    "default": 4
                }
            }),
        ),
        tool(
            "chance_lots",
            "Draw lots (sortition) from a list. Returns the selected items in draw order. Use when performing sortition or random selection from a set of candidates. Pass items from the user request and count for how many to draw.",
            &["items"],
            json!({
                "items": {
                    "type": "array",
                    "description": "Items to draw from.",
                    "items": { "type": "string" },
                    "minItems": 1,
                    "maxItems": 100000
                },
                "count": {
                    "type": "integer",
                    "description": "Number of distinct lots to draw.",
                    "minimum": 1,
                    "default": 1
                }
            }),
        ),
        Tool {
            name: "chance_sources".to_string(),
            description: Some("List available randomness sources. Returns an array of source name strings (e.g. os-csprng, chacha20, xoshiro256**, mix, drand). Use when discovering which entropy backends the server supports. Takes no parameters.".to_string()),
            input_schema: json!({"type": "object", "properties": {}}),
        },
        Tool {
            name: "chance_health".to_string(),
            description: Some("Check the MCP server health. Returns server status and the configured randomness source. Use when verifying the server is responsive and operational. Takes no parameters.".to_string()),
            input_schema: json!({"type": "object", "properties": {}}),
        },
    ];
    tools.sort_by(|a, b| a.name.cmp(&b.name));
    tools
}

fn empty_args() -> Value {
    Value::Object(Map::new())
}

fn args_or_empty(params: &CallToolParams) -> Value {
    params.arguments.clone().unwrap_or_else(empty_args)
}

pub fn call_tool(params: &CallToolParams) -> CallToolResult {
    let args = args_or_empty(params);
    let result: Result<Value, String> = (|| match params.name.as_str() {
        "chance_roll" => {
            let req: RollRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::roll(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_flip" => {
            let req: FlipRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::flip(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_draw" => {
            let req: DrawRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::draw(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_pick" => {
            let req: ListRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::pick(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_shuffle" => {
            let req: ShuffleRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::shuffle(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_integer" => {
            let req: IntRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::integer(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_bytes" => {
            let req: BytesRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::bytes(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_uuid" => {
            let req: UuidRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::uuid(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_password" => {
            let req: PasswordRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::password(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_runes" => {
            let req: RunesRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::runes(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_iching" => {
            let req: IchingRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::iching(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_tarot" => {
            let req: TarotRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::tarot(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_dominoes" => {
            let req: DominoesRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::dominoes(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_roulette" => {
            let req: RouletteRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::roulette(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_lottery" => {
            let req: LotteryRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::lottery(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_knucklebones" => {
            let req: KnucklebonesRequest =
                serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::knucklebones(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_teetotum" => {
            let req: TeetotumRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::teetotum(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_cowrie" => {
            let req: CowrieRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::cowrie(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_lots" => {
            let req: ListRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            serde_json::to_value(services::lots(&req).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "chance_sources" => {
            serde_json::to_value(services::source_names()).map_err(|e| e.to_string())
        }
        "chance_health" => serde_json::to_value(services::health()).map_err(|e| e.to_string()),
        _ => Err(format!("unknown tool: {}", params.name)),
    })();

    match result {
        Ok(value) => {
            let text = serde_json::to_string_pretty(&value).unwrap_or_else(|e| e.to_string());
            CallToolResult::text(text)
        }
        Err(e) => CallToolResult::error(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W8: an over-cap argument must surface as a tool error result
    /// (`is_error == true`), never a panic. The services layer enforces the
    /// 1 MiB byte cap; the MCP dispatch must propagate that as an error.
    #[test]
    fn over_cap_bytes_returns_error_not_panic() {
        let params = CallToolParams {
            name: "chance_bytes".to_string(),
            arguments: Some(json!({ "count": 2_000_000u64, "encoding": "hex" })),
        };
        let result = call_tool(&params);
        assert!(
            result.is_error,
            "expected is_error=true for over-cap count, got: {:?}",
            result.content
        );
    }

    /// W8: an oversized count that still fits in `usize` but exceeds the cap is
    /// rejected by services and surfaced as an error, not a panic. This guards
    /// against a panic path ever being introduced in the dispatch.
    #[test]
    fn over_cap_runes_returns_error() {
        let params = CallToolParams {
            name: "chance_runes".to_string(),
            arguments: Some(json!({ "count": 99u64 })),
        };
        let result = call_tool(&params);
        assert!(result.is_error, "expected is_error=true for 99 runes");
    }

    /// Roulette honesty: the schema must not advertise `french` (which is not a
    /// distinct variant from `european`), while still listing american/european.
    #[test]
    fn roulette_schema_omits_french() {
        let tools = all_tools();
        let roulette = tools
            .iter()
            .find(|t| t.name == "chance_roulette")
            .expect("chance_roulette tool must exist");
        let schema = serde_json::to_string(&roulette.input_schema).unwrap();
        assert!(
            !schema.contains("french"),
            "roulette schema must not advertise french: {}",
            schema
        );
        assert!(schema.contains("european"));
        assert!(schema.contains("american"));
    }

    /// W8: advertised schema maximums must match the caps enforced in
    /// services/mod.rs so the schema is not misleading.
    #[test]
    fn schema_maximums_match_enforced_caps() {
        let tools = all_tools();
        let max_of = |tool_name: &str, field: &str| -> Option<u64> {
            let tool = tools.iter().find(|t| t.name == tool_name).unwrap();
            tool.input_schema["properties"][field]["maximum"].as_u64()
        };

        assert_eq!(max_of("chance_bytes", "count"), Some(1_048_576));
        assert_eq!(max_of("chance_password", "length"), Some(1024));
        assert_eq!(max_of("chance_flip", "times"), Some(1_000_000));
        assert_eq!(max_of("chance_draw", "count"), Some(52));
        assert_eq!(max_of("chance_tarot", "count"), Some(78));
        assert_eq!(max_of("chance_runes", "count"), Some(24));
        assert_eq!(max_of("chance_dominoes", "set"), Some(18));
        assert_eq!(max_of("chance_dominoes", "count"), Some(1000));
        assert_eq!(max_of("chance_lottery", "pool"), Some(99));
        assert_eq!(max_of("chance_lottery", "pick"), Some(20));
        assert_eq!(max_of("chance_knucklebones", "count"), Some(1000));
        assert_eq!(max_of("chance_cowrie", "shells"), Some(64));

        let max_items = |tool_name: &str| -> Option<u64> {
            let tool = tools.iter().find(|t| t.name == tool_name).unwrap();
            tool.input_schema["properties"]["items"]["maxItems"].as_u64()
        };
        assert_eq!(max_items("chance_pick"), Some(100_000));
        assert_eq!(max_items("chance_shuffle"), Some(100_000));
        assert_eq!(max_items("chance_lots"), Some(100_000));

        // Roll notation length cap matches the services notation_len cap.
        let roll = tools.iter().find(|t| t.name == "chance_roll").unwrap();
        assert_eq!(
            roll.input_schema["properties"]["notation"]["maxLength"].as_u64(),
            Some(256)
        );
    }
}
