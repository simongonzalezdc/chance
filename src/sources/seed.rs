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
/// Build a CSPRNG from either an explicit user seed or full OS entropy.
///
/// CSPRNG-class sources (`SourceKind::Csprng`) require their full seed width
/// to resist state-recovery attacks. When the user supplies an explicit seed
/// (CLI `--seed` / `SourceRequest.seed`) determinism is a *feature*, so we
/// keep the existing `seed_from_u64` SplitMix KDF expansion. When **no** seed
/// is supplied the CSPRNG is seeded directly from OS entropy via
/// `SeedableRng::from_entropy()` — which materialises the RNG's full seed
/// width (256 bits for ChaCha20) — instead of collapsing a single 64-bit OS
/// sample through SplitMix. Non-CSPRNG PRNGs should keep using [`seeded_rng`].
pub fn seeded_csprng<R: SeedableRng>(
    seed: Option<&str>,
) -> Result<(R, Option<String>), SourceError> {
    match seed {
        Some(s) => {
            let raw = parse_seed(s)?;
            let seed_str = format!("0x{:016x}", raw);
            Ok((R::seed_from_u64(raw), Some(seed_str)))
        }
        None => {
            // Full-width OS entropy — never collapse a CSPRNG seed to 64 bits.
            Ok((R::from_entropy(), None))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn explicit_seed_is_reproducible() {
        // Determinism is a feature of the explicit-seed path.
        let (mut a, _): (ChaCha20Rng, _) = seeded_csprng(Some("0xDEADBEEF")).unwrap();
        let (mut b, _): (ChaCha20Rng, _) = seeded_csprng(Some("0xDEADBEEF")).unwrap();

        let mut ab = [0u8; 32];
        let mut bb = [0u8; 32];
        a.fill_bytes(&mut ab);
        b.fill_bytes(&mut bb);
        assert_eq!(ab, bb, "same explicit seed must yield identical output");
    }

    #[test]
    fn explicit_seed_records_seed_string() {
        let (_rng, seed_str) = seeded_csprng::<ChaCha20Rng>(Some("42")).unwrap();
        assert_eq!(seed_str.as_deref(), Some("0x000000000000002a"));
    }

    #[test]
    fn unseeded_csprng_uses_full_entropy() {
        // Two unseeded CSPRNGs must not collapse to the same 64-bit state.
        // from_entropy is non-deterministic; assert divergence across a few
        // constructions (probability of accidental all-equal is negligible).
        let mut firsts: Vec<[u8; 32]> = Vec::new();
        for _ in 0..8 {
            let (mut rng, seed_str): (ChaCha20Rng, _) = seeded_csprng(None).unwrap();
            assert!(
                seed_str.is_none(),
                "unseeded CSPRNG must not expose a deterministic seed string"
            );
            let mut buf = [0u8; 32];
            rng.fill_bytes(&mut buf);
            firsts.push(buf);
        }
        let all_equal = firsts.windows(2).all(|w| w[0] == w[1]);
        assert!(!all_equal, "from_entropy must produce divergent streams");
    }

    #[test]
    fn empty_explicit_seed_is_rejected() {
        let res = seeded_csprng::<ChaCha20Rng>(Some(""));
        assert!(res.is_err(), "empty seed string must error");
    }
}
