use crate::{app_state::AppState, rest_service};
use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

pub async fn launch_http_server_task(
    app_state: AppState,
    http_port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_http_app(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], http_port));
    let listener = TcpListener::bind(addr).await?;

    info!("HTTP REST API server listening on {}", addr);

    axum::serve(listener, app).await.map_err(|e| {
        warn!("HTTP server error: {}", e);
        e
    })?;

    Ok(())
}

fn create_http_app(app_state: AppState) -> Router {
    http_service::create_router()
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers(tower_http::cors::Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}

pub async fn launch_http_server_task(
    _app_state: crate::app_state::AppState,
    _http_port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // HTTP feature not enabled
    tokio::time::sleep(std::time::Duration::from_secs(u64::MAX)).await;
    Ok(())
}
