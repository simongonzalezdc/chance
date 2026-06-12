use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;

#[derive(Debug, Clone)]
pub struct TeetotumResult {
    pub face: &'static str,
}

impl std::fmt::Display for TeetotumResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.face)
    }
}

/// Spin a four-sided teetotum / dreidel.
/// Default Latin faces: N (nothing), S (put), A (all), T (take).
pub fn spin_teetotum(source: &mut dyn Source) -> Result<TeetotumResult, crate::core::SourceError> {
    let faces = ["N", "S", "A", "T"];
    let idx = uniform_u64_inclusive(source, 0, 3)? as usize;
    Ok(TeetotumResult { face: faces[idx] })
}

/// Spin a Hebrew dreidel: נ (Nun), ג (Gimmel), ה (Hey), ש (Shin).
pub fn spin_dreidel(source: &mut dyn Source) -> Result<TeetotumResult, crate::core::SourceError> {
    let faces = ["נ (Nun)", "ג (Gimmel)", "ה (Hey)", "ש (Shin)"];
    let idx = uniform_u64_inclusive(source, 0, 3)? as usize;
    Ok(TeetotumResult { face: faces[idx] })
}
