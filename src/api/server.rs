use crate::api::routes::*;
use axum::response::Html;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use std::time::Duration;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
const INDEX_HTML: &str = include_str!("../../web/index.html");

/// Maximum accepted request body size (2 MiB). Defense in depth on top of the
/// per-field input caps already enforced in `services`.
const MAX_REQUEST_BODY_BYTES: usize = 2 * 1024 * 1024;

/// Per-request timeout. Sources backed by network beacons (drand) run on the
/// blocking pool; this bounds total wall-clock for a single request.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Start the HTTP API, binding to `host:port`.
///
/// `host` defaults to the loopback interface (`127.0.0.1`) at the call site so
/// the server is not exposed on every network interface unless the operator
/// opts in (e.g. `--host 0.0.0.0`). The router is hardened with:
/// - a 30s per-request [`TimeoutLayer`],
/// - a 2 MiB [`RequestBodyLimitLayer`] (defense in depth),
/// - a [`TraceLayer`] for structured request spans.
pub async fn serve(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    // Initialise structured logging exactly once. Ignore the error returned when
    // the global subscriber was already installed (e.g. test harness, embedded use).
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("chance=info".parse().unwrap()),
        )
        .try_init();

    let state = Arc::new(AppState);

    let app = Router::new()
        .route("/", get(index))
        .route("/index.html", get(index))
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
        .layer(TimeoutLayer::new(REQUEST_TIMEOUT))
        .layer(RequestBodyLimitLayer::new(MAX_REQUEST_BODY_BYTES))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "chance API listening");
    println!("chance API listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn serve_fails_when_port_already_in_use() {
        // Occupy an ephemeral port for the duration of the call, then attempt to
        // serve on it. This exercises the real serve() code path (subscriber
        // init, router construction with all three tower-http layers, and the
        // bind) and asserts a fast address-in-use failure instead of hanging.
        let guard = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = guard.local_addr().unwrap().port();
        let result = serve("127.0.0.1", port).await;
        assert!(
            result.is_err(),
            "binding an already-held port must surface an error"
        );
        drop(guard);
    }
}
