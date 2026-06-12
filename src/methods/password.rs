use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;

const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{}|;:,.<>?";

pub struct PasswordOptions {
    pub length: usize,
    pub lower: bool,
    pub upper: bool,
    pub digits: bool,
    pub symbols: bool,
}

impl Default for PasswordOptions {
    fn default() -> Self {
        Self {
            length: 16,
            lower: true,
            upper: true,
            digits: true,
            symbols: true,
        }
    }
}

pub fn generate_password(
    source: &mut dyn Source,
    options: &PasswordOptions,
) -> Result<String, crate::core::SourceError> {
    let mut alphabet: Vec<u8> = Vec::new();
    if options.lower {
        alphabet.extend_from_slice(LOWER);
    }
    if options.upper {
        alphabet.extend_from_slice(UPPER);
    }
    if options.digits {
        alphabet.extend_from_slice(DIGITS);
    }
    if options.symbols {
        alphabet.extend_from_slice(SYMBOLS);
    }

    if alphabet.is_empty() {
        return Err(crate::core::SourceError::GenerationFailed(
            "password alphabet is empty".to_string(),
        ));
    }

    let mut pw = String::with_capacity(options.length);
    for _ in 0..options.length {
        let idx = uniform_u64_inclusive(source, 0, (alphabet.len() - 1) as u64)? as usize;
        pw.push(alphabet[idx] as char);
    }

    Ok(pw)
}
