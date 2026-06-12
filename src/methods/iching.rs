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
    pub value: u8,        // 6, 7, 8, 9
    pub yang: bool,       // 7/9 are yang, 6/8 are yin
    pub changing: bool,   // 6/9 are changing
}

impl IChingResult {
    pub fn hexagram_name(&self) -> &'static str {
        HEXAGRAM_NAMES.get(self.primary as usize - 1).copied().unwrap_or("unknown")
    }
}

const HEXAGRAM_NAMES: &[&str] = &[
    "Ch'ien / The Creative", "K'un / The Receptive", "Chun / Difficulty at the Beginning",
    "Mêng / Youthful Folly", "Hsü / Waiting", "Sung / Conflict",
    "Shih / The Army", "Pi / Holding Together", "Hsiao Ch'u / The Taming Power of the Small",
    "Lü / Treading", "T'ai / Peace", "P'i / Standstill",
    "T'ung Jên / Fellowship with Men", "Ta Yu / Possession in Great Measure",
    "Ch'ien / Modesty", "Yü / Enthusiasm", "Sui / Following", "Ku / Work on the Decayed",
    "Lin / Approach", "Kuan / Contemplation", "Shih Ho / Biting Through",
    "Pi / Grace", "Po / Splitting Apart", "Fu / Return",
    "Wu Wang / Innocence", "Ta Ch'u / The Taming Power of the Great",
    "I / The Corners of the Mouth", "Ta Kuo / Preponderance of the Great",
    "K'an / The Abysmal", "Li / The Clinging", "Hsien / Influence",
    "Hêng / Duration", "Tun / Retreat", "Ta Chuang / The Power of the Great",
    "Chin / Progress", "Ming I / Darkening of the Light", "Chia Jên / The Family",
    "K'uei / Opposition", "Chien / Obstruction", "Hsieh / Deliverance",
    "Sun / Decrease", "I / Increase", "Kuai / Break-through",
    "Kou / Coming to Meet", "Ts'ui / Gathering Together", "Shêng / Pushing Upward",
    "K'un / Oppression", "Ching / The Well", "Ko / Revolution",
    "Ting / The Cauldron", "Chên / The Arousing", "Kên / Keeping Still",
    "Chien / Development", "Kuei Mei / The Marrying Maiden", "Fêng / Abundance",
    "Lü / The Wanderer", "Sun / The Gentle", "Tui / The Joyous",
    "Huan / Dispersion", "Chieh / Limitation", "Chung Fu / Inner Truth",
    "Hsiao Kuo / Preponderance of the Small", "Chi Chi / After Completion",
    "Wei Chi / Before Completion",
];

/// Cast one I Ching line using the given method.
fn cast_line(source: &mut dyn Source, method: &str) -> Result<IChingLine, crate::core::SourceError> {
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
            t = (t << 1) | if line.yang {
                if line.value == 9 { 0 } else { 1 }
            } else {
                if line.value == 6 { 1 } else { 0 }
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
