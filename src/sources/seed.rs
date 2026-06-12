use crate::core::SourceError;
use rand::RngCore;
use rand::SeedableRng;

/// Parse a seed string into a fixed 64-bit seed.
///
/// Supports:
/// - Decimal integer: `12345`
/// - Hex integer: `0xCAFE`
/// - Arbitrary string (hashed with SipHash-1-3 for determinism)
pub fn parse_seed(seed: &str) -> Result<u64, SourceError> {
    if seed.is_empty() {
        return Err(SourceError::GenerationFailed(
            "seed string is empty".to_string(),
        ));
    }

    if let Some(hex) = seed.strip_prefix("0x").or_else(|| seed.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).map_err(|e| {
            SourceError::GenerationFailed(format!("invalid hex seed '{}': {}", seed, e))
        })
    } else if let Some(bin) = seed.strip_prefix("0b").or_else(|| seed.strip_prefix("0B")) {
        u64::from_str_radix(bin, 2).map_err(|e| {
            SourceError::GenerationFailed(format!("invalid binary seed '{}': {}", seed, e))
        })
    } else {
        seed.parse::<u64>().or_else(|_| {
            // Fall back to a deterministic hash of the string.
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            seed.hash(&mut hasher);
            Ok(hasher.finish())
        })
    }
}

/// Create a seed from OS entropy.
pub fn random_seed() -> u64 {
    let mut buf = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut buf);
    u64::from_le_bytes(buf)
}

/// Helper to build a seeded RNG from a seed string.
pub fn seeded_rng<R: SeedableRng>(seed: Option<&str>) -> Result<(R, Option<String>), SourceError> {
    let raw = match seed {
        Some(s) => parse_seed(s)?,
        None => random_seed(),
    };
    let seed_str = format!("0x{:016x}", raw);
    Ok((R::seed_from_u64(raw), Some(seed_str)))
}
