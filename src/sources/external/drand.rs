use crate::core::{Source, SourceError, SourceHealth, SourceKind};
use serde::Deserialize;

const DRAND_URL: &str = "https://api.drand.sh/public/latest";

/// drand / League of Entropy — publicly verifiable distributed randomness beacon.
///
/// Note: this MVP implementation trusts the HTTPS endpoint and does not verify
/// the BLS threshold signature. For high-assurance use, add on-chain verification.
pub struct DrandSource {
    client: reqwest::blocking::Client,
    cache: Vec<u8>,
}

impl DrandSource {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            cache: Vec::new(),
        }
    }

    fn fetch_round(&mut self) -> Result<Vec<u8>, SourceError> {
        let resp = self
            .client
            .get(DRAND_URL)
            .send()
            .map_err(|e| SourceError::GenerationFailed(format!("drand request failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(SourceError::GenerationFailed(format!(
                "drand returned status {}",
                resp.status()
            )));
        }

        let body: DrandResponse = resp.json().map_err(|e| {
            SourceError::GenerationFailed(format!("drand response parse failed: {e}"))
        })?;

        hex_to_bytes(&body.randomness)
    }

    fn ensure_bytes(&mut self, count: usize) -> Result<(), SourceError> {
        while self.cache.len() < count {
            let bytes = self.fetch_round()?;
            self.cache.extend_from_slice(&bytes);
        }
        Ok(())
    }
}

impl Source for DrandSource {
    fn name(&self) -> String {
        "drand".to_string()
    }

    fn kind(&self) -> SourceKind {
        SourceKind::Beacon
    }

    fn generate_u64(&mut self) -> Result<u64, SourceError> {
        self.ensure_bytes(8)?;
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&self.cache[..8]);
        self.cache.drain(..8);
        Ok(u64::from_le_bytes(arr))
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), SourceError> {
        self.ensure_bytes(buf.len())?;
        buf.copy_from_slice(&self.cache[..buf.len()]);
        self.cache.drain(..buf.len());
        Ok(())
    }

    fn health(&self) -> SourceHealth {
        SourceHealth::Healthy
    }
}

#[derive(Deserialize)]
struct DrandResponse {
    #[allow(dead_code)]
    round: u64,
    randomness: String,
    #[allow(dead_code)]
    signature: String,
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, SourceError> {
    if hex.len() % 2 != 0 {
        return Err(SourceError::GenerationFailed(
            "invalid hex length".to_string(),
        ));
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| {
            SourceError::GenerationFailed(format!("invalid hex: {e}"))
        })?;
        bytes.push(byte);
    }
    Ok(bytes)
}
