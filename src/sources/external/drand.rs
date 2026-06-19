use crate::core::{Source, SourceError, SourceHealth, SourceKind};
use serde::Deserialize;

const DRAND_URL: &str = "https://api.drand.sh/public/latest";

/// drand / League of Entropy — publicly verifiable distributed randomness beacon.
///
/// Note: this MVP implementation trusts the HTTPS endpoint and does not verify
/// the BLS threshold signature. For high-assurance use, add on-chain verification.
///
/// The beacon emits exactly 32 bytes per round and a new round only every ~30s.
/// Any request needing more than 32 bytes must therefore walk *backwards* through
/// round numbers (`/public/{round-1}`, `/public/{round-2}`, ...) so each 32-byte
/// block is distinct. Hitting `/public/latest` in a loop would instead return the
/// same current round repeatedly and yield `R || R`.
pub struct DrandSource {
    client: reqwest::blocking::Client,
    /// Accumulated beacon bytes not yet consumed (front = next to consume).
    cache: Vec<u8>,
    /// Round number of the most-recently fetched block sitting at the tail of
    /// `cache`. `None` until the first `/public/latest` fetch succeeds. Walking
    /// backwards from here guarantees distinct rounds per 32-byte block.
    current_round: Option<u64>,
}

impl DrandSource {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            cache: Vec::new(),
            current_round: None,
        }
    }

    /// Bootstrap fetch against `/public/latest` to learn the current round
    /// number and its 32 randomness bytes.
    fn fetch_latest(&self) -> Result<(u64, Vec<u8>), SourceError> {
        fetch_round_at(&self.client, DRAND_URL)
    }

    /// Ensure `cache` holds at least `count` bytes, walking *backwards* through
    /// round numbers so each 32-byte block comes from a distinct round (never
    /// `R || R`). See [`assemble_bytes`] for the network-free core.
    fn ensure_bytes(&mut self, count: usize) -> Result<(), SourceError> {
        // Bootstrap from /public/latest so we know which round we're on.
        if self.current_round.is_none() {
            let (round, bytes) = self.fetch_latest()?;
            self.cache.extend_from_slice(&bytes);
            self.current_round = Some(round);
        }

        // Borrow `self.client` (shared) separately from `self.cache` (mut) so
        // the fetch closure can drive the network while the helper owns the
        // mutable byte buffer — the two borrows are of disjoint fields.
        let client = &self.client;
        let mut round = self.current_round.expect("round initialized above");
        assemble_bytes(count, &mut round, &mut self.cache, |r| {
            // `/public/{round-1}` ...; keep the returned bytes (round ignored,
            // we already know it from the URL).
            let (_round, bytes) = fetch_round_at(client, &round_url(r))?;
            Ok(bytes)
        })?;
        self.current_round = Some(round);
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

/// Build a per-round `/public/{round}` URL from the latest URL, swapping the
/// trailing `latest` segment for the numeric round. Single source of truth:
/// only [`DRAND_URL`] is hardcoded.
fn round_url(round: u64) -> String {
    let base = DRAND_URL.strip_suffix("latest").unwrap_or(DRAND_URL);
    format!("{base}{round}")
}

/// GET a drand round endpoint and decode its 32-byte `randomness` field,
/// returning `(round_number, randomness_bytes)`.
fn fetch_round_at(
    client: &reqwest::blocking::Client,
    url: &str,
) -> Result<(u64, Vec<u8>), SourceError> {
    let resp = client.get(url).send().map_err(|e| {
        SourceError::GenerationFailed(format!("drand request failed: {e}"))
    })?;

    if !resp.status().is_success() {
        return Err(SourceError::GenerationFailed(format!(
            "drand returned status {}",
            resp.status()
        )));
    }

    let body: DrandResponse = resp
        .json()
        .map_err(|e| SourceError::GenerationFailed(format!("drand response parse failed: {e}")))?;

    let bytes = hex_to_bytes(&body.randomness)?;
    Ok((body.round, bytes))
}

/// Walk drand round numbers *backwards* to assemble at least `count` bytes,
/// guaranteeing every 32-byte block comes from a distinct round.
///
/// Pure and network-free so the round-walking / offset accounting can be
/// unit-tested in isolation: `current_round` is the round number whose bytes
/// sit at the tail of `cache`, and each iteration fetches the *previous* round
/// (`current_round - 1`) via `fetch` and appends its bytes. Returns
/// [`SourceError::GenerationFailed`] — instead of wrapping — when the round
/// number would underflow below 0.
fn assemble_bytes<F>(
    count: usize,
    current_round: &mut u64,
    cache: &mut Vec<u8>,
    mut fetch: F,
) -> Result<(), SourceError>
where
    F: FnMut(u64) -> Result<Vec<u8>, SourceError>,
{
    while cache.len() < count {
        let next = current_round.checked_sub(1).ok_or_else(|| {
            SourceError::GenerationFailed(
                "drand round number underflow: exhausted historical rounds".to_string(),
            )
        })?;
        *current_round = next;
        let bytes = fetch(next)?;
        if bytes.is_empty() {
            return Err(SourceError::GenerationFailed(
                "drand round returned empty randomness".to_string(),
            ));
        }
        cache.extend_from_slice(&bytes);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression for B1: assembling 64 bytes across two rounds must yield two
    /// *distinct* 32-byte halves. The old loop hit `/public/latest` repeatedly,
    /// producing the byte-identical second block `R || R`.
    #[test]
    fn assemble_bytes_yields_distinct_round_blocks() {
        let round_a = vec![0xAAu8; 32];
        let round_b = vec![0xBBu8; 32];

        // Bootstrap state: latest round already in the cache, current_round = 10.
        let mut round = 10u64;
        let mut cache = round_a.clone();
        let mut calls = 0usize;

        assemble_bytes(64, &mut round, &mut cache, |r| {
            calls += 1;
            // Each fetch must target the *previous* round number, walking down.
            assert_eq!(r, 10 - calls as u64, "must walk to previous round");
            Ok(round_b.clone())
        })
        .expect("assembly should succeed");

        assert_eq!(cache.len(), 64);
        assert_eq!(&cache[..32], &round_a[..], "first half = bootstrap round");
        assert_ne!(
            &cache[..32],
            &cache[32..],
            "two halves must be distinct rounds, never R || R"
        );
        assert_eq!(round, 9, "current_round walked back by exactly one");
        assert_eq!(calls, 1, "exactly one extra round fetched");
    }

    /// Regression for B1 guard: when `count` cannot be satisfied, the round
    /// number must error rather than wrap below 0 (u64 underflow).
    #[test]
    fn assemble_bytes_errors_on_round_underflow() {
        let mut round = 0u64;
        let mut cache: Vec<u8> = Vec::new();

        let res = assemble_bytes(64, &mut round, &mut cache, |_| {
            panic!("fetch must not be called when the round number underflows");
        });

        assert!(
            matches!(res, Err(SourceError::GenerationFailed(_))),
            "underflow must surface as GenerationFailed, got {res:?}"
        );
        assert_eq!(round, 0, "round number must not wrap on underflow");
        assert!(cache.is_empty(), "cache untouched on underflow");
    }
}
