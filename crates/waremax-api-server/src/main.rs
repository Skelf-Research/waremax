//! Standalone server binary for the Waremax simulation API.
//!
//! This is a thin wrapper around `waremax_api::create_router` that binds
//! to a configurable address and serves the API.

use std::net::SocketAddr;
use waremax_api::ApiConfig;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = load_config_from_env();

    let app = waremax_api::create_router(config.clone());

    let port = std::env::var("WAREMAX_API_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let host = std::env::var("WAREMAX_API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid bind address");

    tracing::info!("Waremax API server starting on {}", addr);
    if config.cors_origins.is_empty() {
        tracing::warn!("CORS is configured to allow all origins (development mode)");
    } else {
        tracing::info!("CORS origins: {:?}", config.cors_origins);
    }

    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}

fn load_config_from_env() -> ApiConfig {
    let session_timeout_secs = std::env::var("WAREMAX_SESSION_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30 * 60);

    let max_sessions = std::env::var("WAREMAX_MAX_SESSIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    let cors_origins: Vec<String> = std::env::var("WAREMAX_CORS_ORIGINS")
        .ok()
        .map(|s| s.split(',').map(|o| o.trim().to_string()).collect())
        .unwrap_or_default();

    ApiConfig {
        session_timeout_secs,
        max_sessions,
        cors_origins,
        request_id_header: true,
    }
}
