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

/// Whether a rune has an attested reverse (merkstave) form.
///
/// Gebo and Isa are symmetric glyphs with no historical reverse meaning, so
/// they are always read upright regardless of an orientation draw.
fn has_reverse_form(name: &str) -> bool {
    !matches!(name, "gebo" | "isa")
}

/// Draw a single Elder Futhark rune with a random upright/reversed orientation.
///
/// Each draw is independent — runes are drawn **with replacement**, so the same
/// rune may recur across multiple draws. The orientation is a fair 50/50 flip,
/// except for runes with no reverse form (Gebo, Isa), which are always returned
/// upright even when the orientation flip comes up "reversed".
pub fn draw_rune(source: &mut dyn Source) -> Result<RuneResult, crate::core::SourceError> {
    let idx = uniform_u64_inclusive(source, 0, (ELDER_FUTHARK.len() - 1) as u64)? as usize;
    let name = ELDER_FUTHARK[idx];
    // 1 = upright, 0 = reversed. No-reverse runes ignore the bit.
    let mut upright = uniform_u64_inclusive(source, 0, 1)? == 1;
    if !has_reverse_form(name) {
        upright = true;
    }
    Ok(RuneResult { name, upright })
}

/// Draw `count` Elder Futhark runes. Draws are independent (with replacement),
/// so duplicates are possible. Runes with no reverse form are always upright.
pub fn draw_runes(
    source: &mut dyn Source,
    count: usize,
) -> Result<Vec<RuneResult>, crate::core::SourceError> {
    (0..count).map(|_| draw_rune(source)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{SourceError, SourceHealth, SourceKind};

    /// Deterministic source that yields a programmed sequence of u64 values.
    struct ScriptedSource {
        values: Vec<u64>,
        pos: usize,
    }

    impl Source for ScriptedSource {
        fn name(&self) -> String {
            "scripted".to_string()
        }
        fn kind(&self) -> SourceKind {
            SourceKind::Prng
        }
        fn generate_u64(&mut self) -> Result<u64, SourceError> {
            if self.pos >= self.values.len() {
                return Err(SourceError::GenerationFailed(
                    "scripted source exhausted".to_string(),
                ));
            }
            let v = self.values[self.pos];
            self.pos += 1;
            Ok(v)
        }
        fn fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), SourceError> {
            for chunk in buf.chunks_mut(8) {
                let v = self.generate_u64()?;
                let bytes = v.to_le_bytes();
                for (dst, &src) in chunk.iter_mut().zip(bytes.iter()) {
                    *dst = src;
                }
            }
            Ok(())
        }
        fn health(&self) -> SourceHealth {
            SourceHealth::Healthy
        }
    }

    #[test]
    fn only_gebo_and_isa_lack_a_reverse_form() {
        assert!(!has_reverse_form("gebo"));
        assert!(!has_reverse_form("isa"));
        for name in ELDER_FUTHARK {
            let expected = !matches!(*name, "gebo" | "isa");
            assert_eq!(
                has_reverse_form(name),
                expected,
                "{name} reverse-form classification wrong"
            );
        }
    }

    /// Core fix: Gebo and Isa come back upright even when the orientation draw
    /// is "reversed", while a normal rune (fehu) with the same draw IS reversed.
    /// Values are chosen so the Lemire sampler lands on a known index without
    /// entering its rejection path, making the run fully deterministic.
    #[test]
    fn no_reverse_runes_stay_upright_when_reversal_drawn() {
        // (x * range) mod 2^64 comfortably exceeds the Lemire threshold for each.
        let x_gebo = (6u128 << 64) / 24 + 100; // idx 6 = gebo
        let x_isa = (10u128 << 64) / 24 + 100; // idx 10 = isa
        let x_fehu = 100u128; // idx 0 = fehu (reversible)
        let reversed_bit = 1u128; // orientation draw -> 0 -> reversed

        let mut src = ScriptedSource {
            values: vec![
                x_gebo as u64, reversed_bit as u64,
                x_isa as u64, reversed_bit as u64,
                x_fehu as u64, reversed_bit as u64,
            ],
            pos: 0,
        };

        let gebo = draw_rune(&mut src).unwrap();
        assert_eq!(gebo.name, "gebo");
        assert!(gebo.upright, "gebo must be upright (no reverse form)");

        let isa = draw_rune(&mut src).unwrap();
        assert_eq!(isa.name, "isa");
        assert!(isa.upright, "isa must be upright (no reverse form)");

        let fehu = draw_rune(&mut src).unwrap();
        assert_eq!(fehu.name, "fehu");
        assert!(
            !fehu.upright,
            "fehu must be reversed when the orientation bit is reversed"
        );
    }

    /// `draw_runes` draws with replacement and never produces a reversed
    /// Gebo/Isa across a real deterministic stream.
    #[test]
    fn draw_runes_never_reverses_no_reverse_runes() {
        let mut src = crate::sources::splitmix::splitmix64(Some("runes-no-reverse-seed"))
            .expect("splitmix source");
        let runes = draw_runes(src.as_mut(), 3000).unwrap();
        assert_eq!(runes.len(), 3000);
        for r in &runes {
            if !has_reverse_form(r.name) {
                assert!(r.upright, "{} must never be reversed", r.name);
            }
        }
        // Orientation still flips for reversible runes.
        assert!(runes.iter().any(|r| !r.upright), "expected at least one reversed rune");
    }
}
