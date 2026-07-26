use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;

/// Draw `count` distinct winners from a list of items (sortition).
pub fn draw_lots<T: Clone>(
    source: &mut dyn Source,
    items: &[T],
    count: usize,
) -> Result<Vec<T>, crate::core::SourceError> {
    if count > items.len() {
        return Err(crate::core::SourceError::GenerationFailed(
            "cannot draw more lots than items".to_string(),
        ));
    }
    if count == 0 {
        return Ok(Vec::new());
    }

    let mut indices: Vec<usize> = (0..items.len()).collect();
    for i in (1..indices.len()).rev() {
        let j = uniform_u64_inclusive(source, 0, i as u64)? as usize;
        indices.swap(i, j);
    }

    Ok(indices
        .into_iter()
        .take(count)
        .map(|i| items[i].clone())
        .collect())
}
