use crate::core::{RngSource, Source, SourceKind};
use crate::sources::seed::seeded_rng;
use rand_xoshiro::Xoshiro256StarStar;

/// xoshiro256** — fast, statistically excellent PRNG.
pub fn xoshiro256_star_star(
    seed: Option<&str>,
) -> Result<Box<dyn Source>, crate::core::SourceError> {
    let (rng, seed_str) = seeded_rng::<Xoshiro256StarStar>(seed)?;
    Ok(Box::new(
        RngSource::new(rng, "xoshiro256**", SourceKind::Prng)
            .with_seed(seed_str.unwrap_or_default()),
    ))
}

/// xoshiro256++ — slightly faster sibling.
pub fn xoshiro256_plus_plus(
    seed: Option<&str>,
) -> Result<Box<dyn Source>, crate::core::SourceError> {
    let (rng, seed_str) = seeded_rng::<rand_xoshiro::Xoshiro256PlusPlus>(seed)?;
    Ok(Box::new(
        RngSource::new(rng, "xoshiro256++", SourceKind::Prng)
            .with_seed(seed_str.unwrap_or_default()),
    ))
}

/// xoroshiro128** — smaller-state variant.
pub fn xoroshiro128_star_star(
    seed: Option<&str>,
) -> Result<Box<dyn Source>, crate::core::SourceError> {
    let (rng, seed_str) = seeded_rng::<rand_xoshiro::Xoroshiro128StarStar>(seed)?;
    Ok(Box::new(
        RngSource::new(rng, "xoroshiro128**", SourceKind::Prng)
            .with_seed(seed_str.unwrap_or_default()),
    ))
}
