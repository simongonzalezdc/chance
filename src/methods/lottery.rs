use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;

#[derive(Debug, Clone)]
pub struct LotteryResult {
    pub numbers: Vec<u8>,
    pub bonus: Option<u8>,
}

impl std::fmt::Display for LotteryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.numbers)?;
        if let Some(b) = self.bonus {
            write!(f, " + {}", b)?;
        }
        Ok(())
    }
}

/// Draw `pick` numbers from 1..=pool without replacement, optionally plus a bonus ball.
pub fn draw_lottery(
    source: &mut dyn Source,
    pool: u8,
    pick: usize,
    bonus_pool: Option<u8>,
) -> Result<LotteryResult, crate::core::SourceError> {
    let mut numbers: Vec<u8> = (1..=pool).collect();
    for i in (1..numbers.len()).rev() {
        let j = uniform_u64_inclusive(source, 0, i as u64)? as usize;
        numbers.swap(i, j);
    }
    numbers.truncate(pick);
    numbers.sort();

    let bonus = match bonus_pool {
        Some(bp) => Some(uniform_u64_inclusive(source, 1, bp as u64)? as u8),
        None => None,
    };

    Ok(LotteryResult { numbers, bonus })
}
