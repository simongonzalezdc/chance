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

    fn mix(&mut self, mut inputs: Vec<Vec<u8>>) -> Result<Vec<u8>, SourceError> {
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
        let mut okm = [0u8; 64];
        hkdf.expand(b"chance-randomness", &mut okm)
            .map_err(|e| SourceError::GenerationFailed(format!("hkdf expand failed: {e}")))?;
        Ok(okm.to_vec())
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

        let mixed = self.mix(outputs)?;
        buf.copy_from_slice(&mixed[..needed]);
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
