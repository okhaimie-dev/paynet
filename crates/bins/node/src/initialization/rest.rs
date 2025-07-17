use crate::app_state::AppState;
#[cfg(feature = "rest")]
use {
    crate::rest_service,
    axum::Router,
    std::net::SocketAddr,
    tokio::net::TcpListener,
    tower_http::{cors::CorsLayer, trace::TraceLayer},
    tracing::{info, warn},
};

#[cfg(feature = "rest")]
pub async fn launch_rest_server_task(
    app_state: AppState,
    rest_port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_rest_app(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], rest_port));
    let listener = TcpListener::bind(addr).await?;

    info!("HTTP REST API server listening on {}", addr);

    axum::serve(listener, app).await.map_err(|e| e)?;

    Ok(())
}

#[cfg(feature = "rest")]
fn create_rest_app(app_state: AppState) -> Router {
    rest_service::create_router()
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

#[cfg(not(feature = "rest"))]
pub async fn launch_rest_server_task(
    _app_state: AppState,
    _rest_port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // HTTP feature not enabled
    tokio::time::sleep(std::time::Duration::from_secs(u64::MAX)).await;
    Ok(())
}
