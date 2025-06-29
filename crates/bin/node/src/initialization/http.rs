#[cfg(feature = "http")]
use axum::Router;
#[cfg(feature = "http")]
use std::net::SocketAddr;
#[cfg(feature = "http")]
use tokio::net::TcpListener;
#[cfg(feature = "http")]
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
#[cfg(feature = "http")]
use tracing::{info, warn};

#[cfg(feature = "http")]
use crate::{
    app_state::AppState,
    http_service,
};

/// Launch the HTTP server task
#[cfg(feature = "http")]
pub async fn launch_http_server_task(
    app_state: AppState,
    http_port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_http_app(app_state);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], http_port));
    let listener = TcpListener::bind(addr).await?;
    
    info!("HTTP REST API server listening on {}", addr);
    
    axum::serve(listener, app)
        .await
        .map_err(|e| {
            warn!("HTTP server error: {}", e);
            e
        })?;
    
    Ok(())
}

/// Create the HTTP application with routes and middleware
#[cfg(feature = "http")]
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

#[cfg(not(feature = "http"))]
pub async fn launch_http_server_task(
    _app_state: crate::app_state::AppState,
    _http_port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // HTTP feature not enabled
    tokio::time::sleep(std::time::Duration::from_secs(u64::MAX)).await;
    Ok(())
}