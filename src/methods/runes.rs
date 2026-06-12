use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;

const ELDER_FUTHARK: &[&str] = &[
    "fehu", "uruz", "thurisaz", "ansuz", "raido", "kenaz",
    "gebo", "wunjo", "hagalaz", "nauthiz", "isa", "jera",
    "eihwaz", "perthro", "algiz", "sowilo", "tiwaz", "berkano",
    "ehwaz", "mannaz", "laguz", "ingwaz", "dagaz", "othala",
];

#[derive(Debug, Clone)]
pub struct RuneResult {
    pub name: &'static str,
    pub upright: bool,
}

impl std::fmt::Display for RuneResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.upright {
            write!(f, "{} (upright)", self.name)
        } else {
            write!(f, "{} (reversed)", self.name)
        }
    }
}

pub fn draw_rune(source: &mut dyn Source) -> Result<RuneResult, crate::core::SourceError> {
    let idx = uniform_u64_inclusive(source, 0, (ELDER_FUTHARK.len() - 1) as u64)? as usize;
    let upright = uniform_u64_inclusive(source, 0, 1)? == 1;
    Ok(RuneResult {
        name: ELDER_FUTHARK[idx],
        upright,
    })
}

pub fn draw_runes(
    source: &mut dyn Source,
    count: usize,
) -> Result<Vec<RuneResult>, crate::core::SourceError> {
    (0..count).map(|_| draw_rune(source)).collect()
}
