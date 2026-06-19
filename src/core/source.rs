use crate::core::{SourceHealth, SourceKind};
use thiserror::Error;

/// Errors that can occur when using a randomness source.
#[derive(Debug, Error)]
pub enum SourceError {
    #[error("source unavailable: {0}")]
    Unavailable(String),
    #[error("failed to generate random data: {0}")]
    GenerationFailed(String),
    #[error("invalid source name: {0}")]
    InvalidSource(String),
    #[error("unsupported operation for source {src}: {operation}")]
    UnsupportedOperation { src: String, operation: String },
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

/// A generic source of randomness.
pub trait Source: Send + Sync {
    /// Unique source identifier, e.g. `os-csprng`, `xoshiro256**`, `drand`.
    fn name(&self) -> String;

    /// Classification of the source.
    fn kind(&self) -> SourceKind;

    /// Generate a single `u64` value.
    fn generate_u64(&mut self) -> Result<u64, SourceError>;

    /// Fill a buffer with random bytes.
    fn fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), SourceError>;

    /// Current health status of the source.
    fn health(&self) -> SourceHealth;

    /// Optional seed, if the source is deterministic.
    fn seed(&self) -> Option<String> {
        None
    }
}

/// Adapter that turns any `rand::RngCore` into a `Source`.
pub struct RngSource<R> {
    rng: R,
    name: &'static str,
    kind: SourceKind,
    seed: Option<String>,
}

impl<R: rand::RngCore + Send + Sync> RngSource<R> {
    pub fn new(rng: R, name: &'static str, kind: SourceKind) -> Self {
        Self {
            rng,
            name,
            kind,
            seed: None,
        }
    }

    pub fn with_seed(mut self, seed: impl Into<String>) -> Self {
        self.seed = Some(seed.into());
        self
    }
}

impl<R: rand::RngCore + Send + Sync> Source for RngSource<R> {
    fn name(&self) -> String {
        self.name.to_string()
    }

    fn kind(&self) -> SourceKind {
        self.kind
    }

    fn generate_u64(&mut self) -> Result<u64, SourceError> {
        Ok(self.rng.next_u64())
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), SourceError> {
        self.rng.fill_bytes(buf);
        Ok(())
    }

    fn health(&self) -> SourceHealth {
        SourceHealth::Healthy
    }

    fn seed(&self) -> Option<String> {
        self.seed.clone()
    }
}
