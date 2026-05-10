//! Axum router builder for the Waremax simulation API

use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};

use crate::handlers::{api, websocket};
use crate::session::{start_cleanup_task, SessionManager};
use crate::state::AppState;

/// API configuration for router creation
#[derive(Clone, Debug)]
pub struct ApiConfig {
    /// Session timeout in seconds
    pub session_timeout_secs: u64,
    /// Maximum concurrent sessions
    pub max_sessions: usize,
    /// CORS origins (empty = allow all for development)
    pub cors_origins: Vec<String>,
    /// Whether to enable request ID headers
    pub request_id_header: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            session_timeout_secs: 30 * 60, // 30 minutes
            max_sessions: 100,
            cors_origins: vec![],
            request_id_header: true,
        }
    }
}

/// Create the application router
pub fn create_router(config: ApiConfig) -> Router {
    // Create session manager
    let session_manager = Arc::new(SessionManager::new(
        Duration::from_secs(config.session_timeout_secs),
        config.max_sessions,
    ));

    // Start session cleanup task
    let _cleanup_task = start_cleanup_task(session_manager.clone());

    // Create app state
    let state = Arc::new(AppState { session_manager });

    // CORS configuration
    let cors = if config.cors_origins.is_empty() {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins: Vec<_> = config
            .cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::DELETE,
            ])
            .allow_headers([axum::http::header::CONTENT_TYPE])
    };

    // API routes
    let api_routes = Router::new()
        .route("/health", get(api::health_check))
        .route("/presets", get(api::get_presets))
        .route("/session", post(api::create_session))
        .route("/session/:id", delete(api::delete_session))
        .route("/session/:id/map", get(api::get_session_map))
        .route("/session/:id/start", post(api::start_session))
        .route("/session/:id/pause", post(api::pause_session))
        .route("/session/:id/resume", post(api::resume_session))
        .route("/session/:id/speed", post(api::set_speed))
        .route("/session/:id/step", post(api::step_session))
        .route("/session/:id/add-robot", post(api::add_robot));

    // WebSocket route
    let ws_routes = Router::new().route("/:id", get(websocket::websocket_handler));

    // Combine all routes
    Router::new()
        .nest("/api", api_routes)
        .nest("/ws", ws_routes)
        .layer(cors)
        .with_state(state)
}
