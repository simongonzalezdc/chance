use crate::core::{Source, SourceError, SourceHealth, SourceKind};
use rand::rngs::OsRng;
use rand::RngCore;

/// The operating system's cryptographically secure random number generator.
///
/// Uses `getrandom` / `/dev/urandom` / `arc4random` / `BCryptGenRandom`
/// depending on the platform.
pub struct OsCsprng;

impl OsCsprng {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OsCsprng {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for OsCsprng {
    fn name(&self) -> String {
        "os-csprng".to_string()
    }

    fn kind(&self) -> SourceKind {
        SourceKind::Csprng
    }

    fn generate_u64(&mut self) -> Result<u64, SourceError> {
        let mut buf = [0u8; 8];
        OsRng.fill_bytes(&mut buf);
        Ok(u64::from_le_bytes(buf))
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), SourceError> {
        OsRng.fill_bytes(buf);
        Ok(())
    }

    fn health(&self) -> SourceHealth {
        SourceHealth::Healthy
    }
}
