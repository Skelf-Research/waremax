//! REST API handlers for simulation control

use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::session::SessionManager;
use crate::types::{
    SessionConfig, SessionResponse, SpeedRequest, AddRobotRequest,
    PresetInfo, ErrorResponse,
};

/// Application state shared across handlers
pub struct AppState {
    pub session_manager: Arc<SessionManager>,
}

/// Create a new simulation session
pub async fn create_session(
    State(state): State<Arc<AppState>>,
    Json(config): Json<SessionConfig>,
) -> impl IntoResponse {
    match state.session_manager.create_session(config).await {
        Ok(session_id) => (
            StatusCode::CREATED,
            Json(SessionResponse {
                session_id,
                status: "created".to_string(),
            }),
        ).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(e)),
        ).into_response(),
    }
}

/// Get the warehouse map for a session
pub async fn get_session_map(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.session_manager.get_session(&session_id).await {
        Some(session) => {
            let session = session.lock().await;
            let map_data = session.map_data().clone();
            (StatusCode::OK, Json(map_data)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Session not found")),
        ).into_response(),
    }
}

/// Start a simulation session
pub async fn start_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.session_manager.get_session(&session_id).await {
        Some(session) => {
            let mut session = session.lock().await;
            session.touch();
            match session.start().await {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "started"}))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e))).into_response(),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Session not found")),
        ).into_response(),
    }
}

/// Pause a simulation session
pub async fn pause_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.session_manager.get_session(&session_id).await {
        Some(session) => {
            let mut session = session.lock().await;
            session.touch();
            match session.pause().await {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "paused"}))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e))).into_response(),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Session not found")),
        ).into_response(),
    }
}

/// Resume a simulation session
pub async fn resume_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.session_manager.get_session(&session_id).await {
        Some(session) => {
            let mut session = session.lock().await;
            session.touch();
            match session.resume().await {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "running"}))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e))).into_response(),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Session not found")),
        ).into_response(),
    }
}

/// Set simulation speed
pub async fn set_speed(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(req): Json<SpeedRequest>,
) -> impl IntoResponse {
    match state.session_manager.get_session(&session_id).await {
        Some(session) => {
            let mut session = session.lock().await;
            session.touch();
            match session.set_speed(req.speed).await {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({"speed": req.speed}))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e))).into_response(),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Session not found")),
        ).into_response(),
    }
}

/// Step one event in the simulation
pub async fn step_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.session_manager.get_session(&session_id).await {
        Some(session) => {
            let mut session = session.lock().await;
            session.touch();
            match session.step().await {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "stepped"}))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e))).into_response(),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Session not found")),
        ).into_response(),
    }
}

/// Add a robot to the simulation
pub async fn add_robot(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(req): Json<AddRobotRequest>,
) -> impl IntoResponse {
    match state.session_manager.get_session(&session_id).await {
        Some(session) => {
            let mut session = session.lock().await;
            session.touch();
            match session.add_robot(req.node_id).await {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "robot_added"}))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e))).into_response(),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Session not found")),
        ).into_response(),
    }
}

/// Get current simulation state
pub async fn get_state(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.session_manager.get_session(&session_id).await {
        Some(session) => {
            let mut session = session.lock().await;
            session.touch();
            match session.get_state().await {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "state_requested"}))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e))).into_response(),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Session not found")),
        ).into_response(),
    }
}

/// Delete a session
pub async fn delete_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    if state.session_manager.remove_session(&session_id).await {
        (StatusCode::OK, Json(serde_json::json!({"status": "deleted"}))).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Session not found")),
        ).into_response()
    }
}

/// Get available presets
pub async fn get_presets() -> impl IntoResponse {
    let presets = vec![
        PresetInfo {
            name: "small".to_string(),
            description: "Small warehouse: 5 robots, 2 stations".to_string(),
            robots: 5,
            stations: 2,
            order_rate: 30.0,
            duration_minutes: 30.0,
            grid_size: "5x5".to_string(),
        },
        PresetInfo {
            name: "standard".to_string(),
            description: "Standard warehouse: 15 robots, 4 stations".to_string(),
            robots: 15,
            stations: 4,
            order_rate: 60.0,
            duration_minutes: 60.0,
            grid_size: "10x10".to_string(),
        },
        PresetInfo {
            name: "large".to_string(),
            description: "Large warehouse: 30 robots, 8 stations".to_string(),
            robots: 30,
            stations: 8,
            order_rate: 120.0,
            duration_minutes: 60.0,
            grid_size: "15x15".to_string(),
        },
    ];

    Json(presets)
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
