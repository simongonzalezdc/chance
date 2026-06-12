use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;

/// Astragalus (knucklebone) faces with empirically observed biased probabilities.
/// Approximate distribution: broad sides ~40% each, narrow sides ~10% each.
/// Faces: 1, 3, 4, 6 (the four-sided talus bone; opposite faces are 1-6 and 3-4).
#[derive(Debug, Clone)]
pub struct KnucklebonesResult {
    pub values: Vec<u8>,
}

impl std::fmt::Display for KnucklebonesResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.values)
    }
}

fn cast_astragalus(source: &mut dyn Source) -> Result<u8, crate::core::SourceError> {
    // Cumulative distribution: 1 (10%), 3 (50%), 4 (90%), 6 (100%).
    let r = uniform_u64_inclusive(source, 1, 100)?;
    Ok(match r {
        1..=10 => 1,
        11..=50 => 3,
        51..=90 => 4,
        _ => 6,
    })
}

pub fn cast_knucklebones(
    source: &mut dyn Source,
    count: usize,
) -> Result<KnucklebonesResult, crate::core::SourceError> {
    let values = (0..count).map(|_| cast_astragalus(source)).collect::<Result<Vec<_>, _>>()?;
    Ok(KnucklebonesResult { values })
}
