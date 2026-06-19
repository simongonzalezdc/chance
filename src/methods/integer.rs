use crate::core::range::uniform_i64_inclusive;
use crate::core::source::Source;


pub fn random_i64(source: &mut dyn Source, min: i64, max: i64) -> Result<i64, crate::core::SourceError> {
    uniform_i64_inclusive(source, min, max)
}
