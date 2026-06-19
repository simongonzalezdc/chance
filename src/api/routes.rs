use crate::services::dto::*;
use crate::services;
use crate::core::SourceError;
use axum::response::Json;
use axum::http::StatusCode;

pub struct AppState;

pub type ApiResult<T> = Result<Json<ApiResponse<T>>, (StatusCode, Json<serde_json::Value>)>;

/// Convert a [`SourceError`] into an HTTP error response.
///
/// **W10 (no internal leakage):** never echoes the raw `SourceError` text to
/// the client (it can contain upstream URLs, parse details, etc.). Instead it
/// logs the full error via `tracing::error!` keyed by a freshly minted
/// `request_id`, and returns only a generic message plus that id. The one
/// exception is [`SourceError::InvalidInput`], whose messages are constructed
/// from validated, user-facing field names and bounds and are safe to echo.
fn map_error(e: SourceError) -> (StatusCode, Json<serde_json::Value>) {
    let request_id = generate_request_id();

    match &e {
        SourceError::InvalidInput(msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": msg, "request_id": request_id })),
        ),
        SourceError::InvalidSource(_) | SourceError::GenerationFailed(_) => {
            tracing::error!(error = %e, request_id = %request_id, "request failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "internal error", "request_id": request_id })),
            )
        }
        SourceError::Unavailable(_) => {
            tracing::error!(error = %e, request_id = %request_id, "request failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "source unavailable", "request_id": request_id })),
            )
        }
        SourceError::UnsupportedOperation { .. } => {
            tracing::error!(error = %e, request_id = %request_id, "request failed");
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "unsupported operation", "request_id": request_id })),
            )
        }
    }
}

/// Short, opaque correlation id (`req_<16 hex>`), reused for error tracing.
fn generate_request_id() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    format!("req_{}", hex_encode(&bytes))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// Run a (possibly blocking) service call on the blocking thread pool so that
/// sources backed by `reqwest::blocking` (e.g. drand) cannot panic the
/// multi-threaded tokio runtime that axum handlers run on.
async fn run<T, F>(f: F) -> ApiResult<T>
where
    F: FnOnce() -> Result<ApiResponse<T>, SourceError> + Send + 'static,
    T: Send + 'static,
{
    match tokio::task::spawn_blocking(f).await {
        Ok(r) => r.map(Json).map_err(map_error),
        Err(e) => {
            // The JoinError is an internal runtime detail; never leak it. Log
            // the full detail keyed by an id and return a generic message.
            let request_id = generate_request_id();
            tracing::error!(error = %e, request_id = %request_id, "spawn_blocking join failed");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "internal error", "request_id": request_id })),
            ))
        }
    }
}

pub async fn roll(Json(req): Json<RollRequest>) -> ApiResult<RollResultDto> {
    run(move || services::roll(&req)).await
}

pub async fn flip(Json(req): Json<FlipRequest>) -> ApiResult<Vec<String>> {
    run(move || services::flip(&req)).await
}

pub async fn draw(Json(req): Json<DrawRequest>) -> ApiResult<Vec<String>> {
    run(move || services::draw(&req)).await
}

pub async fn pick(Json(req): Json<ListRequest>) -> ApiResult<Vec<String>> {
    run(move || services::pick(&req)).await
}

pub async fn shuffle(Json(req): Json<ShuffleRequest>) -> ApiResult<Vec<String>> {
    run(move || services::shuffle(&req)).await
}

pub async fn int(Json(req): Json<IntRequest>) -> ApiResult<i64> {
    run(move || services::integer(&req)).await
}

pub async fn bytes(Json(req): Json<BytesRequest>) -> ApiResult<String> {
    run(move || services::bytes(&req)).await
}

pub async fn uuid(Json(req): Json<UuidRequest>) -> ApiResult<String> {
    run(move || services::uuid(&req)).await
}

pub async fn password(Json(req): Json<PasswordRequest>) -> ApiResult<String> {
    run(move || services::password(&req)).await
}

