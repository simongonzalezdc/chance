use crate::core::{RngSource, Source, SourceKind};
use crate::sources::seed::seeded_rng;
use rand_pcg::Pcg64;

/// PCG64 — 128-bit LCG with XSL output; NumPy uses the DXSM variant.
pub fn pcg64(seed: Option<&str>) -> Result<Box<dyn Source>, crate::core::SourceError> {
    let (rng, seed_str) = seeded_rng::<Pcg64>(seed)?;
    Ok(Box::new(
        RngSource::new(rng, "pcg64", SourceKind::Prng)
            .with_seed(seed_str.unwrap_or_default().to_string()),
    ))
}

/// PCG64MCG — multiplicative congruential variant, faster.
pub fn pcg64mcg(seed: Option<&str>) -> Result<Box<dyn Source>, crate::core::SourceError> {
    let (rng, seed_str) = seeded_rng::<rand_pcg::Pcg64Mcg>(seed)?;
    Ok(Box::new(
        RngSource::new(rng, "pcg64mcg", SourceKind::Prng)
            .with_seed(seed_str.unwrap_or_default().to_string()),
    ))
}
