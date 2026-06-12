use crate::core::{Source, SourceError};

/// List all built-in source names.
pub fn source_names() -> &'static [&'static str] {
    &[
        "os-csprng",
        "chacha20",
        "xoshiro256**",
        "xoshiro256++",
        "xoroshiro128**",
        "pcg64",
        "pcg64mcg",
        "splitmix64",
        #[cfg(feature = "external-sources")]
        "drand",
        #[cfg(target_arch = "x86_64")]
        "rdrand",
        #[cfg(target_arch = "x86_64")]
        "rdseed",
        #[cfg(feature = "mixing")]
        "mix",
    ]
}

/// Create a source by name.
///
/// Source names:
/// - `os-csprng` — OS CSPRNG
/// - `chacha20` — ChaCha20 deterministic CSPRNG
/// - `xoshiro256**`, `xoshiro256++`, `xoroshiro128**`
/// - `pcg64`, `pcg64mcg`
/// - `splitmix64`
/// - `drand` — distributed public randomness beacon (with `external-sources` feature)
/// - `rdrand` — x86_64 on-chip hardware RNG (RDRAND instruction)
/// - `rdseed` — x86_64 on-chip hardware entropy source (RDSEED instruction)
/// - `mix:source1,source2,...` (with `mixing` feature)
pub fn create_source(name: &str, seed: Option<&str>) -> Result<Box<dyn Source>, SourceError> {
    #[cfg(feature = "mixing")]
    if let Some(rest) = name.strip_prefix("mix:") {
        return create_mix_source(rest, seed);
    }

    match name {
        "os-csprng" => Ok(Box::new(crate::sources::OsCsprng::new())),
        "chacha20" => crate::sources::chacha20(seed),
        "xoshiro256**" => crate::sources::xoshiro256_star_star(seed),
        "xoshiro256++" => crate::sources::xoshiro256_plus_plus(seed),
        "xoroshiro128**" => crate::sources::xoroshiro128_star_star(seed),
        "pcg64" => crate::sources::pcg64(seed),
        "pcg64mcg" => crate::sources::pcg64mcg(seed),
        "splitmix64" => crate::sources::splitmix64(seed),
        #[cfg(feature = "external-sources")]
        "drand" => Ok(Box::new(crate::sources::DrandSource::new())),
        #[cfg(target_arch = "x86_64")]
        "rdrand" => Ok(Box::new(crate::sources::RdrandSource::new()?)),
        #[cfg(target_arch = "x86_64")]
        "rdseed" => Ok(Box::new(crate::sources::RdseedSource::new()?)),
        _ => Err(SourceError::InvalidSource(name.to_string())),
    }
}

#[cfg(feature = "mixing")]
fn create_mix_source(spec: &str, seed: Option<&str>) -> Result<Box<dyn Source>, SourceError> {
    let names: Vec<&str> = spec.split(',').collect();
    if names.len() < 2 {
        return Err(SourceError::InvalidSource(
            "mix: requires at least two source names".to_string(),
        ));
    }
    let mut sources: Vec<Box<dyn Source>> = Vec::new();
    for name in names {
        sources.push(create_source(name, seed)?);
    }
    Ok(Box::new(crate::core::Mixer::new(
        sources,
        &format!("mix:{}", spec),
    )))
}
