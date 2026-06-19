use crate::core::source::Source;
use crate::core::{SourceError, SourceHealth, SourceKind};

/// A source that mixes multiple independent sources via HKDF-SHA-256.
///
/// Each call fetches bytes from every underlying source, concatenates them,
/// and extracts a fixed-length output with HKDF using an incrementing salt.
/// If any source fails, the mixer falls back to the remaining sources.
pub struct Mixer {
    sources: Vec<Box<dyn Source>>,
    name: String,
    counter: u64,
}

impl Mixer {
    pub fn new(sources: Vec<Box<dyn Source>>, name: &str) -> Self {
        Self {
            sources,
            name: name.to_string(),
            counter: 0,
        }
    }

    fn mix(
        &mut self,
        mut inputs: Vec<Vec<u8>>,
        needed: usize,
    ) -> Result<Vec<u8>, SourceError> {
        const MAX_OUTPUT: usize = 1024 * 1024; // 1 MiB cap
        const BLOCK_LEN: usize = 32; // SHA-256 output block size

        if needed == 0 {
            return Ok(Vec::new());
        }
        if needed > MAX_OUTPUT {
            return Err(SourceError::GenerationFailed(format!(
                "requested {needed} bytes exceeds mixer maximum of {MAX_OUTPUT} bytes"
            )));
        }
        if inputs.is_empty() {
            return Err(SourceError::GenerationFailed(
                "no source outputs available to mix".to_string(),
            ));
        }

        // Concatenate all source outputs.
        let mut combined = Vec::new();
        for input in &mut inputs {
            combined.append(input);
        }

        // Use HKDF-SHA-256 extract with a counter-based salt.
        let salt = format!("chance-mixer-{}", self.counter);
        self.counter += 1;

        let hkdf = hkdf::Hkdf::<sha2::Sha256>::new(Some(salt.as_bytes()), &combined);

        // Expand in 32-byte blocks, each under a counter-suffixed `info`, to
        // reach an arbitrary `needed` length. A single HKDF-Expand call tops
        // out at 255 * hashlen, so we drive one independent block per counter.
        let mut okm = Vec::with_capacity(needed);
        let mut block_index: u64 = 0;
        while okm.len() < needed {
            let take = std::cmp::min(BLOCK_LEN, needed - okm.len());
            let info = format!("chance-randomness-{}", block_index);
            let mut block = [0u8; BLOCK_LEN];
            hkdf.expand(info.as_bytes(), &mut block[..take])
                .map_err(|e| SourceError::GenerationFailed(format!("hkdf expand failed: {e}")))?;
            okm.extend_from_slice(&block[..take]);
            block_index += 1;
        }

        Ok(okm)
    }

    fn ensure_buffer(&mut self, buf: &mut [u8]) -> Result<(), SourceError> {
        let needed = buf.len();
        let mut outputs: Vec<Vec<u8>> = Vec::new();
        for source in &mut self.sources {
            let mut out = vec![0u8; needed];
            match source.fill_bytes(&mut out) {
                Ok(_) => outputs.push(out),
                Err(e) => {
                    eprintln!("mixer warning: source {} failed: {}", source.name(), e);
                }
            }
        }

        if outputs.is_empty() {
            return Err(SourceError::GenerationFailed(
                "all mixer sources failed".to_string(),
            ));
        }

        let mixed = self.mix(outputs, needed)?;
        buf.copy_from_slice(&mixed);
        Ok(())
    }
}

impl Source for Mixer {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn kind(&self) -> SourceKind {
        SourceKind::Csprng
    }

    fn generate_u64(&mut self) -> Result<u64, SourceError> {
        let mut buf = [0u8; 8];
        self.ensure_buffer(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), SourceError> {
        self.ensure_buffer(buf)
    }

    fn health(&self) -> SourceHealth {
        if self.sources.iter().any(|s| s.health() == SourceHealth::Healthy) {
            SourceHealth::Healthy
        } else {
            SourceHealth::Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::OsCsprng;

    fn two_os_mixer() -> Mixer {
        let sources: Vec<Box<dyn Source>> = vec![
            Box::new(OsCsprng::new()),
            Box::new(OsCsprng::new()),
        ];
        Mixer::new(sources, "test-mixer")
    }

    /// Regression for B3: requesting more than the old fixed 64-byte OKM buffer
    /// used to index-out-of-bounds panic in `ensure_buffer`. Now `mix()` produces
    /// exactly `needed` bytes by looping HKDF-Expand in 32-byte blocks.
    #[test]
    fn fill_bytes_handles_buffers_larger_than_64() {
        let mut mixer = two_os_mixer();
        let mut buf = [0u8; 200];
        mixer.fill_bytes(&mut buf).expect("fill_bytes should succeed");

        // Exactly 200 bytes produced (no panic, no short slice).
        assert_eq!(buf.len(), 200);

        // A real CSPRNG-mixed buffer of 200 bytes is astronomically unlikely
        // to be all zeros; this guards against a no-op / wrong-length path.
        assert!(buf.iter().any(|&b| b != 0), "buffer should not be all zeros");
    }
}