pub async fn runes(Json(req): Json<RunesRequest>) -> ApiResult<Vec<String>> {
    run(move || services::runes(&req)).await
}

pub async fn iching(Json(req): Json<IchingRequest>) -> ApiResult<IchingResultDto> {
    run(move || services::iching(&req)).await
}

pub async fn tarot(Json(req): Json<TarotRequest>) -> ApiResult<Vec<TarotCardDto>> {
    run(move || services::tarot(&req)).await
}

pub async fn dominoes(Json(req): Json<DominoesRequest>) -> ApiResult<Vec<DominoDto>> {
    run(move || services::dominoes(&req)).await
}

pub async fn roulette(Json(req): Json<RouletteRequest>) -> ApiResult<RouletteResultDto> {
    run(move || services::roulette(&req)).await
}

pub async fn lottery(Json(req): Json<LotteryRequest>) -> ApiResult<LotteryResultDto> {
    run(move || services::lottery(&req)).await
}

pub async fn knucklebones(Json(req): Json<KnucklebonesRequest>) -> ApiResult<Vec<u8>> {
    run(move || services::knucklebones(&req)).await
}

pub async fn teetotum(Json(req): Json<TeetotumRequest>) -> ApiResult<String> {
    run(move || services::teetotum(&req)).await
}

pub async fn cowrie(Json(req): Json<CowrieRequest>) -> ApiResult<CowrieResultDto> {
    run(move || services::cowrie(&req)).await
}

pub async fn lots(Json(req): Json<ListRequest>) -> ApiResult<Vec<String>> {
    run(move || services::lots(&req)).await
}

pub async fn sources() -> Json<Vec<String>> {
    Json(services::source_names())
}

pub async fn health() -> Json<serde_json::Value> {
    Json(services::health())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::SourceError;

    /// InvalidInput messages are user-facing and safe; echoed verbatim at 400.
    #[test]
    fn map_error_invalid_input_is_safe() {
        let (status, body) = map_error(SourceError::InvalidInput(
            "count 5 is out of range 1..=3".to_string(),
        ));
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let obj = body.0.as_object().unwrap();
        assert_eq!(obj["error"], "count 5 is out of range 1..=3");
        assert!(
            obj["request_id"].as_str().unwrap().starts_with("req_"),
            "must carry a correlation id"
        );
    }

    /// GenerationFailed must never leak internals (URLs, parse errors) — only a
    /// generic message + request_id, at 500.
    #[test]
    fn map_error_generation_failed_hides_internals() {
        let (status, body) = map_error(SourceError::GenerationFailed(
            "drand request failed: error sending request for url (https://api.drand.sh/public/latest): connection refused"
                .to_string(),
        ));
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        let serialized = serde_json::to_string(&body.0).unwrap();
        assert!(
            !serialized.contains("api.drand.sh"),
            "must not leak internal URL: {serialized}"
        );
        assert!(
            !serialized.contains("error sending request"),
            "must not leak internal error text: {serialized}"
        );
        assert!(serialized.contains("internal error"));
        assert!(serialized.contains("request_id"));
    }

    /// InvalidSource is treated as a 500 internal error (its message can name
    /// internal source plumbing) and is not echoed.
    #[test]
    fn map_error_invalid_source_is_500_generic() {
        let (status, body) =
            map_error(SourceError::InvalidSource("some-internal-source".to_string()));
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        let serialized = serde_json::to_string(&body.0).unwrap();
        assert!(!serialized.contains("some-internal-source"));
        assert!(serialized.contains("internal error"));
    }

    /// Unavailable is 503 with a generic message, no leak.
    #[test]
    fn map_error_unavailable_is_503_generic() {
        let (status, body) = map_error(SourceError::Unavailable(
            "drand: dial tcp: lookup api.drand.sh: no such host".to_string(),
        ));
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        let serialized = serde_json::to_string(&body.0).unwrap();
        assert!(!serialized.contains("api.drand.sh"));
        assert!(serialized.contains("source unavailable"));
    }
}
