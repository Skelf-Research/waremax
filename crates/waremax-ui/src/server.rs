//! Web server setup for the simulation UI

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use axum::{
    routing::{get, post, delete},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::handlers::api::{self, AppState};
use crate::handlers::{websocket, static_files};
use crate::session::{SessionManager, start_cleanup_task};

/// Server configuration
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Port to listen on
    pub port: u16,
    /// Whether to open browser automatically
    pub open_browser: bool,
    /// Session timeout in seconds
    pub session_timeout_secs: u64,
    /// Maximum concurrent sessions
    pub max_sessions: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            open_browser: true,
            session_timeout_secs: 30 * 60, // 30 minutes
            max_sessions: 100,
        }
    }
}

/// Create the application router
fn create_router(state: Arc<AppState>) -> Router {
    // CORS configuration for development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

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
        .route("/session/:id/add-robot", post(api::add_robot))
        .route("/session/:id/state", get(api::get_state));

    // WebSocket route
    let ws_routes = Router::new()
        .route("/:id", get(websocket::websocket_handler));

    // Combine all routes
    Router::new()
        .nest("/api", api_routes)
        .nest("/ws", ws_routes)
        .route("/", get(static_files::serve_index))
        .fallback(static_files::serve_static)
        .layer(cors)
        .with_state(state)
}

/// Run the web server
pub async fn run_server(config: ServerConfig) -> anyhow::Result<()> {
    // Create session manager
    let session_manager = Arc::new(SessionManager::new(
        Duration::from_secs(config.session_timeout_secs),
        config.max_sessions,
    ));

    // Start session cleanup task
    let _cleanup_task = start_cleanup_task(session_manager.clone());

    // Create app state
    let state = Arc::new(AppState { session_manager });

    // Create router
    let app = create_router(state);

    // Bind to address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let url = format!("http://localhost:{}", config.port);
    println!("Waremax Simulation UI starting...");
    println!("  Server: {}", url);
    println!("  Press Ctrl+C to stop\n");

    // Open browser if requested
    if config.open_browser {
        if let Err(e) = open_browser(&url) {
            println!("  Could not open browser: {}", e);
            println!("  Please open {} manually\n", url);
        }
    }

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}

/// Open the default browser to a URL
fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
