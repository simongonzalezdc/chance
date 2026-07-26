use crate::core::source::Source;

pub fn random_bytes(
    source: &mut dyn Source,
    count: usize,
) -> Result<Vec<u8>, crate::core::SourceError> {
    let mut buf = vec![0u8; count];
    source.fill_bytes(&mut buf)?;
    Ok(buf)
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

pub fn bytes_to_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    let mut chunks = bytes.chunks_exact(3);
    for chunk in chunks.by_ref() {
        let n = (chunk[0] as u32) << 16 | (chunk[1] as u32) << 8 | (chunk[2] as u32);
        out.push(ALPHABET[((n >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        out.push(ALPHABET[((n >> 6) & 63) as usize] as char);
        out.push(ALPHABET[(n & 63) as usize] as char);
    }
    let rem = chunks.remainder();
    if !rem.is_empty() {
        let n = match rem.len() {
            1 => (rem[0] as u32) << 16,
            2 => (rem[0] as u32) << 16 | (rem[1] as u32) << 8,
            _ => unreachable!(),
        };
        out.push(ALPHABET[((n >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        if rem.len() == 2 {
            out.push(ALPHABET[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        out.push('=');
    }
    out
}
