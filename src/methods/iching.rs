use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;

#[derive(Debug, Clone)]
pub struct IChingResult {
    pub primary: u8,
    pub transformed: Option<u8>,
    pub lines: Vec<IChingLine>,
    pub method: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct IChingLine {
    pub value: u8,      // 6, 7, 8, 9
    pub yang: bool,     // 7/9 are yang, 6/8 are yin
    pub changing: bool, // 6/9 are changing
}

impl IChingResult {
    pub fn hexagram_name(&self) -> &'static str {
        let kw = KING_WEN_BY_PRIMARY[self.primary as usize];
        HEXAGRAM_NAMES
            .get(kw as usize - 1)
            .copied()
            .unwrap_or("unknown")
    }
}

const HEXAGRAM_NAMES: &[&str] = &[
    "Ch'ien / The Creative",
    "K'un / The Receptive",
    "Chun / Difficulty at the Beginning",
    "Mêng / Youthful Folly",
    "Hsü / Waiting",
    "Sung / Conflict",
    "Shih / The Army",
    "Pi / Holding Together",
    "Hsiao Ch'u / The Taming Power of the Small",
    "Lü / Treading",
    "T'ai / Peace",
    "P'i / Standstill",
    "T'ung Jên / Fellowship with Men",
    "Ta Yu / Possession in Great Measure",
    "Ch'ien / Modesty",
    "Yü / Enthusiasm",
    "Sui / Following",
    "Ku / Work on the Decayed",
    "Lin / Approach",
    "Kuan / Contemplation",
    "Shih Ho / Biting Through",
    "Pi / Grace",
    "Po / Splitting Apart",
    "Fu / Return",
    "Wu Wang / Innocence",
    "Ta Ch'u / The Taming Power of the Great",
    "I / The Corners of the Mouth",
    "Ta Kuo / Preponderance of the Great",
    "K'an / The Abysmal",
    "Li / The Clinging",
    "Hsien / Influence",
    "Hêng / Duration",
    "Tun / Retreat",
    "Ta Chuang / The Power of the Great",
    "Chin / Progress",
    "Ming I / Darkening of the Light",
    "Chia Jên / The Family",
    "K'uei / Opposition",
    "Chien / Obstruction",
    "Hsieh / Deliverance",
    "Sun / Decrease",
    "I / Increase",
    "Kuai / Break-through",
    "Kou / Coming to Meet",
    "Ts'ui / Gathering Together",
    "Shêng / Pushing Upward",
    "K'un / Oppression",
    "Ching / The Well",
    "Ko / Revolution",
    "Ting / The Cauldron",
    "Chên / The Arousing",
    "Kên / Keeping Still",
    "Chien / Development",
    "Kuei Mei / The Marrying Maiden",
    "Fêng / Abundance",
    "Lü / The Wanderer",
    "Sun / The Gentle",
    "Tui / The Joyous",
    "Huan / Dispersion",
    "Chieh / Limitation",
    "Chung Fu / Inner Truth",
    "Hsiao Kuo / Preponderance of the Small",
    "Chi Chi / After Completion",
    "Wei Chi / Before Completion",
];
// Map from the `primary` bitmap to the King Wen hexagram number (1..64).
//
// The bitmap built by `cast_iching` treats the BOTTOM line (line 1) as the
// most-significant bit: bit 5 = line 1, ..., bit 0 = line 6. The trigram in
// lines 1-3 (lower) therefore occupies the high 3 bits and the trigram in
// lines 4-6 (upper) the low 3 bits.
//
// Each entry is the canonical King Wen number for that binary hexagram, i.e.
// the value of `kw = KING_WEN_BY_PRIMARY[primary]` indexes into `HEXAGRAM_NAMES`
// at `kw - 1`. The mapping is a bijection over 0..=63; the four palindromic
// complement-pairs (1/2, 27/28, 29/30, 61/62) and the remaining 28 rotation
// pairs are consistent with the King Wen pairing structure.
const KING_WEN_BY_PRIMARY: [u8; 64] = [
    2, 23, 8, 20, 16, 35, 45, 12, 15, 52, 39, 53, 62, 56, 41, 33, 7, 4, 29, 59, 40, 64, 47, 6, 46,
    18, 48, 57, 32, 50, 28, 44, 24, 27, 3, 42, 51, 21, 17, 25, 36, 22, 63, 37, 55, 30, 49, 13, 19,
    31, 60, 61, 54, 38, 58, 10, 11, 26, 5, 9, 34, 14, 43, 1,
];

/// Cast one I Ching line using the given method.
fn cast_line(
    source: &mut dyn Source,
    method: &str,
) -> Result<IChingLine, crate::core::SourceError> {
    let value = match method {
        "yarrow" => cast_yarrow(source)?,
        "coin" | _ => cast_coin(source)?,
    };
    Ok(IChingLine {
        value,
        yang: value == 7 || value == 9,
        changing: value == 6 || value == 9,
    })
}

/// Yarrow stalk method probabilities: 6: 1/16, 7: 5/16, 8: 7/16, 9: 3/16.
fn cast_yarrow(source: &mut dyn Source) -> Result<u8, crate::core::SourceError> {
    let r = uniform_u64_inclusive(source, 1, 16)?;
    Ok(match r {
        1 => 6,
        2..=6 => 7,
        7..=13 => 8,
        _ => 9,
    })
}

/// Three-coin method probabilities: 6: 1/8, 7: 3/8, 8: 3/8, 9: 1/8.
fn cast_coin(source: &mut dyn Source) -> Result<u8, crate::core::SourceError> {
    let r = uniform_u64_inclusive(source, 1, 8)?;
    Ok(match r {
        1 => 6,
        2..=4 => 7,
        5..=7 => 8,
        _ => 9,
    })
}

