use crate::services::dto::*;
use crate::services;
use crate::core::SourceError;
use axum::response::Json;
use axum::http::StatusCode;

pub struct AppState;

pub type ApiResult<T> = Result<Json<ApiResponse<T>>, (StatusCode, Json<serde_json::Value>)>;

fn map_error(e: SourceError) -> (StatusCode, Json<serde_json::Value>) {
    let status = match &e {
        SourceError::InvalidSource(_) | SourceError::UnsupportedOperation { .. } => {
            StatusCode::BAD_REQUEST
        }
        SourceError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        SourceError::GenerationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    eprintln!("API error: {}", e);
    let body = serde_json::json!({ "error": e.to_string() });
    (status, Json(body))
}

pub async fn roll(Json(req): Json<RollRequest>) -> ApiResult<RollResultDto> {
    services::roll(&req).map(Json).map_err(map_error)
}

pub async fn flip(Json(req): Json<FlipRequest>) -> ApiResult<Vec<String>> {
    services::flip(&req).map(Json).map_err(map_error)
}

pub async fn draw(Json(req): Json<DrawRequest>) -> ApiResult<Vec<String>> {
    services::draw(&req).map(Json).map_err(map_error)
}

pub async fn pick(Json(req): Json<ListRequest>) -> ApiResult<Vec<String>> {
    services::pick(&req).map(Json).map_err(map_error)
}

pub async fn shuffle(Json(req): Json<ShuffleRequest>) -> ApiResult<Vec<String>> {
    services::shuffle(&req).map(Json).map_err(map_error)
}

pub async fn int(Json(req): Json<IntRequest>) -> ApiResult<i64> {
    services::integer(&req).map(Json).map_err(map_error)
}

pub async fn bytes(Json(req): Json<BytesRequest>) -> ApiResult<String> {
    services::bytes(&req).map(Json).map_err(map_error)
}

pub async fn uuid(Json(req): Json<UuidRequest>) -> ApiResult<String> {
    services::uuid(&req).map(Json).map_err(map_error)
}

pub async fn password(Json(req): Json<PasswordRequest>) -> ApiResult<String> {
    services::password(&req).map(Json).map_err(map_error)
}

pub async fn runes(Json(req): Json<RunesRequest>) -> ApiResult<Vec<String>> {
    services::runes(&req).map(Json).map_err(map_error)
}

pub async fn iching(Json(req): Json<IchingRequest>) -> ApiResult<IchingResultDto> {
    services::iching(&req).map(Json).map_err(map_error)
}

pub async fn tarot(Json(req): Json<TarotRequest>) -> ApiResult<Vec<TarotCardDto>> {
    services::tarot(&req).map(Json).map_err(map_error)
}

pub async fn dominoes(Json(req): Json<DominoesRequest>) -> ApiResult<Vec<DominoDto>> {
    services::dominoes(&req).map(Json).map_err(map_error)
}

pub async fn roulette(Json(req): Json<RouletteRequest>) -> ApiResult<RouletteResultDto> {
    services::roulette(&req).map(Json).map_err(map_error)
}

pub async fn lottery(Json(req): Json<LotteryRequest>) -> ApiResult<LotteryResultDto> {
    services::lottery(&req).map(Json).map_err(map_error)
}

pub async fn knucklebones(Json(req): Json<KnucklebonesRequest>) -> ApiResult<Vec<u8>> {
    services::knucklebones(&req).map(Json).map_err(map_error)
}

pub async fn teetotum(Json(req): Json<TeetotumRequest>) -> ApiResult<String> {
    services::teetotum(&req).map(Json).map_err(map_error)
}

pub async fn cowrie(Json(req): Json<CowrieRequest>) -> ApiResult<CowrieResultDto> {
    services::cowrie(&req).map(Json).map_err(map_error)
}

pub async fn lots(Json(req): Json<ListRequest>) -> ApiResult<Vec<String>> {
    services::lots(&req).map(Json).map_err(map_error)
}

pub async fn sources() -> Json<Vec<String>> {
    Json(services::source_names())
}

pub async fn health() -> Json<serde_json::Value> {
    Json(services::health())
}
