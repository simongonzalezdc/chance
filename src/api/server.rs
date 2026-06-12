use crate::api::routes::*;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

pub async fn serve(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState);

    let app = Router::new()
        .route("/v1/roll", post(roll))
        .route("/v1/flip", post(flip))
        .route("/v1/draw", post(draw))
        .route("/v1/pick", post(pick))
        .route("/v1/shuffle", post(shuffle))
        .route("/v1/int", post(int))
        .route("/v1/bytes", post(bytes))
        .route("/v1/uuid", post(uuid))
        .route("/v1/password", post(password))
        .route("/v1/runes", post(runes))
        .route("/v1/iching", post(iching))
        .route("/v1/tarot", post(tarot))
        .route("/v1/dominoes", post(dominoes))
        .route("/v1/roulette", post(roulette))
        .route("/v1/lottery", post(lottery))
        .route("/v1/knucklebones", post(knucklebones))
        .route("/v1/teetotum", post(teetotum))
        .route("/v1/cowrie", post(cowrie))
        .route("/v1/lots", post(lots))
        .route("/v1/sources", get(sources))
        .route("/v1/health", get(health))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("chance API listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
