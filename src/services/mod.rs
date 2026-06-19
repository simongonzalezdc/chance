pub mod dto;

use crate::core::range::uniform_entropy_bits;
use crate::core::source::Source;
use crate::core::SourceError;
use crate::methods::*;
use crate::sources::create_source;
use dto::*;
use std::time::Instant;

/// Enforce a server-side cap on an untrusted request field. Rejects before any
/// allocation or source work runs so attacker-controlled bounds cannot drive
/// loops or allocations.
fn require_in_range<T: PartialOrd + std::fmt::Display>(
    name: &str,
    val: T,
    lo: T,
    hi: T,
) -> Result<(), SourceError> {
    if val < lo || val > hi {
        return Err(SourceError::InvalidInput(format!(
            "{name} {val} is out of range {lo}..={hi}"
        )));
    }
    Ok(())
}

pub fn make_source(req: &SourceRequest) -> Result<Box<dyn Source>, SourceError> {
    let name = req.source.as_deref().unwrap_or("os-csprng");
    create_source(name, req.seed.as_deref())
}

/// Map a [`SourceHealth`](crate::core::SourceHealth) onto the lowercase string
/// used in provenance / health JSON (`healthy` / `degraded` / `unavailable`).
fn health_status_str(h: crate::core::SourceHealth) -> &'static str {
    use crate::core::SourceHealth;
    match h {
        SourceHealth::Healthy => "healthy",
        SourceHealth::Degraded => "degraded",
        SourceHealth::Unavailable => "unavailable",
    }
}

pub fn build_provenance(source: &dyn Source, entropy_bits: f64, latency_ms: f64) -> Provenance {
    Provenance {
        source: source.name(),
        source_kind: source.kind().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        entropy_bits,
        request_id: generate_request_id(),
        seed: source.seed(),
        latency_ms,
        // Self-reported health of *this* response's source. Live network
        // probing is exposed separately via [`health`].
        source_health: health_status_str(source.health()).to_string(),
    }
}

fn generate_request_id() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    format!("req_{}", hex::encode(&bytes))
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            s.push(HEX[(b >> 4) as usize] as char);
            s.push(HEX[(b & 0x0f) as usize] as char);
        }
        s
    }
}

/// Shannon entropy (in bits) of drawing `k` *distinct* items from a set of
/// `n` (sampling **without** replacement): `log2(n! / (n-k)!) =
/// sum_{i=0}^{k-1} log2(n - i)`. For a full permutation pass `k == n` to get
/// `log2(n!)`. `k` is capped at `n` so the index never underflows (you cannot
/// draw more distinct items than exist), which keeps callers such as lottery
/// that only validate `pick <= 20` panic-free when `pick > pool`.
fn log2_permutations(n: usize, k: usize) -> f64 {
    let k = k.min(n);
    (0..k).map(|i| ((n - i) as f64).log2()).sum()
}

fn entropy_bits_dice(notation: &str) -> f64 {
    use crate::methods::dice::ast::*;
    use crate::methods::dice::parser::parse;

    if let Ok(Expr::Sum(terms)) = parse(notation) {
        terms
            .iter()
            .filter_map(|(_, term)| match term {
                Term::Dice(dice) => {
                    let sides = match dice.size {
                        DieSize::Sides(n) => n as f64,
                        DieSize::Percentile => 100.0,
                        DieSize::Fudge => 3.0,
                    };
                    Some(dice.count as f64 * uniform_entropy_bits(sides as u64))
                }
                _ => None,
            })
            .sum()
    } else {
        0.0
    }
}

pub fn roll(req: &RollRequest) -> Result<ApiResponse<RollResultDto>, SourceError> {
    require_in_range("notation_len", req.notation.len(), 1usize, 256usize)?;
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let result = roll_dice(source.as_mut(), &req.notation)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    let dto = RollResultDto {
        total: result.total,
        rolls: result
            .rolls
            .iter()
            .map(|r| DieRollDto {
                value: r.value,
                size: r.size,
                exploded: r.exploded,
                rerolled: r.rerolled,
            })
            .collect(),
        dropped: result.dropped.iter().map(|r| r.value).collect(),
        modifier_total: result.modifier_total,
        success_count: result.success_count,
    };

    Ok(ApiResponse {
        result: dto,
        provenance: build_provenance(source.as_ref(), entropy_bits_dice(&req.notation), latency),
    })
}

