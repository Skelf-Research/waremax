//! Application state shared across handlers

use std::sync::Arc;

use crate::session::SessionManager;

/// Application state shared across handlers
pub struct AppState {
    pub session_manager: Arc<SessionManager>,
}
