use crate::core::{RngSource, Source, SourceKind};
use crate::sources::seed::seeded_rng;
use rand_chacha::ChaCha20Rng;

/// ChaCha20-based deterministic CSPRNG.
pub fn chacha20(seed: Option<&str>) -> Result<Box<dyn Source>, crate::core::SourceError> {
    let (rng, seed_str) = seeded_rng::<ChaCha20Rng>(seed)?;
    Ok(Box::new(
        RngSource::new(rng, "chacha20", SourceKind::Csprng).with_seed(seed_str.unwrap_or_default()),
    ))
}