pub fn flip(req: &FlipRequest) -> Result<ApiResponse<Vec<String>>, SourceError> {
    require_in_range("times", req.times, 1u64, 1_000_000u64)?;
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let flips = flip_n(source.as_mut(), req.times)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;
    let out: Vec<String> = flips.iter().map(|f| f.to_string()).collect();

    Ok(ApiResponse {
        result: out,
        provenance: build_provenance(source.as_ref(), req.times as f64, latency),
    })
}

pub fn draw(req: &DrawRequest) -> Result<ApiResponse<Vec<String>>, SourceError> {
    require_in_range("count", req.count, 1usize, 52usize)?;
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let cards = draw_cards(source.as_mut(), req.count)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;
    let out: Vec<String> = cards.iter().map(|c| c.to_string()).collect();

    Ok(ApiResponse {
        result: out,
        provenance: build_provenance(
            source.as_ref(),
            log2_permutations(52, req.count),
            latency,
        ),
    })
}

pub fn pick(req: &ListRequest) -> Result<ApiResponse<Vec<String>>, SourceError> {
    if req.items.is_empty() {
        return Err(SourceError::InvalidInput("cannot pick from an empty list".to_string()));
    }
    if req.items.len() > 100_000 {
        return Err(SourceError::InvalidInput(format!(
            "cannot pick from {} items (max 100000)",
            req.items.len()
        )));
    }
    if req.count < 1 || req.count > req.items.len() {
        return Err(SourceError::InvalidInput(format!(
            "cannot pick {} from {} available items",
            req.count,
            req.items.len()
        )));
    }
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let winners = pick_distinct(source.as_mut(), &req.items, req.count)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    Ok(ApiResponse {
        result: winners,
        provenance: build_provenance(
            source.as_ref(),
            log2_permutations(req.items.len(), req.count),
            latency,
        ),
    })
}

pub fn shuffle(req: &ShuffleRequest) -> Result<ApiResponse<Vec<String>>, SourceError> {
    require_in_range("items_len", req.items.len(), 0usize, 100_000usize)?;
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let mut items = req.items.clone();
    crate::methods::shuffle::shuffle(source.as_mut(), &mut items)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    Ok(ApiResponse {
        result: items,
        provenance: build_provenance(
            source.as_ref(),
            log2_permutations(req.items.len(), req.items.len()),
            latency,
        ),
    })
}

pub fn integer(req: &IntRequest) -> Result<ApiResponse<i64>, SourceError> {
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let value = random_i64(source.as_mut(), req.min, req.max)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;
    let range = (req.max - req.min + 1).max(1) as f64;

    Ok(ApiResponse {
        result: value,
        provenance: build_provenance(source.as_ref(), range.log2(), latency),
    })
}

pub fn bytes(req: &BytesRequest) -> Result<ApiResponse<String>, SourceError> {
    require_in_range("count", req.count, 1usize, 1_048_576usize)?;
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let bytes = random_bytes(source.as_mut(), req.count)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    let encoded = match req.encoding.as_str() {
        "base64" => bytes_to_base64(&bytes),
        _ => bytes_to_hex(&bytes),
    };

    Ok(ApiResponse {
        result: encoded,
        provenance: build_provenance(source.as_ref(), req.count as f64 * 8.0, latency),
    })
}

pub fn uuid(req: &UuidRequest) -> Result<ApiResponse<String>, SourceError> {
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let uuid = generate_uuid(source.as_mut(), req.version)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    Ok(ApiResponse {
        result: uuid.to_string(),
        provenance: build_provenance(source.as_ref(), 128.0, latency),
    })
}

pub fn password(req: &PasswordRequest) -> Result<ApiResponse<String>, SourceError> {
    require_in_range("length", req.length, 1usize, 1024usize)?;
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let options = PasswordOptions {
        length: req.length,
        symbols: req.symbols,
        ..Default::default()
    };
    let pw = generate_password(source.as_mut(), &options)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;
    let alphabet_size = if options.symbols { 26 + 26 + 10 + 25 } else { 26 + 26 + 10 } as f64;

    Ok(ApiResponse {
        result: pw,
        provenance: build_provenance(
            source.as_ref(),
            req.length as f64 * alphabet_size.log2(),
            latency,
        ),
    })
}

