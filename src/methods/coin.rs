use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoinSide {
    Heads,
    Tails,
}

impl std::fmt::Display for CoinSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoinSide::Heads => write!(f, "heads"),
            CoinSide::Tails => write!(f, "tails"),
        }
    }
}

pub fn flip(source: &mut dyn Source) -> Result<CoinSide, crate::core::SourceError> {
    let v = uniform_u64_inclusive(source, 0, 1)?;
    Ok(if v == 0 { CoinSide::Heads } else { CoinSide::Tails })
}

pub fn flip_n(source: &mut dyn Source, n: u64) -> Result<Vec<CoinSide>, crate::core::SourceError> {
    (0..n).map(|_| flip(source)).collect()
}
