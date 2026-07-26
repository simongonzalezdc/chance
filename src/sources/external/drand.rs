use crate::core::{Source, SourceError, SourceHealth, SourceKind};
use serde::Deserialize;

const DRAND_URL: &str = "https://api.drand.sh/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971/public/latest";

/// drand / League of Entropy — publicly verifiable distributed randomness beacon.
///
/// By default this source trusts the HTTPS endpoint and does not verify the
/// drand BLS threshold signature. Enable the `drand-verify` cargo feature to
/// fetch each chain's public key from `/info` and cryptographically verify
/// every fetched round's signature before its bytes are trusted (see
/// [`DrandSource`] verification path, gated on `drand-verify`). Verification
/// currently supports the G1-signature schemes (quicknet /
/// `bls-unchained-g1-rfc9380`); other schemes are rejected when the feature
/// is on.
///
/// The beacon emits exactly 32 bytes per round and a new round only every ~3s
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
    /// Cached chain verification material, fetched once from `/info`. Only
    /// present under the `drand-verify` feature; when the feature is off the
    /// source trusts HTTPS and these fields do not exist.
    #[cfg(feature = "drand-verify")]
    public_key: Option<Vec<u8>>,
    #[cfg(feature = "drand-verify")]
    scheme_g1: Option<bool>,
}

impl DrandSource {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            cache: Vec::new(),
            current_round: None,
            #[cfg(feature = "drand-verify")]
            public_key: None,
            #[cfg(feature = "drand-verify")]
            scheme_g1: None,
        }
    }

    /// Bootstrap fetch against `/public/latest` to learn the current round
    /// number and its 32 randomness bytes.
    fn fetch_latest(&self) -> Result<(u64, Vec<u8>, Vec<u8>), SourceError> {
        fetch_round_at(&self.client, DRAND_URL)
    }

    /// Ensure `cache` holds at least `count` bytes, walking *backwards* through
    /// round numbers so each 32-byte block comes from a distinct round (never
    /// `R || R`). See [`assemble_bytes`] for the network-free core.
    fn ensure_bytes(&mut self, count: usize) -> Result<(), SourceError> {
        // Under `drand-verify`: load chain info (public key + scheme) once so we
        // can validate every fetched round's BLS signature before trusting its
        // bytes. When the feature is off this line does not exist and we trust
        // HTTPS exactly as before.
        #[cfg(feature = "drand-verify")]
        let verify_ctx = self.load_verify_ctx()?;

        // Bootstrap from /public/latest so we know which round we're on.
        if self.current_round.is_none() {
            let (round, bytes, signature) = self.fetch_latest()?;
            #[cfg(not(feature = "drand-verify"))]
            let _ = &signature;
            #[cfg(feature = "drand-verify")]
            verify_ctx.verify(round, &signature)?;
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
            let (fetched_round, bytes, signature) = fetch_round_at(client, &round_url(r))?;
            #[cfg(feature = "drand-verify")]
            verify_ctx.verify(fetched_round, &signature)?;
            #[cfg(not(feature = "drand-verify"))]
            {
                let _ = (fetched_round, signature);
            }
            Ok(bytes)
        })?;
        self.current_round = Some(round);
        Ok(())
    }
}

impl Default for DrandSource {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "drand-verify")]
impl DrandSource {
    /// Fetch `/info` once and cache the chain's public key plus whether the
    /// chain is a G1-signature scheme this verifier supports. Returns an owned
    /// [`VerifyCtx`] (cloned from the cache) so verification can run inside the
    /// `assemble_bytes` closure without borrowing `self` alongside
    /// `&mut self.cache`.
    fn load_verify_ctx(&mut self) -> Result<VerifyCtx, SourceError> {
        if self.public_key.is_none() {
            let info = fetch_chain_info(&self.client)?;
            let pk = hex_to_bytes(&info.public_key).map_err(|_| {
                SourceError::GenerationFailed("drand signature verification failed".to_string())
            })?;
            self.public_key = Some(pk);
            self.scheme_g1 = Some(info.scheme_g1());
        }
        Ok(VerifyCtx {
            pk_bytes: self.public_key.clone().expect("populated above"),
            scheme_g1: self.scheme_g1.unwrap_or(false),
        })
    }
}

/// Cached chain parameters needed to verify a fetched round's signature. Owned
/// (cloned from [`DrandSource`]) so verification can run inside a closure.
#[cfg(feature = "drand-verify")]
struct VerifyCtx {
    pk_bytes: Vec<u8>,
    scheme_g1: bool,
}