pub fn runes(req: &RunesRequest) -> Result<ApiResponse<Vec<String>>, SourceError> {
    require_in_range("count", req.count, 1usize, 24usize)?;
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let runes = draw_runes(source.as_mut(), req.count)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;
    let out: Vec<String> = runes.iter().map(|r| r.to_string()).collect();

    Ok(ApiResponse {
        result: out,
        provenance: build_provenance(source.as_ref(), req.count as f64 * 24f64.log2(), latency),
    })
}

pub fn iching(req: &IchingRequest) -> Result<ApiResponse<IchingResultDto>, SourceError> {
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let reading = cast_iching(source.as_mut(), &req.method)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    let dto = IchingResultDto {
        primary: reading.primary,
        primary_name: reading.hexagram_name().to_string(),
        transformed: reading.transformed,
        method: reading.method.to_string(),
        lines: reading
            .lines
            .iter()
            .map(|l| IchingLineDto {
                value: l.value,
                yang: l.yang,
                changing: l.changing,
            })
            .collect(),
    };

    Ok(ApiResponse {
        result: dto,
        provenance: build_provenance(source.as_ref(), 6.0, latency),
    })
}

pub fn tarot(req: &TarotRequest) -> Result<ApiResponse<Vec<TarotCardDto>>, SourceError> {
    require_in_range("count", req.count, 1usize, 78usize)?;
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let cards = draw_tarot(source.as_mut(), req.count)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;
    let out: Vec<TarotCardDto> = cards
        .iter()
        .map(|c| TarotCardDto {
            name: c.name.clone(),
            upright: c.upright,
        })
        .collect();

    Ok(ApiResponse {
        result: out,
        provenance: build_provenance(source.as_ref(), log2_permutations(78, req.count), latency),
    })
}

pub fn dominoes(req: &DominoesRequest) -> Result<ApiResponse<Vec<DominoDto>>, SourceError> {
    require_in_range("set", req.set, 0u8, 18u8)?;
    require_in_range("count", req.count, 1usize, 1000usize)?;
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let dominoes = draw_dominoes(source.as_mut(), req.set, req.count)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;
    let set_size = ((req.set as u64 + 1) * (req.set as u64 + 2) / 2) as f64;
    let out: Vec<DominoDto> = dominoes
        .iter()
        .map(|d| DominoDto {
            left: d.left,
            right: d.right,
        })
        .collect();

    Ok(ApiResponse {
        result: out,
        provenance: build_provenance(
            source.as_ref(),
            crate::services::log2_permutations(set_size as usize, req.count),
            latency,
        ),
    })
}

pub fn roulette(req: &RouletteRequest) -> Result<ApiResponse<RouletteResultDto>, SourceError> {
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let result = spin_roulette(source.as_mut(), &req.variant)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    let dto = RouletteResultDto {
        number: result.number,
        color: result.color.to_string(),
        variant: result.variant.to_string(),
        house_edge_percent: result.house_edge_percent,
    };

    Ok(ApiResponse {
        result: dto,
        provenance: build_provenance(source.as_ref(), (37.0f64).log2(), latency),
    })
}

pub fn lottery(req: &LotteryRequest) -> Result<ApiResponse<LotteryResultDto>, SourceError> {
    require_in_range("pool", req.pool, 1u8, 99u8)?;
    require_in_range("pick", req.pick, 1usize, 20usize)?;
    if let Some(bonus_pool) = req.bonus_pool {
        require_in_range("bonus_pool", bonus_pool, 1u8, 99u8)?;
    }
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let result = draw_lottery(source.as_mut(), req.pool, req.pick, req.bonus_pool)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    let dto = LotteryResultDto {
        numbers: result.numbers,
        bonus: result.bonus,
    };

    Ok(ApiResponse {
        result: dto,
        provenance: build_provenance(
            source.as_ref(),
            log2_permutations(req.pool as usize, req.pick),
            latency,
        ),
    })
}

pub fn knucklebones(req: &KnucklebonesRequest) -> Result<ApiResponse<Vec<u8>>, SourceError> {
    require_in_range("count", req.count, 1usize, 1000usize)?;
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let result = cast_knucklebones(source.as_mut(), req.count)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    Ok(ApiResponse {
        result: result.values,
        provenance: build_provenance(source.as_ref(), req.count as f64 * 4f64.log2(), latency),
    })
}

