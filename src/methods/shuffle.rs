use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;

/// Fisher-Yates shuffle of a slice.
pub fn shuffle<T>(source: &mut dyn Source, items: &mut [T]) -> Result<(), crate::core::SourceError> {
    let n = items.len();
    for i in (1..n).rev() {
        let j = uniform_u64_inclusive(source, 0, i as u64)? as usize;
        items.swap(i, j);
    }
    Ok(())
}
