use crate::core::{RngSource, Source, SourceKind};
use crate::sources::seed::seeded_csprng;
use rand_chacha::ChaCha20Rng;

/// ChaCha20-based CSPRNG.
///
/// When an explicit seed is supplied the source is deterministic (reproducible
/// output for the same seed). When no seed is supplied it is seeded from full
/// 256-bit OS entropy via `from_entropy()` — see [`seed::seeded_csprng`].
pub fn chacha20(seed: Option<&str>) -> Result<Box<dyn Source>, crate::core::SourceError> {
    let (rng, seed_str) = seeded_csprng::<ChaCha20Rng>(seed)?;
    let mut src = RngSource::new(rng, "chacha20", SourceKind::Csprng);
    // Only expose a seed string when the source was constructed deterministically;
    // an entropy-seeded CSPRNG has no user-visible seed.
    if let Some(s) = seed_str {
        src = src.with_seed(s);
    }
    Ok(Box::new(src))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_chacha20_is_reproducible() {
        // Determinism is a feature when the user supplies an explicit seed.
        let mut a = chacha20(Some("0xCAFE")).unwrap();
        let mut b = chacha20(Some("0xCAFE")).unwrap();
        assert_eq!(a.generate_u64().unwrap(), b.generate_u64().unwrap());
        assert_eq!(
            a.seed().as_deref(),
            Some("0x000000000000cafe"),
            "explicit seed must be surfaced for reproducibility"
        );
    }

    #[test]
    fn unseeded_chacha20_is_non_deterministic() {
        // Two unseeded chacha20 sources must not collapse to a shared 64-bit
        // state. Pull a few u64s from each of several constructions and assert
        // they are not all identical.
        let mut firsts: Vec<u64> = Vec::new();
        for _ in 0..8 {
            let mut s = chacha20(None).unwrap();
            assert!(
                s.seed().is_none(),
                "entropy-seeded CSPRNG must not report a deterministic seed"
            );
            firsts.push(s.generate_u64().unwrap());
        }
        let all_equal = firsts.windows(2).all(|w| w[0] == w[1]);
        assert!(!all_equal, "unseeded chacha20 streams must diverge");
    }

    #[test]
    fn chacha20_is_csprng_kind() {
        let s = chacha20(None).unwrap();
        assert_eq!(s.kind(), crate::core::SourceKind::Csprng);
    }
}
