pub mod dto;

use crate::core::range::uniform_entropy_bits;
use crate::core::source::Source;
use crate::core::SourceError;
use crate::methods::*;
use crate::sources::create_source;
use dto::*;
use std::time::Instant;

pub fn make_source(req: &SourceRequest) -> Result<Box<dyn Source>, SourceError> {
    let name = req.source.as_deref().unwrap_or("os-csprng");
    create_source(name, req.seed.as_deref())
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
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let cards = draw_cards(source.as_mut(), req.count)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;
    let out: Vec<String> = cards.iter().map(|c| c.to_string()).collect();

    Ok(ApiResponse {
        result: out,
        provenance: build_provenance(
            source.as_ref(),
            req.count as f64 * (52.0f64.log2()),
            latency,
        ),
    })
}

pub fn pick(req: &ListRequest) -> Result<ApiResponse<Vec<String>>, SourceError> {
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let winners = pick_distinct(source.as_mut(), &req.items, req.count)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    Ok(ApiResponse {
        result: winners,
        provenance: build_provenance(
            source.as_ref(),
            (req.items.len() as f64).log2() * req.count as f64,
            latency,
        ),
    })
}

pub fn shuffle(req: &ShuffleRequest) -> Result<ApiResponse<Vec<String>>, SourceError> {
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let mut items = req.items.clone();
    crate::methods::shuffle::shuffle(source.as_mut(), &mut items)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    Ok(ApiResponse {
        result: items,
        provenance: build_provenance(
            source.as_ref(),
            (req.items.len() as f64).log2() * req.items.len() as f64,
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
        provenance: build_provenance(source.as_ref(), req.count as f64 * 78f64.log2(), latency),
    })
}

pub fn dominoes(req: &DominoesRequest) -> Result<ApiResponse<Vec<DominoDto>>, SourceError> {
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
            req.count as f64 * set_size.log2(),
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
            req.pick as f64 * (req.pool as f64).log2(),
            latency,
        ),
    })
}

pub fn knucklebones(req: &KnucklebonesRequest) -> Result<ApiResponse<Vec<u8>>, SourceError> {
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
    let start = Instant::now();
    let mut source = make_source(&req.source)?;
    let winners = draw_lots(source.as_mut(), &req.items, req.count)?;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    Ok(ApiResponse {
        result: winners,
        provenance: build_provenance(
            source.as_ref(),
            (req.items.len() as f64).log2() * req.count as f64,
            latency,
        ),
    })
}

pub fn source_names() -> Vec<String> {
    crate::sources::source_names().iter().map(|s| s.to_string()).collect()
}

pub fn health() -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })
}
