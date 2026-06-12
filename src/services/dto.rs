use serde::{Deserialize, Serialize};

/// Common provenance metadata returned with every random result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub source: String,
    pub source_kind: String,
    pub timestamp: String,
    pub entropy_bits: f64,
    pub request_id: String,
    pub seed: Option<String>,
    pub latency_ms: f64,
}

/// Wrap any result with provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub result: T,
    pub provenance: Provenance,
}

/// Common request fields for source selection.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SourceRequest {
    pub source: Option<String>,
    pub seed: Option<String>,
}

// Dice
#[derive(Debug, Clone, Deserialize)]
pub struct RollRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default = "default_notation")]
    pub notation: String,
}

impl Default for RollRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            notation: default_notation(),
        }
    }
}

fn default_notation() -> String {
    "d20".to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct RollResultDto {
    pub total: i64,
    pub rolls: Vec<DieRollDto>,
    pub dropped: Vec<i64>,
    pub modifier_total: i64,
    pub success_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DieRollDto {
    pub value: i64,
    pub size: i64,
    pub exploded: bool,
    pub rerolled: bool,
}

// Coin
#[derive(Debug, Clone, Deserialize)]
pub struct FlipRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default = "default_one_u64")]
    pub times: u64,
}

impl Default for FlipRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            times: 1,
        }
    }
}

fn default_one_u64() -> u64 {
    1
}

// Cards
#[derive(Debug, Clone, Deserialize)]
pub struct DrawRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default = "default_five")]
    pub count: usize,
}

impl Default for DrawRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            count: 5,
        }
    }
}

fn default_five() -> usize {
    5
}

// Pick / Shuffle / Lots
#[derive(Debug, Clone, Deserialize)]
pub struct ListRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default = "default_one_usize")]
    pub count: usize,
}

impl Default for ListRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            items: Vec::new(),
            count: 1,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShuffleRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default)]
    pub items: Vec<String>,
}

impl Default for ShuffleRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            items: Vec::new(),
        }
    }
}

fn default_one_usize() -> usize {
    1
}

// Integer
#[derive(Debug, Clone, Deserialize)]
pub struct IntRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default = "default_min_int")]
    pub min: i64,
    #[serde(default = "default_max_int")]
    pub max: i64,
}

impl Default for IntRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            min: 1,
            max: 100,
        }
    }
}

fn default_min_int() -> i64 {
    1
}

fn default_max_int() -> i64 {
    100
}

// Bytes
#[derive(Debug, Clone, Deserialize)]
pub struct BytesRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default = "default_sixteen")]
    pub count: usize,
    #[serde(default = "default_hex")]
    pub encoding: String,
}

impl Default for BytesRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            count: 16,
            encoding: "hex".to_string(),
        }
    }
}

fn default_sixteen() -> usize {
    16
}

fn default_hex() -> String {
    "hex".to_string()
}

// UUID
#[derive(Debug, Clone, Deserialize)]
pub struct UuidRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default = "default_four_u8")]
    pub version: u8,
}

impl Default for UuidRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            version: 4,
        }
    }
}

fn default_four_u8() -> u8 {
    4
}

// Password
#[derive(Debug, Clone, Deserialize)]
pub struct PasswordRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default = "default_sixteen")]
    pub length: usize,
    #[serde(default = "default_true")]
    pub symbols: bool,
}

impl Default for PasswordRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            length: 16,
            symbols: true,
        }
    }
}

fn default_true() -> bool {
    true
}

// Runes
#[derive(Debug, Clone, Deserialize)]
pub struct RunesRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default = "default_one_usize")]
    pub count: usize,
}

impl Default for RunesRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            count: 1,
        }
    }
}

// I Ching
#[derive(Debug, Clone, Deserialize)]
pub struct IchingRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default = "default_coin")]
    pub method: String,
}

impl Default for IchingRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            method: "coin".to_string(),
        }
    }
}

fn default_coin() -> String {
    "coin".to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct IchingResultDto {
    pub primary: u8,
    pub primary_name: String,
    pub transformed: Option<u8>,
    pub method: String,
    pub lines: Vec<IchingLineDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IchingLineDto {
    pub value: u8,
    pub yang: bool,
    pub changing: bool,
}

// Tarot
#[derive(Debug, Clone, Deserialize)]
pub struct TarotRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default = "default_one_usize")]
    pub count: usize,
}

impl Default for TarotRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            count: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TarotCardDto {
    pub name: String,
    pub upright: bool,
}

// Dominoes
#[derive(Debug, Clone, Deserialize)]
pub struct DominoesRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default = "default_six_u8")]
    pub set: u8,
    #[serde(default = "default_one_usize")]
    pub count: usize,
}

impl Default for DominoesRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            set: 6,
            count: 1,
        }
    }
}

fn default_six_u8() -> u8 {
    6
}

#[derive(Debug, Clone, Serialize)]
pub struct DominoDto {
    pub left: u8,
    pub right: u8,
}

// Roulette
#[derive(Debug, Clone, Deserialize)]
pub struct RouletteRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default = "default_european")]
    pub variant: String,
}

impl Default for RouletteRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            variant: "european".to_string(),
        }
    }
}

fn default_european() -> String {
    "european".to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct RouletteResultDto {
    pub number: u8,
    pub color: String,
    pub variant: String,
    pub house_edge_percent: f64,
}

// Lottery
#[derive(Debug, Clone, Deserialize)]
pub struct LotteryRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default = "default_forty_nine")]
    pub pool: u8,
    #[serde(default = "default_six")]
    pub pick: usize,
    pub bonus_pool: Option<u8>,
}

impl Default for LotteryRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            pool: 49,
            pick: 6,
            bonus_pool: None,
        }
    }
}

fn default_forty_nine() -> u8 {
    49
}

fn default_six() -> usize {
    6
}

#[derive(Debug, Clone, Serialize)]
pub struct LotteryResultDto {
    pub numbers: Vec<u8>,
    pub bonus: Option<u8>,
}

// Knucklebones
#[derive(Debug, Clone, Deserialize)]
pub struct KnucklebonesRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default = "default_four_usize")]
    pub count: usize,
}

impl Default for KnucklebonesRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            count: 4,
        }
    }
}

fn default_four_usize() -> usize {
    4
}

// Teetotum
#[derive(Debug, Clone, Deserialize)]
pub struct TeetotumRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default)]
    pub dreidel: bool,
}

impl Default for TeetotumRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            dreidel: false,
        }
    }
}

// Cowrie
#[derive(Debug, Clone, Deserialize)]
pub struct CowrieRequest {
    #[serde(flatten)]
    pub source: SourceRequest,
    #[serde(default = "default_four_usize")]
    pub shells: usize,
}

impl Default for CowrieRequest {
    fn default() -> Self {
        Self {
            source: SourceRequest::default(),
            shells: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CowrieResultDto {
    pub shells: usize,
    pub open_count: u8,
    pub meaning: String,
}
