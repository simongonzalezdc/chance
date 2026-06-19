use crate::services::dto::*;
use crate::services;
use crate::core::SourceError;
use axum::response::Json;
use axum::http::StatusCode;

pub struct AppState;

pub type ApiResult<T> = Result<Json<ApiResponse<T>>, (StatusCode, Json<serde_json::Value>)>;

fn map_error(e: SourceError) -> (StatusCode, Json<serde_json::Value>) {
    let status = match &e {
        SourceError::InvalidSource(_) | SourceError::UnsupportedOperation { .. } | SourceError::InvalidInput(_) => {
            StatusCode::BAD_REQUEST
        }
        SourceError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        SourceError::GenerationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    eprintln!("API error: {}", e);
    let body = serde_json::json!({ "error": e.to_string() });
    (status, Json(body))
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
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("join: {e}") })),
        )),
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
