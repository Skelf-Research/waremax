//! Session management for web UI
//!
//! Each browser tab gets its own independent simulation session.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, Mutex, RwLock};
use uuid::Uuid;

use crate::simulation::{SimulationHandle, SimulationConfig, SimCommand, SimUpdate, spawn_simulation};
use crate::types::{SessionConfig, MapData};

/// A simulation session for a single user/browser tab
pub struct Session {
    pub id: String,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub config: SessionConfig,
    pub handle: SimulationHandle,
    pub task: tokio::task::JoinHandle<()>,
}

impl Session {
    /// Create a new session
    pub fn new(config: SessionConfig) -> Self {
        let id = Uuid::new_v4().to_string();

        let sim_config = SimulationConfig {
            preset: config.preset.clone(),
            robot_count: config.robot_count,
            order_rate: config.order_rate,
            duration_minutes: config.duration_minutes.unwrap_or(60.0),
            grid_rows: config.grid_rows,
            grid_cols: config.grid_cols,
        };

        let (handle, task) = spawn_simulation(sim_config);

        Self {
            id,
            created_at: Instant::now(),
            last_activity: Instant::now(),
            config,
            handle,
            task,
        }
    }

    /// Touch the session (update last activity time)
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Check if the session has expired
    pub fn is_expired(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }

    /// Get the map data for this session
    pub fn map_data(&self) -> &MapData {
        self.handle.map_data()
    }

    /// Subscribe to simulation updates
    pub fn subscribe(&self) -> broadcast::Receiver<SimUpdate> {
        self.handle.subscribe()
    }

    /// Send a control command
    pub async fn send_command(&self, cmd: SimCommand) -> Result<(), String> {
        self.handle.send_command(cmd).await.map_err(|e| e.to_string())
    }

    /// Start the simulation
    pub async fn start(&self) -> Result<(), String> {
        self.handle.resume().await.map_err(|e| e.to_string())
    }

    /// Pause the simulation
    pub async fn pause(&self) -> Result<(), String> {
        self.handle.pause().await.map_err(|e| e.to_string())
    }

    /// Resume the simulation
    pub async fn resume(&self) -> Result<(), String> {
        self.handle.resume().await.map_err(|e| e.to_string())
    }

    /// Set simulation speed
    pub async fn set_speed(&self, speed: f64) -> Result<(), String> {
        self.handle.set_speed(speed).await.map_err(|e| e.to_string())
    }

    /// Step one event
    pub async fn step(&self) -> Result<(), String> {
        self.handle.step().await.map_err(|e| e.to_string())
    }

    /// Add a robot
    pub async fn add_robot(&self, node_id: Option<u32>) -> Result<(), String> {
        self.handle.add_robot(node_id).await.map_err(|e| e.to_string())
    }

    /// Stop the simulation
    pub async fn stop(&self) -> Result<(), String> {
        self.handle.stop().await.map_err(|e| e.to_string())
    }

    /// Request state update
    pub async fn get_state(&self) -> Result<(), String> {
        self.handle.get_state().await.map_err(|e| e.to_string())
    }
}

/// Session manager that handles multiple concurrent sessions
pub struct SessionManager {
    sessions: RwLock<HashMap<String, Arc<Mutex<Session>>>>,
    session_timeout: Duration,
    max_sessions: usize,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(session_timeout: Duration, max_sessions: usize) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            session_timeout,
            max_sessions,
        }
    }

    /// Create a new session
    pub async fn create_session(&self, config: SessionConfig) -> Result<String, String> {
        // Check session limit
        let sessions = self.sessions.read().await;
        if sessions.len() >= self.max_sessions {
            return Err("Maximum number of sessions reached".to_string());
        }
        drop(sessions);

        let session = Session::new(config);
        let id = session.id.clone();

        let mut sessions = self.sessions.write().await;
        sessions.insert(id.clone(), Arc::new(Mutex::new(session)));

        Ok(id)
    }

    /// Get a session by ID
    pub async fn get_session(&self, id: &str) -> Option<Arc<Mutex<Session>>> {
        let sessions = self.sessions.read().await;
        sessions.get(id).cloned()
    }

    /// Remove a session
    pub async fn remove_session(&self, id: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.remove(id) {
            // Stop the simulation task
            let session = session.lock().await;
            let _ = session.stop().await;
            session.task.abort();
            true
        } else {
            false
        }
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self) {
        let mut sessions = self.sessions.write().await;
        let expired: Vec<String> = sessions
            .iter()
            .filter_map(|(id, session)| {
                // Try to lock without blocking
                if let Ok(s) = session.try_lock() {
                    if s.is_expired(self.session_timeout) {
                        return Some(id.clone());
                    }
                }
                None
            })
            .collect();

        for id in expired {
            if let Some(session) = sessions.remove(&id) {
                if let Ok(s) = session.try_lock() {
                    let _ = s.handle.stop().await;
                    s.task.abort();
                }
            }
        }
    }

    /// Get the number of active sessions
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// List all session IDs
    pub async fn list_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.read().await;
        sessions.keys().cloned().collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(
            Duration::from_secs(30 * 60), // 30 minute timeout
            100,                           // Max 100 concurrent sessions
        )
    }
}

/// Start a background task to periodically clean up expired sessions
pub fn start_cleanup_task(manager: Arc<SessionManager>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            manager.cleanup_expired().await;
        }
    })
}