#[cfg(feature = "drand-verify")]
impl VerifyCtx {
    /// Verify the drand BLS signature for `round` against the cached public key.
    ///
    /// `signature` is the raw `signature` field bytes: for the unchained-g1 /
    /// quicknet schemes this is a 48-byte compressed G1 point. (The
    /// `randomness` field is SHA-256 of the signature and is *not* a valid
    /// signature point.)
    fn verify(&self, round: u64, signature: &[u8]) -> Result<(), SourceError> {
        use blst::min_sig::{PublicKey, Signature};
        use blst::BLST_ERROR;

        // drand unchained-g1 / quicknet: signature in G1, public key in G2,
        // message hashed to G1. The DST is therefore the *G1* DST. We verified
        // this empirically against a live quicknet vector using a reference
        // BLS12-381 implementation: SHA-256(round_be) verifies with the G1 DST
        // and fails with the G2 DST.
        const DST: &[u8] = b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_";

        if !self.scheme_g1 {
            return Err(SourceError::GenerationFailed(
                "drand scheme not supported for verification".to_string(),
            ));
        }

        // Signed message = SHA-256 of the round number as 8-byte big-endian.
        let msg = sha256_oneshot(&round.to_be_bytes());

        let sig = Signature::from_bytes(signature).map_err(|_| {
            SourceError::GenerationFailed("drand signature verification failed".to_string())
        })?;
        let pk = PublicKey::from_bytes(&self.pk_bytes).map_err(|_| {
            SourceError::GenerationFailed("drand signature verification failed".to_string())
        })?;

        match sig.verify(true, &msg, DST, b"", &pk, true) {
            BLST_ERROR::BLST_SUCCESS => Ok(()),
            _ => Err(SourceError::GenerationFailed(
                "drand signature verification failed".to_string(),
            )),
        }
    }
}

/// One-shot SHA-256 over `msg`, writing 32 bytes to the returned array.
///
/// We reuse blst's bundled `blst_sha256` so the `drand-verify` feature pulls in
/// no extra dependency: `sha2` is gated behind `mixing`, not `drand-verify`, so
/// it is unavailable under a minimal `--no-default-features --features
/// drand-verify` build.
#[cfg(feature = "drand-verify")]
fn sha256_oneshot(msg: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    // SAFETY: blst_sha256 writes exactly 32 bytes to `out` for an input of
    // length `msg.len()`; both pointers are valid for the stated sizes.
    unsafe {
        blst::blst_sha256(out.as_mut_ptr(), msg.as_ptr(), msg.len());
    }
    out
}

#[cfg(feature = "drand-verify")]
fn info_url() -> String {
    let base = DRAND_URL.strip_suffix("public/latest").unwrap_or(DRAND_URL);
    format!("{base}info")
}

#[cfg(feature = "drand-verify")]
fn fetch_chain_info(client: &reqwest::blocking::Client) -> Result<DrandChainInfo, SourceError> {
    let resp = client.get(info_url()).send().map_err(|e| {
        tracing::warn!(error = %e, "drand /info request failed");
        SourceError::GenerationFailed("drand chain info request failed".to_string())
    })?;
    if !resp.status().is_success() {
        return Err(SourceError::GenerationFailed(format!(
            "drand chain info returned status {}",
            resp.status()
        )));
    }
    resp.json::<DrandChainInfo>().map_err(|e| {
        tracing::warn!(error = %e, "drand /info parse failed");
        SourceError::GenerationFailed("drand chain info parse failed".to_string())
    })
}

/// `/info` response (only the fields we need). drand exposes the scheme id
/// either nested as `{"scheme":{"id":...}}` or at the top level as
/// `schemeID`; we accept both.
#[cfg(feature = "drand-verify")]
#[derive(Deserialize)]
struct DrandChainInfo {
    public_key: String,
    #[serde(default)]
    scheme: Option<DrandScheme>,
    #[serde(default, rename = "schemeID")]
    scheme_id: Option<String>,
}

#[cfg(feature = "drand-verify")]
impl DrandChainInfo {
    /// True if the chain uses G1 signatures (the only family this verifier
    /// supports).
    fn scheme_g1(&self) -> bool {
        let id = self
            .scheme
            .as_ref()
            .and_then(|s| s.id.as_deref())
            .or(self.scheme_id.as_deref());
        id.map(|s| s.contains("g1")).unwrap_or(false)
    }
}