/// Cast a full I Ching reading.
pub fn cast_iching(
    source: &mut dyn Source,
    method: &str,
) -> Result<IChingResult, crate::core::SourceError> {
    let method = if method == "yarrow" { "yarrow" } else { "coin" };
    let mut lines = Vec::with_capacity(6);
    let mut primary = 0u8;
    let mut transformed: Option<u8> = None;

    for _ in 0..6 {
        let line = cast_line(source, method)?;
        primary = (primary << 1) | if line.yang { 1 } else { 0 };
        lines.push(line);
    }

    if lines.iter().any(|l| l.changing) {
        let mut t = 0u8;
        for line in lines.iter().rev() {
            t = (t << 1)
                | if line.yang {
                    if line.value == 9 {
                        0
                    } else {
                        1
                    }
                } else {
                    if line.value == 6 {
                        1
                    } else {
                        0
                    }
                };
        }
        transformed = Some(t);
    }

    Ok(IChingResult {
        primary,
        transformed,
        lines,
        method,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build the `primary` bitmap exactly the way `cast_iching` does: lines are
    /// supplied bottom-first (index 0 = line 1 / bottom, index 5 = line 6 / top),
    /// and each line is shifted in as `(acc << 1) | bit`. The result therefore has
    /// the bottom line in the most-significant bit (bit 5).
    fn primary_from_lines(lines: [bool; 6]) -> u8 {
        let mut primary = 0u8;
        for &yang in lines.iter() {
            primary = (primary << 1) | if yang { 1 } else { 0 };
        }
        primary
    }

    fn name_for_primary(primary: u8) -> &'static str {
        IChingResult {
            primary,
            transformed: None,
            lines: Vec::new(),
            method: "test",
        }
        .hexagram_name()
    }

    #[test]
    fn all_yang_is_chien_kw1() {
        // (a) 6 yang lines -> primary 0b111111 = 63 -> KW1 "Ch'ien / The Creative".
        let primary = primary_from_lines([true, true, true, true, true, true]);
        assert_eq!(primary, 0b111111);
        assert_eq!(primary, 63);
        assert_eq!(name_for_primary(primary), "Ch'ien / The Creative");
        assert_eq!(KING_WEN_BY_PRIMARY[primary as usize], 1);
    }

    #[test]
    fn all_yin_is_kun_kw2() {
        // (b) 6 yin lines -> primary 0 -> KW2 "K'un / The Receptive".
        // This previously underflowed `0usize - 1` in `hexagram_name`.
        let primary = primary_from_lines([false, false, false, false, false, false]);
        assert_eq!(primary, 0);
        assert_eq!(name_for_primary(primary), "K'un / The Receptive");
        assert_eq!(KING_WEN_BY_PRIMARY[primary as usize], 2);
    }

    #[test]
    fn bottom_yang_top_yin_is_tai_kw11() {
        // (c) bottom 3 yang + top 3 yin (Earth above Heaven) -> KW11 "T'ai / Peace".
        let primary = primary_from_lines([true, true, true, false, false, false]);
        assert_eq!(primary, 0b111000);
        assert_eq!(primary, 56);
        assert_eq!(name_for_primary(primary), "T'ai / Peace");
        assert_eq!(KING_WEN_BY_PRIMARY[primary as usize], 11);
    }

    #[test]
    fn bottom_yin_top_yang_is_pi_kw12() {
        // (d) bottom 3 yin + top 3 yang (Heaven above Earth) -> KW12 "P'i / Standstill".
        let primary = primary_from_lines([false, false, false, true, true, true]);
        assert_eq!(primary, 0b000111);
        assert_eq!(primary, 7);
        assert_eq!(name_for_primary(primary), "P'i / Standstill");
        assert_eq!(KING_WEN_BY_PRIMARY[primary as usize], 12);
    }

    #[test]
    fn doubled_water_is_kan_kw29() {
        // (e) Kan over Kan: each Kan trigram is (yin, yang, yin) bottom-to-top.
        let primary = primary_from_lines([false, true, false, false, true, false]);
        assert_eq!(primary, 0b010010);
        assert_eq!(primary, 18);
        assert_eq!(name_for_primary(primary), "K'an / The Abysmal");
        assert_eq!(KING_WEN_BY_PRIMARY[primary as usize], 29);
    }

    #[test]
    fn doubled_fire_is_li_kw30() {
        // (f) Li over Li: each Li trigram is (yang, yin, yang) bottom-to-top.
        let primary = primary_from_lines([true, false, true, true, false, true]);
        assert_eq!(primary, 0b101101);
        assert_eq!(primary, 45);
        assert_eq!(name_for_primary(primary), "Li / The Clinging");
        assert_eq!(KING_WEN_BY_PRIMARY[primary as usize], 30);
    }

    #[test]
    fn table_is_a_bijection_of_king_wen_numbers() {
        // Every King Wen number 1..=64 must appear exactly once: this guards
        // against transcription errors in the rest of the table beyond the six
        // pinned vectors above.
        let mut seen = [false; 65];
        for &kw in KING_WEN_BY_PRIMARY.iter() {
            assert!(
                (1..=64).contains(&kw),
                "King Wen number {kw} out of range 1..=64"
            );
            assert!(!seen[kw as usize], "King Wen number {kw} appears twice");
            seen[kw as usize] = true;
        }
        for n in 1..=64u8 {
            assert!(seen[n as usize], "King Wen number {n} is missing");
        }
    }
}
