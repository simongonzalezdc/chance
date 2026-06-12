use crate::services::dto::*;
use crate::services;
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
    let mut props = source_properties()
        .as_object()
        .cloned()
        .unwrap_or_default();
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
        input_schema: schema(&req.iter().map(|s| s.as_str()).collect::<Vec<_>>(), properties),
    }
}

pub fn all_tools() -> Vec<Tool> {
    vec![
        tool(
            "chance_roll",
            "Roll dice using standard RPG notation (e.g. d20, 4d6kh3, 2d20adv).",
            &["notation"],
            json!({
                "notation": {
                    "type": "string",
                    "description": "Dice notation expression.",
                    "default": "d20"
                }
            }),
        ),
        tool(
            "chance_flip",
            "Flip one or more coins.",
            &[],
            json!({
                "times": {
                    "type": "integer",
                    "description": "Number of flips.",
                    "minimum": 1,
                    "default": 1
                }
            }),
        ),
        tool(
            "chance_draw",
            "Draw cards from a shuffled 52-card deck.",
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
            "Pick one or more distinct winners from a list.",
            &["items"],
            json!({
                "items": {
                    "type": "array",
                    "description": "Items to choose from.",
                    "items": { "type": "string" },
                    "minItems": 1
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
            "Shuffle a list of items.",
            &["items"],
            json!({
                "items": {
                    "type": "array",
                    "description": "Items to shuffle.",
                    "items": { "type": "string" },
                    "minItems": 1
                }
            }),
        ),
        tool(
            "chance_integer",
            "Generate a random integer in an inclusive range.",
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
            "Generate random bytes, encoded as hex or base64.",
            &[],
            json!({
                "count": {
                    "type": "integer",
                    "description": "Number of bytes.",
                    "minimum": 1,
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
            "Generate a random UUID (v4 or v7).",
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
            "Generate a random password.",
            &[],
            json!({
                "length": {
                    "type": "integer",
                    "description": "Password length.",
                    "minimum": 1,
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
            "Draw Elder Futhark runes.",
            &[],
            json!({
                "count": {
                    "type": "integer",
                    "description": "Number of runes to draw.",
                    "minimum": 1,
                    "default": 1
                }
            }),
        ),
        tool(
            "chance_iching",
            "Cast an I Ching hexagram using coin or yarrow method.",
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
            "Draw Tarot cards (Major Arcana + Minor Arcana) with upright/reversed orientation.",
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
            "Draw dominoes from a double-n set.",
            &[],
            json!({
                "set": {
                    "type": "integer",
                    "description": "Double-n set size (e.g. 6 for double-six).",
                    "minimum": 0,
                    "default": 6
                },
                "count": {
                    "type": "integer",
                    "description": "Number of dominoes to draw.",
                    "minimum": 1,
                    "default": 1
                }
            }),
        ),
        tool(
            "chance_roulette",
            "Spin a roulette wheel.",
            &[],
            json!({
                "variant": {
                    "type": "string",
                    "description": "Roulette variant.",
                    "enum": ["european", "american", "french"],
                    "default": "european"
                }
            }),
        ),
        tool(
            "chance_lottery",
            "Draw lottery numbers from a pool.",
            &[],
            json!({
                "pool": {
                    "type": "integer",
                    "description": "Highest numbered ball in the pool.",
                    "minimum": 1,
                    "default": 49
                },
                "pick": {
                    "type": "integer",
                    "description": "How many numbers to draw.",
                    "minimum": 1,
                    "default": 6
                },
                "bonus_pool": {
                    "type": ["integer", "null"],
                    "description": "Optional separate bonus ball pool size."
                }
            }),
        ),
        tool(
            "chance_knucklebones",
            "Cast knucklebones / astragali.",
            &[],
            json!({
                "count": {
                    "type": "integer",
                    "description": "Number of bones to cast.",
                    "minimum": 1,
                    "default": 4
                }
            }),
        ),
        tool(
            "chance_teetotum",
            "Spin a teetotum or dreidel.",
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
            "Cast cowrie shells (Santería / Ifá divination).",
            &[],
            json!({
                "shells": {
                    "type": "integer",
                    "description": "Number of shells (traditionally 4 or 16).",
                    "minimum": 1,
                    "default": 4
                }
            }),
        ),
        tool(
            "chance_lots",
            "Draw lots (sortition) from a list.",
            &["items"],
            json!({
                "items": {
                    "type": "array",
                    "description": "Items to draw from.",
                    "items": { "type": "string" },
                    "minItems": 1
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
            description: Some("List available randomness sources.".to_string()),
            input_schema: json!({"type": "object", "properties": {}}),
        },
        Tool {
            name: "chance_health".to_string(),
            description: Some("Check server health.".to_string()),
            input_schema: json!({"type": "object", "properties": {}}),
        },
    ]
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
        "chance_sources" => serde_json::to_value(services::source_names()).map_err(|e| e.to_string()),
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
