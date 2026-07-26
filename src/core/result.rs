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
