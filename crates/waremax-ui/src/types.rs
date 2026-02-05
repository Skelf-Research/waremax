//! API data types and DTOs for the web UI

use serde::{Deserialize, Serialize};

/// Session configuration for creating new simulations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Preset name: "small", "standard", "large"
    pub preset: String,
    /// Number of robots (overrides preset if set)
    pub robot_count: Option<u32>,
    /// Order arrival rate per hour (overrides preset if set)
    pub order_rate: Option<f64>,
    /// Simulation duration in minutes
    pub duration_minutes: Option<f64>,
    /// Grid size rows (overrides preset if set)
    pub grid_rows: Option<u32>,
    /// Grid size columns (overrides preset if set)
    pub grid_cols: Option<u32>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            preset: "standard".to_string(),
            robot_count: None,
            order_rate: None,
            duration_minutes: None,
            grid_rows: None,
            grid_cols: None,
        }
    }
}

/// Response for session creation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub status: String,
}

/// Simulation status
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SimulationStatus {
    Idle,
    Running,
    Paused,
    Finished,
    Error,
}

/// Simulation state response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulationState {
    pub status: SimulationStatus,
    pub time_s: f64,
    pub speed: f64,
    pub events_processed: u64,
    pub orders_completed: u64,
    pub robots: Vec<RobotState>,
    pub stations: Vec<StationState>,
    pub metrics: MetricsSnapshot,
}

/// Robot state for frontend display
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RobotState {
    pub id: u32,
    pub node_id: u32,
    pub state: String,
    pub battery_soc: Option<f64>,
    pub current_task: Option<u32>,
    pub is_failed: bool,
}

/// Station state for frontend display
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StationState {
    pub id: u32,
    pub name: String,
    pub node_id: u32,
    pub station_type: String,
    pub queue_length: usize,
    pub serving_count: usize,
    pub concurrency: u32,
}

/// Metrics snapshot for dashboard
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MetricsSnapshot {
    pub throughput_per_hour: f64,
    pub orders_completed: u64,
    pub orders_pending: u64,
    pub robot_utilization: f64,
    pub station_utilization: f64,
    pub avg_cycle_time_s: f64,
    pub late_orders: u64,
}

/// Warehouse map data for frontend rendering
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapData {
    pub nodes: Vec<NodeData>,
    pub edges: Vec<EdgeData>,
    pub bounds: MapBounds,
}

/// Node data for map rendering
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeData {
    pub id: u32,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub node_type: String,
}

/// Edge data for map rendering
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeData {
    pub id: u32,
    pub from: u32,
    pub to: u32,
    pub length: f64,
    pub bidirectional: bool,
}

/// Map bounds for canvas sizing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapBounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

/// Control command from frontend
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ControlCommand {
    Start,
    Pause,
    Resume,
    SetSpeed { speed: f64 },
    Step,
    AddRobot { node_id: Option<u32> },
    Reset,
    Stop,
}

/// Speed change request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeedRequest {
    pub speed: f64,
}

/// Add robot request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddRobotRequest {
    pub node_id: Option<u32>,
}

/// Preset information for frontend
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresetInfo {
    pub name: String,
    pub description: String,
    pub robots: u32,
    pub stations: u32,
    pub order_rate: f64,
    pub duration_minutes: f64,
    pub grid_size: String,
}

/// WebSocket event wrapper for frontend
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// Connection established
    Connected { session_id: String },
    /// Simulation tick update
    Tick {
        time_s: f64,
        events_processed: u64,
    },
    /// Robot position changed
    RobotMoved {
        robot_id: u32,
        from_node: u32,
        to_node: u32,
        time_s: f64,
    },
    /// Robot state changed
    RobotStateChanged {
        robot_id: u32,
        old_state: String,
        new_state: String,
        time_s: f64,
    },
    /// Order completed
    OrderCompleted {
        order_id: u32,
        cycle_time_s: f64,
        on_time: bool,
    },
    /// Metrics update
    MetricsUpdate {
        metrics: MetricsSnapshot,
    },
    /// Full state sync
    StateSync {
        state: SimulationState,
    },
    /// Simulation finished
    Finished {
        final_metrics: MetricsSnapshot,
    },
    /// Error occurred
    Error {
        message: String,
    },
}

/// Error response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: None,
        }
    }

    pub fn with_details(error: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: Some(details.into()),
        }
    }
}
