use crate::services;
use crate::services::dto::*;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Popup {
    None,
    Source,
    Seed,
    Viz,
}

pub type MethodRun = dyn Fn(&App) -> Result<Value, String>;

pub struct MethodItem {
    pub name: &'static str,
    pub description: &'static str,
    pub run: Box<MethodRun>,
}

pub struct App {
    pub methods: Vec<MethodItem>,
    pub selected_method: usize,
    pub sources: Vec<String>,
    pub selected_source: usize,
    pub seed: String,
    pub popup: Popup,
    pub popup_selection: usize,
    pub last_result: Option<serde_json::Value>,
    pub last_provenance: Option<serde_json::Value>,
    pub status_message: Option<String>,
    pub should_quit: bool,
    pub tick: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            methods: build_methods(),
            selected_method: 0,
            sources: services::source_names(),
            selected_source: 0,
            seed: String::new(),
            popup: Popup::None,
            popup_selection: 0,
            last_result: None,
            last_provenance: None,
            status_message: None,
            should_quit: false,
            tick: 0,
        }
    }

    fn set_result(&mut self, value: serde_json::Value) {
        self.last_provenance = value.get("provenance").cloned();
        self.last_result = value.get("result").cloned().or(Some(value));
    }

    pub fn source_request(&self) -> SourceRequest {
        SourceRequest {
            source: Some(self.sources[self.selected_source].clone()),
            seed: if self.seed.is_empty() {
                None
            } else {
                Some(self.seed.clone())
            },
        }
    }

    pub fn run_selected(&mut self) {
        let item = &self.methods[self.selected_method];
        match (item.run)(self) {
            Ok(value) => {
                self.set_result(value);
                self.status_message = None;
            }
            Err(e) => {
                self.status_message = Some(format!("error: {}", e));
            }
        }
    }

    pub fn current_source_name(&self) -> &str {
        &self.sources[self.selected_source]
    }
}

fn demo_items() -> Vec<String> {
    vec![
        "Alice".to_string(),
        "Bob".to_string(),
        "Carol".to_string(),
        "Diana".to_string(),
    ]
}

fn build_methods() -> Vec<MethodItem> {
    vec![
        MethodItem {
            name: "roll",
            description: "Roll dice (default d20)",
            run: Box::new(|app| {
                let req = RollRequest {
                    source: app.source_request(),
                    notation: "d20".to_string(),
                };
                serde_json::to_value(services::roll(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "flip",
            description: "Flip a coin",
            run: Box::new(|app| {
                let req = FlipRequest {
                    source: app.source_request(),
                    times: 1,
                };
                serde_json::to_value(services::flip(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "draw",
            description: "Draw 5 cards",
            run: Box::new(|app| {
                let req = DrawRequest {
                    source: app.source_request(),
                    count: 5,
                };
                serde_json::to_value(services::draw(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "pick",
            description: "Pick one from a demo list",
            run: Box::new(|app| {
                let req = ListRequest {
                    source: app.source_request(),
                    items: demo_items(),
                    count: 1,
                };
                serde_json::to_value(services::pick(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "shuffle",
            description: "Shuffle a demo list",
            run: Box::new(|app| {
                let req = ShuffleRequest {
                    source: app.source_request(),
                    items: demo_items(),
                };
                serde_json::to_value(services::shuffle(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "integer",
            description: "Random integer 1..=100",
            run: Box::new(|app| {
                let req = IntRequest {
                    source: app.source_request(),
                    min: 1,
                    max: 100,
                };
                serde_json::to_value(services::integer(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "bytes",
            description: "16 random bytes (hex)",
            run: Box::new(|app| {
                let req = BytesRequest {
                    source: app.source_request(),
                    count: 16,
                    encoding: "hex".to_string(),
                };
                serde_json::to_value(services::bytes(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "uuid",
            description: "Generate a UUIDv4",
            run: Box::new(|app| {
                let req = UuidRequest {
                    source: app.source_request(),
                    version: 4,
                };
                serde_json::to_value(services::uuid(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "password",
            description: "Generate a 16-char password",
            run: Box::new(|app| {
                let req = PasswordRequest {
                    source: app.source_request(),
                    length: 16,
                    symbols: true,
                };
                serde_json::to_value(services::password(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "runes",
            description: "Draw an Elder Futhark rune",
            run: Box::new(|app| {
                let req = RunesRequest {
                    source: app.source_request(),
                    count: 1,
                };
                serde_json::to_value(services::runes(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "iching",
            description: "Cast I Ching with coins",
            run: Box::new(|app| {
                let req = IchingRequest {
                    source: app.source_request(),
                    method: "coin".to_string(),
                };
                serde_json::to_value(services::iching(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "tarot",
            description: "Draw a Tarot card",
            run: Box::new(|app| {
                let req = TarotRequest {
                    source: app.source_request(),
                    count: 1,
                };
                serde_json::to_value(services::tarot(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "dominoes",
            description: "Draw a double-six domino",
            run: Box::new(|app| {
                let req = DominoesRequest {
                    source: app.source_request(),
                    set: 6,
                    count: 1,
                };
                serde_json::to_value(services::dominoes(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "roulette",
            description: "Spin a European roulette wheel",
            run: Box::new(|app| {
                let req = RouletteRequest {
                    source: app.source_request(),
                    variant: "european".to_string(),
                };
                serde_json::to_value(services::roulette(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "lottery",
            description: "Draw 6/49 lottery numbers",
            run: Box::new(|app| {
                let req = LotteryRequest {
                    source: app.source_request(),
                    pool: 49,
                    pick: 6,
                    bonus_pool: None,
                };
                serde_json::to_value(services::lottery(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "knucklebones",
            description: "Cast 4 knucklebones",
            run: Box::new(|app| {
                let req = KnucklebonesRequest {
                    source: app.source_request(),
                    count: 4,
                };
                serde_json::to_value(services::knucklebones(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "teetotum",
            description: "Spin a teetotum",
            run: Box::new(|app| {
                let req = TeetotumRequest {
                    source: app.source_request(),
                    dreidel: false,
                };
                serde_json::to_value(services::teetotum(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "cowrie",
            description: "Cast 4 cowrie shells",
            run: Box::new(|app| {
                let req = CowrieRequest {
                    source: app.source_request(),
                    shells: 4,
                };
                serde_json::to_value(services::cowrie(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
        MethodItem {
            name: "lots",
            description: "Draw one lot from a demo list",
            run: Box::new(|app| {
                let req = ListRequest {
                    source: app.source_request(),
                    items: demo_items(),
                    count: 1,
                };
                serde_json::to_value(services::lots(&req).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())
            }),
        },
    ]
}
