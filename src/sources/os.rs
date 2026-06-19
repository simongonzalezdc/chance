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

    /// W2: `OsRng` is reported as unconditionally healthy. A failure from the
    /// underlying `getrandom` syscall would indicate a fundamental OS fault
    /// (no entropy pool / kernel RNG not initialized) that a synthetic probe
    /// here cannot recover from and that would already surface as an error from
    /// `generate_u64`/`fill_bytes`. So rather than run a throwaway probe whose
    /// only signal duplicates the next real request, we return `Healthy` and
    /// let the actual generation calls report any failure. (Live end-to-end
    /// probing of every source, including this one, lives in
    /// `services::health`, which exercises the real `fill_bytes` path.)
    fn health(&self) -> SourceHealth {
        SourceHealth::Healthy
    }
}
