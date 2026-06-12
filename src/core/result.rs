use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The kind of randomness source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    /// Deterministic pseudo-random number generator.
    Prng,
    /// Cryptographically secure pseudo-random number generator.
    Csprng,
    /// True random number generator (physical entropy).
    Trng,
    /// Quantum random number generator.
    Qrng,
    /// Distributed / publicly verifiable beacon.
    Beacon,
    /// Emulated traditional randomizer (dice, cards, etc.).
    Traditional,
}

impl std::fmt::Display for SourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceKind::Prng => write!(f, "prng"),
            SourceKind::Csprng => write!(f, "csprng"),
            SourceKind::Trng => write!(f, "trng"),
            SourceKind::Qrng => write!(f, "qrng"),
            SourceKind::Beacon => write!(f, "beacon"),
            SourceKind::Traditional => write!(f, "traditional"),
        }
    }
}

/// Health status of a source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceHealth {
    Healthy,
    Degraded,
    Unavailable,
}

/// A single randomness outcome with full provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChanceResult<T> {
    pub result: T,
    pub source: String,
    pub source_kind: SourceKind,
    pub timestamp: DateTime<Utc>,
    pub entropy_bits: f64,
    pub request_id: String,
    pub seed: Option<String>,
    pub latency_ms: f64,
}

impl<T> ChanceResult<T> {
    pub fn new(result: T, source: &str, source_kind: SourceKind) -> Self {
        Self {
            result,
            source: source.to_string(),
            source_kind,
            timestamp: Utc::now(),
            entropy_bits: 0.0,
            request_id: generate_request_id(),
            seed: None,
            latency_ms: 0.0,
        }
    }
}

fn generate_request_id() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    format!("req_{}", hex::encode(&bytes))
}

// Minimal local hex encoder until we add a hex crate.
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            s.push(HEX[(b >> 4) as usize] as char);
            s.push(HEX[(b & 0x0f) as usize] as char);
        }
        s
    }
}