pub fn teetotum(req: &TeetotumRequest) -> Result<ApiResponse<String>, SourceError> {
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let result = if req.dreidel {
        spin_dreidel(source.as_mut())
    } else {
        spin_teetotum(source.as_mut())
    }?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    Ok(ApiResponse {
        result: result.face.to_string(),
        provenance: build_provenance(source.as_ref(), 2.0, latency),
    })
}

pub fn cowrie(req: &CowrieRequest) -> Result<ApiResponse<CowrieResultDto>, SourceError> {
    require_in_range("shells", req.shells, 1usize, 64usize)?;
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let result = cast_cowrie(source.as_mut(), req.shells)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    let dto = CowrieResultDto {
        shells: result.shells,
        open_count: result.open_count,
        meaning: result.meaning.to_string(),
    };

    Ok(ApiResponse {
        result: dto,
        provenance: build_provenance(source.as_ref(), req.shells as f64, latency),
    })
}

pub fn lots(req: &ListRequest) -> Result<ApiResponse<Vec<String>>, SourceError> {
    if req.items.is_empty() {
        return Err(SourceError::InvalidInput("cannot draw lots from an empty list".to_string()));
    }
    if req.items.len() > 100_000 {
        return Err(SourceError::InvalidInput(format!(
            "cannot draw lots from {} items (max 100000)",
            req.items.len()
        )));
    }
    if req.count < 1 || req.count > req.items.len() {
        return Err(SourceError::InvalidInput(format!(
            "cannot draw {} lots from {} available items",
            req.count,
            req.items.len()
        )));
    }
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let winners = draw_lots(source.as_mut(), &req.items, req.count)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    Ok(ApiResponse {
        result: winners,
        provenance: build_provenance(
            source.as_ref(),
            log2_permutations(req.items.len(), req.count),
            latency,
        ),
    })
}

pub fn source_names() -> Vec<String> {
    crate::sources::source_names().iter().map(|s| s.to_string()).collect()
}

pub fn health() -> serde_json::Value {
    let names = crate::sources::source_names();
    let mut sources = serde_json::Map::new();
    let mut all_healthy = true;

    for &name in names {
        // `mix` is a composite that needs a sub-source spec, so probe it with a
        // representative healthy pairing rather than the bare (unconstructable)
        // name `mix`.
        let probe_name = if name == "mix" {
            "mix:os-csprng,chacha20"
        } else {
            name
        };
        let status = probe_source(name, probe_name);
        if status != "healthy" {
            all_healthy = false;
        }
        sources.insert(
            name.to_string(),
            serde_json::Value::String(status.to_string()),
        );
    }

    serde_json::json!({
        "status": if all_healthy { "ok" } else { "degraded" },
        "sources": sources,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })
}

