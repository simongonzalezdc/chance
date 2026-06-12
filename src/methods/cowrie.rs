use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;

#[derive(Debug, Clone)]
pub struct CowrieResult {
    pub shells: usize,
    pub open_count: u8,
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
    let mut open_count = 0u8;
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