#[cfg(feature = "drand-verify")]
#[derive(Deserialize)]
struct DrandScheme {
    #[serde(default)]
    id: Option<String>,
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
        let byte = u8::from_str_radix(&hex[i..i + 2], 16)
            .map_err(|e| SourceError::GenerationFailed(format!("invalid hex: {e}")))?;
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

/// GET a drand round endpoint and decode its `randomness` field (the SHA-256 of
/// the BLS signature, used as the beacon's output bytes) and its `signature`
/// field (the raw BLS signature, used for verification under `drand-verify`).
/// Returns `(round_number, randomness_bytes, signature_bytes)`.
fn fetch_round_at(
    client: &reqwest::blocking::Client,
    url: &str,
) -> Result<(u64, Vec<u8>, Vec<u8>), SourceError> {
    let resp = client
        .get(url)
        .send()
        .map_err(|e| SourceError::GenerationFailed(format!("drand request failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(SourceError::GenerationFailed(format!(
            "drand returned status {}",
            resp.status()
        )));
    }

    let body: DrandResponse = resp
        .json()
        .map_err(|e| SourceError::GenerationFailed(format!("drand response parse failed: {e}")))?;

    let randomness = hex_to_bytes(&body.randomness)?;
    let signature = hex_to_bytes(&body.signature)?;
    Ok((body.round, randomness, signature))
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

/// W11: BLS signature verification (`drand-verify` feature). These tests are
/// fully offline — they exercise the real `blst` verification path against a
/// permanent quicknet (`bls-unchained-g1-rfc9380`) test vector. The vector was
/// fetched live and independently verified with a reference BLS12-381
/// implementation *before* being embedded, so a passing
/// `quicknet_signature_verifies` proves the verify path is wired correctly,
/// and a failing one proves the test is a real check (not a tautology).
#[cfg(all(test, feature = "drand-verify"))]
mod verify_tests {
    use super::*;

    // quicknet chain public key (permanent, 96-byte G2 point) from /info.
    const QUICKNET_PK: &str = "83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a";
    // A signed round (48-byte G1 signature); the (round, signature) pair is
    // valid forever once the beacon signed it.
    const ROUND: u64 = 29695341;
    const SIG: &str = "973dcef6c129441a36381a6341e1bbf10ee8a605c74830fbd6bd3981d52d3fdb690494899fb213cc861b8deb4d59f63c";
    // SHA-256 of the 8-byte big-endian encoding of `ROUND` (the signed message).
    const ROUND_MSG_SHA256: &str =
        "4acd4994dec268e54142e8d19072e46e64c9f55a965abc7a36bc533ec80bb93d";

    fn ctx() -> VerifyCtx {
        VerifyCtx {
            pk_bytes: hex_to_bytes(QUICKNET_PK).unwrap(),
            scheme_g1: true,
        }
    }

    #[test]
    fn sha256_oneshot_matches_node_crypto() {
        // Guards that blst's blst_sha256 is standard SHA-256 (used for the
        // signed-message hash). Computed independently with Node's crypto.
        let msg = sha256_oneshot(&ROUND.to_be_bytes());
        let expected = hex_to_bytes(ROUND_MSG_SHA256).unwrap();
        assert_eq!(
            &msg[..],
            &expected[..],
            "blst_sha256 must equal standard SHA-256"
        );
    }

    #[test]
    fn quicknet_signature_verifies() {
        let sig = hex_to_bytes(SIG).unwrap();
        ctx()
            .verify(ROUND, &sig)
            .expect("real quicknet signature must verify");
    }

    #[test]
    fn tampered_signature_is_rejected() {
        let mut sig = hex_to_bytes(SIG).unwrap();
        // Flip a low bit so it still decodes as a G1 point but is no longer the
        // valid signature for this round.
        let last = sig.len() - 1;
        sig[last] ^= 0x01;
        let res = ctx().verify(ROUND, &sig);
        assert!(
            matches!(res, Err(SourceError::GenerationFailed(_))),
            "tampered signature must be rejected"
        );
    }

    #[test]
    fn wrong_round_is_rejected() {
        let sig = hex_to_bytes(SIG).unwrap();
        let res = ctx().verify(ROUND + 1, &sig);
        assert!(
            matches!(res, Err(SourceError::GenerationFailed(_))),
            "signature for a different round must fail"
        );
    }

    #[test]
    fn non_g1_scheme_is_rejected() {
        let ctx = VerifyCtx {
            pk_bytes: vec![0u8; 96],
            scheme_g1: false,
        };
        let sig = hex_to_bytes(SIG).unwrap();
        let res = ctx.verify(ROUND, &sig);
        match res {
            Err(SourceError::GenerationFailed(msg)) => assert!(
                msg.contains("not supported"),
                "non-g1 scheme must be rejected with the scheme message, got {msg}"
            ),
            other => panic!("expected GenerationFailed for non-g1 scheme, got {other:?}"),
        }
    }
}
