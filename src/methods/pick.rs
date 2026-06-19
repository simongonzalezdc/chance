use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;

/// Pick one item uniformly from a list.
pub fn pick_one<T: Clone>(source: &mut dyn Source, items: &[T]) -> Result<T, crate::core::SourceError> {
    if items.is_empty() {
        return Err(crate::core::SourceError::GenerationFailed(
            "cannot pick from empty list".to_string(),
        ));
    }
    let idx = uniform_u64_inclusive(source, 0, (items.len() - 1) as u64)? as usize;
    Ok(items[idx].clone())
}

/// Pick `count` distinct winners from a list (simple random sample without replacement).
pub fn pick_distinct<T: Clone>(
    source: &mut dyn Source,
    items: &[T],
    count: usize,
) -> Result<Vec<T>, crate::core::SourceError> {
    if count > items.len() {
        return Err(crate::core::SourceError::GenerationFailed(
            "cannot pick more items than available".to_string(),
        ));
    }
    if count == 0 {
        return Ok(Vec::new());
    }

    // Reservoir sampling.
    let mut result: Vec<T> = items.iter().take(count).cloned().collect();
    for (i, item) in items.iter().enumerate().skip(count) {
        let j = uniform_u64_inclusive(source, 0, i as u64)? as usize;
        if j < count {
            result[j] = item.clone();
        }
    }
    Ok(result)
}