/// Probe one source by name inside a short timeout. Constructs the source via
/// the normal [`make_source`] path and runs a single 8-byte `fill_bytes` probe.
/// Returns a stable status string for the health map:
/// - `healthy`     — constructed and produced bytes,
/// - `unavailable` — constructed but the probe returned an error,
/// - `unreachable` — the probe did not finish within the timeout (e.g. a
///                   network beacon with no connectivity).
fn probe_source(display_name: &str, source_name: &str) -> &'static str {
    use std::sync::mpsc;
    use std::time::Duration;

    let key = source_name.to_string();
    let (tx, rx) = mpsc::channel::<Result<(), SourceError>>();

    let handle = std::thread::Builder::new()
        .name(format!("chance-probe-{display_name}"))
        .spawn(move || {
            let outcome = (|| {
                let req = SourceRequest {
                    source: Some(key.clone()),
                    seed: None,
                };
                let mut src = make_source(&req)?;
                let mut buf = [0u8; 8];
                src.fill_bytes(&mut buf)?;
                Ok::<(), SourceError>(())
            })();
            // `send` only fails if the receiver was dropped (timeout path),
            // in which case there is nothing useful to do.
            let _ = tx.send(outcome);
        });

    if handle.is_err() {
        return "unavailable";
    }

    match rx.recv_timeout(Duration::from_millis(800)) {
        Ok(Ok(())) => "healthy",
        Ok(Err(_)) => "unavailable",
        Err(mpsc::RecvTimeoutError::Timeout) => "unreachable",
        Err(mpsc::RecvTimeoutError::Disconnected) => "unavailable",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::SourceError;

    #[test]
    fn bytes_count_over_cap_rejected() {
        let req = BytesRequest {
            source: SourceRequest::default(),
            count: 1_048_577,
            encoding: "hex".to_string(),
        };
        let err = bytes(&req).unwrap_err();
        assert!(matches!(err, SourceError::InvalidInput(_)));
    }

    #[test]
    fn cowrie_shells_over_cap_rejected() {
        let req = CowrieRequest {
            source: SourceRequest::default(),
            shells: 65,
        };
        let err = cowrie(&req).unwrap_err();
        assert!(matches!(err, SourceError::InvalidInput(_)));
    }

    #[test]
    fn pick_count_exceeds_items_rejected() {
        let req = ListRequest {
            source: SourceRequest::default(),
            items: vec!["a".to_string()],
            count: 5,
        };
        let err = pick(&req).unwrap_err();
        assert!(matches!(err, SourceError::InvalidInput(_)));
    }

    /// W2: `health()` must actually probe sources and return an honest overall
    /// status with a non-empty per-source map (not a static `{"status":"ok"}`).
    #[test]
    fn health_probes_each_source() {
        let h = health();
        let status = h["status"].as_str().expect("status present");
        assert!(
            status == "ok" || status == "degraded",
            "overall status must be ok or degraded, got {status}"
        );
        let sources = h["sources"].as_object().expect("sources is an object");
        assert!(!sources.is_empty(), "sources map must be non-empty");
        // The OS CSPRNG is always locally available and must be healthy.
        assert_eq!(sources["os-csprng"].as_str(), Some("healthy"));
        // Every per-source value is one of the documented statuses.
        for (_, v) in sources {
            let s = v.as_str().expect("source status is a string");
            assert!(
                matches!(s, "healthy" | "degraded" | "unavailable" | "unreachable"),
                "unexpected source status {s}"
            );
        }
    }

    /// W7: shuffling a 52-card deck yields exactly log2(52!) bits of entropy
    /// (true without-replacement accounting), within float tolerance.
    #[test]
    fn shuffle_entropy_is_log2_factorial() {
        let items: Vec<String> = (0..52).map(|i| i.to_string()).collect();
        let req = ShuffleRequest {
            source: SourceRequest::default(),
            items,
        };
        let resp = shuffle(&req).expect("shuffle should succeed");
        let expected: f64 = (1..=52).map(|i| (i as f64).log2()).sum();
        assert!(
            (resp.provenance.entropy_bits - expected).abs() < 1e-6,
            "shuffle entropy {} != log2(52!) {}",
            resp.provenance.entropy_bits,
            expected
        );
        // Provenance now carries the source's self-reported health.
        assert_eq!(resp.provenance.source_health, "healthy");
    }

    /// W7: picking 2 distinct items from 4 is log2(4) + log2(3), i.e. the
    /// permutation entropy log2(4P2) — not the with-replacement 2*log2(4).
    #[test]
    fn pick_entropy_is_permutation() {
        let items: Vec<String> = vec!["a", "b", "c", "d"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let req = ListRequest {
            source: SourceRequest::default(),
            items,
            count: 2,
        };
        let resp = pick(&req).expect("pick should succeed");
        let expected = (4.0f64).log2() + (3.0f64).log2();
        assert!(
            (resp.provenance.entropy_bits - expected).abs() < 1e-9,
            "pick(2 from 4) entropy {} != log2(4)+log2(3) {}",
            resp.provenance.entropy_bits,
            expected
        );
    }

    /// W7: `log2_permutations` directly. log2(5P2) = log2(5)+log2(4) and a full
    /// permutation of 3 is log2(3!) = log2(6).
    #[test]
    fn log2_permutations_helper_is_correct() {
        assert!((log2_permutations(5, 2) - ((5.0f64).log2() + (4.0f64).log2())).abs() < 1e-9);
        assert!((log2_permutations(3, 3) - (6.0f64).log2()).abs() < 1e-9);
        // k > n caps at n, never underflows.
        assert!((log2_permutations(3, 10) - (6.0f64).log2()).abs() < 1e-9);
        assert_eq!(log2_permutations(0, 0), 0.0);
    }
}
