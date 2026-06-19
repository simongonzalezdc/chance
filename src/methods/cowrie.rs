use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;

#[derive(Debug, Clone)]
pub struct CowrieResult {
    pub shells: usize,
    pub open_count: u32,
    pub meaning: &'static str,
}

impl std::fmt::Display for CowrieResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} shells, {} open ({})", self.shells, self.open_count, self.meaning)
    }
}

/// Cast `shells` cowrie shells. Each shell is open (mouth up) with probability ~0.5.
/// Traditional Santería / Ifá divination uses 4 or 16 shells.
pub fn cast_cowrie(
    source: &mut dyn Source,
    shells: usize,
) -> Result<CowrieResult, crate::core::SourceError> {
    let mut open_count = 0u32;
    for _ in 0..shells {
        if uniform_u64_inclusive(source, 0, 1)? == 1 {
            open_count += 1;
        }
    }

    let meaning = match shells {
        4 => match open_count {
            0 => "okana (all closed)",
            1 => "okana meji / one open",
            2 => "ejife / two open",
            3 => "eyila / three open",
            4 => "alafia / all open",
            _ => "unknown",
        },
        16 => match open_count {
            0..=5 => "mostly closed",
            6..=10 => "balanced",
            _ => "mostly open",
        },
        _ => "custom cast",
    };

    Ok(CowrieResult {
        shells,
        open_count,
        meaning,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{SourceError, SourceHealth, SourceKind};

    /// Deterministic source that always yields the same `u64` (here `1`, so every
    /// shell comes up open). `uniform_u64_inclusive(src, 0, 1)` maps any non-zero
    /// u64 to `1`, giving an open shell.
    struct AllOpenSource;

    impl Source for AllOpenSource {
        fn name(&self) -> String {
            "all-open".to_string()
        }
        fn kind(&self) -> SourceKind {
            SourceKind::Csprng
        }
        fn generate_u64(&mut self) -> Result<u64, SourceError> {
            Ok(u64::MAX)
        }
        fn fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), SourceError> {
            buf.fill(u8::MAX);
            Ok(())
        }
        fn health(&self) -> SourceHealth {
            SourceHealth::Healthy
        }
    }

    /// Regression: with `shells > 255` the old `open_count: u8` accumulator
    /// overflowed (debug panic). The method must be safe when called directly
    /// with a large shell count, regardless of the API-level cap of 64.
    #[test]
    fn large_shell_count_does_not_overflow() {
        let mut src = AllOpenSource;
        let result = cast_cowrie(&mut src, 300);
        let result = result.expect("300 shells should succeed");
        assert_eq!(result.shells, 300);
        assert!(
            result.open_count <= 300,
            "open_count {} exceeds shells 300",
            result.open_count
        );
        // Every shell was forced open, so the u32 accumulator held 300 without
        // wrapping; this is the value that previously panicked as a u8.
        assert_eq!(result.open_count, 300);
    }
}
