use crate::core::source::Source;
use crate::methods::bytes::random_bytes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uuid(pub String);

impl std::fmt::Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Generate a UUIDv4 (random) or UUIDv7 (time-ordered, random suffix).
pub fn generate_uuid(
    source: &mut dyn Source,
    version: u8,
) -> Result<Uuid, crate::core::SourceError> {
    match version {
        4 => {
            let mut b = random_bytes(source, 16)?;
            b[6] = (b[6] & 0x0f) | 0x40;
            b[8] = (b[8] & 0x3f) | 0x80;
            Ok(Uuid(format_uuid(&b)))
        }
        7 => {
            // UUIDv7: 48-bit Unix timestamp ms + 74 random bits.
            let now = chrono::Utc::now().timestamp_millis() as u64;
            let mut b = [0u8; 16];
            b[0..6].copy_from_slice(&now.to_be_bytes()[2..8]);
            let mut rand = random_bytes(source, 10)?;
            rand[0] = (rand[0] & 0x0f) | 0x70;
            rand[2] = (rand[2] & 0x3f) | 0x80;
            b[6..16].copy_from_slice(&rand);
            Ok(Uuid(format_uuid(&b)))
        }
        _ => Err(crate::core::SourceError::GenerationFailed(format!(
            "unsupported UUID version {}; use 4 or 7",
            version
        ))),
    }
}

fn format_uuid(bytes: &[u8]) -> String {
    assert_eq!(bytes.len(), 16);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(36);
    for (i, b) in bytes.iter().enumerate() {
        if [4, 6, 8, 10].contains(&i) {
            s.push('-');
        }
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}
