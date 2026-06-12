use crate::core::{Source, SourceError, SourceHealth, SourceKind};

const MAX_RETRIES: u32 = 10;

/// x86_64 on-chip hardware RNG using the `RDRAND` instruction.
///
/// RDRAND reads from a hardware RNG that is seeded by an on-chip entropy
/// source (thermal noise on Intel/AMD designs). It is a true random number
/// generator from the perspective of this library: its output is
/// non-deterministic and does not require a seed.
pub struct RdrandSource {
    supported: bool,
}

impl RdrandSource {
    pub fn new() -> Result<Self, SourceError> {
        let supported = is_x86_feature_detected!("rdrand");
        if !supported {
            return Err(SourceError::Unavailable(
                "RDRAND instruction is not supported on this CPU".to_string(),
            ));
        }
        Ok(Self { supported })
    }
}

impl Source for RdrandSource {
    fn name(&self) -> String {
        "rdrand".to_string()
    }

    fn kind(&self) -> SourceKind {
        SourceKind::Trng
    }

    fn generate_u64(&mut self) -> Result<u64, SourceError> {
        rdrand64().ok_or_else(|| {
            SourceError::GenerationFailed(
                "RDRAND failed to produce a value after retries".to_string(),
            )
        })
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), SourceError> {
        let mut remaining = &mut buf[..];
        while !remaining.is_empty() {
            let val = self.generate_u64()?;
            let chunk = &val.to_le_bytes()[..remaining.len().min(8)];
            let len = chunk.len();
            remaining[..len].copy_from_slice(chunk);
            remaining = &mut remaining[len..];
        }
        Ok(())
    }

    fn health(&self) -> SourceHealth {
        if self.supported {
            SourceHealth::Healthy
        } else {
            SourceHealth::Unavailable
        }
    }
}

/// x86_64 on-chip hardware entropy source using the `RDSEED` instruction.
///
/// RDSEED returns raw entropy straight from the hardware entropy source,
/// bypassing the deterministic random bit generator that RDRAND uses.
/// It is intended primarily for seeding other generators, but works as a
/// standalone true random source here.
pub struct RdseedSource {
    supported: bool,
}

impl RdseedSource {
    pub fn new() -> Result<Self, SourceError> {
        let supported = is_x86_feature_detected!("rdseed");
        if !supported {
            return Err(SourceError::Unavailable(
                "RDSEED instruction is not supported on this CPU".to_string(),
            ));
        }
        Ok(Self { supported })
    }
}

impl Source for RdseedSource {
    fn name(&self) -> String {
        "rdseed".to_string()
    }

    fn kind(&self) -> SourceKind {
        SourceKind::Trng
    }

    fn generate_u64(&mut self) -> Result<u64, SourceError> {
        rdseed64().ok_or_else(|| {
            SourceError::GenerationFailed(
                "RDSEED failed to produce a value after retries".to_string(),
            )
        })
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), SourceError> {
        let mut remaining = &mut buf[..];
        while !remaining.is_empty() {
            let val = self.generate_u64()?;
            let chunk = &val.to_le_bytes()[..remaining.len().min(8)];
            let len = chunk.len();
            remaining[..len].copy_from_slice(chunk);
            remaining = &mut remaining[len..];
        }
        Ok(())
    }

    fn health(&self) -> SourceHealth {
        if self.supported {
            SourceHealth::Healthy
        } else {
            SourceHealth::Unavailable
        }
    }
}

#[target_feature(enable = "rdrand")]
unsafe fn rdrand64_step() -> Option<u64> {
    let mut val = 0u64;
    if std::arch::x86_64::_rdrand64_step(&mut val) == 1 {
        Some(val)
    } else {
        None
    }
}

fn rdrand64() -> Option<u64> {
    for _ in 0..MAX_RETRIES {
        if let Some(v) = unsafe { rdrand64_step() } {
            return Some(v);
        }
    }
    None
}

#[target_feature(enable = "rdseed")]
unsafe fn rdseed64_step() -> Option<u64> {
    let mut val = 0u64;
    if std::arch::x86_64::_rdseed64_step(&mut val) == 1 {
        Some(val)
    } else {
        None
    }
}

fn rdseed64() -> Option<u64> {
    for _ in 0..MAX_RETRIES {
        if let Some(v) = unsafe { rdseed64_step() } {
            return Some(v);
        }
    }
    None
}
