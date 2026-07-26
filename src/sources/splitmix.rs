use crate::core::{RngSource, Source, SourceKind};

/// SplitMix64 — fast, small-state; ideal for seeding larger generators.
pub fn splitmix64(seed: Option<&str>) -> Result<Box<dyn Source>, crate::core::SourceError> {
    // SplitMix64 is not a separate crate here, but StdRng with a 64-bit seed
    // is implemented via SplitMix64-style seed expansion in rand. We use a
    // custom tiny implementation to expose a true SplitMix64 source.
    let raw = match seed {
        Some(s) => crate::sources::seed::parse_seed(s)?,
        None => crate::sources::seed::random_seed(),
    };
    let seed_str = format!("0x{:016x}", raw);
    Ok(Box::new(
        RngSource::new(SplitMix64::new(raw), "splitmix64", SourceKind::Prng).with_seed(seed_str),
    ))
}

/// Minimal SplitMix64 implementation.
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}

impl rand::RngCore for SplitMix64 {
    fn next_u32(&mut self) -> u32 {
        self.next() as u32
    }

    fn next_u64(&mut self) -> u64 {
        self.next()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut chunks = dest.chunks_exact_mut(8);
        for chunk in chunks.by_ref() {
            chunk.copy_from_slice(&self.next().to_le_bytes());
        }
        let remainder = chunks.into_remainder();
        if !remainder.is_empty() {
            let bytes = self.next().to_le_bytes();
            remainder.copy_from_slice(&bytes[..remainder.len()]);
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}
