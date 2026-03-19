//! World state snapshots for replay
//!
//! v3: Capture simulation state at a point in time for replay and debugging
//!
//! Uses primitive types for serialization to avoid requiring serde derives on core types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use waremax_core::{RobotId, NodeId, StationId, TaskId, OrderId, SimTime, ScheduledEvent};

/// Snapshot of a robot's state at a point in time
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RobotSnapshot {
    pub id: u32,
    pub current_node: u32,
    /// Serialized state type (e.g., "Idle", "Moving", "Charging")
    pub state_type: String,
    pub current_task: Option<u32>,
    pub task_queue: Vec<u32>,
    pub current_path: Vec<u32>,
    pub path_index: usize,
    pub battery_soc: f64,
    pub seeking_charging: bool,
    pub assigned_charging_station: Option<u32>,
    pub seeking_maintenance: bool,
    pub assigned_maintenance_station: Option<u32>,
    pub is_failed: bool,
}

/// Snapshot of a station's state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StationSnapshot {
    pub id: u32,
    pub queue: Vec<u32>,
    pub serving: Vec<u32>,
}

/// Snapshot of an order's state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderSnapshot {
    pub id: u32,
    /// Serialized status type
    pub status: String,
    pub tasks_total: u32,
    pub tasks_completed: u32,
}

/// Snapshot of a task's state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskSnapshot {
    pub id: u32,
    /// Serialized status type
    pub status: String,
    pub assigned_robot: Option<u32>,
    pub destination_station: u32,
}

/// Complete world state snapshot
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldSnapshot {
    /// Simulation time when snapshot was taken (seconds)
    pub timestamp_s: f64,
    /// Robot states
    pub robots: HashMap<u32, RobotSnapshot>,
    /// Station states
    pub stations: HashMap<u32, StationSnapshot>,
    /// Order states
    pub orders: HashMap<u32, OrderSnapshot>,
    /// Task states
    pub tasks: HashMap<u32, TaskSnapshot>,
    /// Pending task queue
    pub pending_tasks: Vec<u32>,
    /// Pending events (serialized)
    pub pending_events: Vec<SerializedEvent>,
    /// Node occupancy (node -> robot)
    pub node_occupancy: HashMap<u32, u32>,
}

/// Serialized event for snapshot storage
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializedEvent {
    pub time_s: f64,
    pub event_type: String,
}

impl WorldSnapshot {
    /// Create a snapshot from simulation state
    pub fn capture(
        timestamp: SimTime,
        robots: &HashMap<RobotId, waremax_entities::Robot>,
        stations: &HashMap<StationId, waremax_entities::Station>,
        orders: &HashMap<OrderId, waremax_entities::Order>,
        tasks: &HashMap<TaskId, waremax_entities::Task>,
        pending_tasks: &[TaskId],
        pending_events: &[ScheduledEvent],
        node_occupancy: &HashMap<NodeId, RobotId>,
    ) -> Self {
        // Snapshot robots
        let robots = robots
            .iter()
            .map(|(id, robot)| {
                (
                    id.0,
                    RobotSnapshot {
                        id: robot.id.0,
                        current_node: robot.current_node.0,
                        state_type: format!("{:?}", robot.state),
                        current_task: robot.current_task.map(|t| t.0),
                        task_queue: robot.task_queue.iter().map(|t| t.0).collect(),
                        current_path: robot.current_path.iter().map(|n| n.0).collect(),
                        path_index: robot.path_index,
                        battery_soc: robot.battery.soc,
                        seeking_charging: robot.seeking_charging,
                        assigned_charging_station: robot.assigned_charging_station.map(|c| c.0),
                        seeking_maintenance: robot.seeking_maintenance,
                        assigned_maintenance_station: robot.assigned_maintenance_station.map(|m| m.0),
                        is_failed: robot.maintenance.is_failed,
                    },
                )
            })
            .collect();

        // Snapshot stations
        let stations = stations
            .iter()
            .map(|(id, station)| {
                (
                    id.0,
                    StationSnapshot {
                        id: station.id.0,
                        queue: station.queue.iter().map(|r| r.0).collect(),
                        serving: station.serving.iter().map(|r| r.0).collect(),
                    },
                )
            })
            .collect();

        // Snapshot orders
        let orders = orders
            .iter()
            .map(|(id, order)| {
                (
                    id.0,
                    OrderSnapshot {
                        id: order.id.0,
                        status: format!("{:?}", order.status),
                        tasks_total: order.tasks_total,
                        tasks_completed: order.tasks_completed,
                    },
                )
            })
            .collect();

        // Snapshot tasks
        let tasks = tasks
            .iter()
            .map(|(id, task)| {
                (
                    id.0,
                    TaskSnapshot {
                        id: task.id.0,
                        status: format!("{:?}", task.status),
                        assigned_robot: task.assigned_robot.map(|r| r.0),
                        destination_station: task.destination_station.0,
                    },
                )
            })
            .collect();

        // Serialize pending events (just type and time)
        let pending_events = pending_events
            .iter()
            .map(|e| SerializedEvent {
                time_s: e.time.as_seconds(),
                event_type: e.event.event_type_name().to_string(),
            })
            .collect();

        Self {
            timestamp_s: timestamp.as_seconds(),
            robots,
            stations,
            orders,
            tasks,
            pending_tasks: pending_tasks.iter().map(|t| t.0).collect(),
            pending_events,
            node_occupancy: node_occupancy.iter().map(|(n, r)| (n.0, r.0)).collect(),
        }
    }

    /// Serialize snapshot to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize snapshot from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize snapshot to bytes (compact)
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize snapshot from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Get robot positions as a map of robot_id -> node_id
    pub fn robot_positions(&self) -> HashMap<u32, u32> {
        self.robots.iter().map(|(id, r)| (*id, r.current_node)).collect()
    }

    /// Get count of idle robots
    pub fn idle_robot_count(&self) -> usize {
        self.robots.values().filter(|r| r.state_type.starts_with("Idle")).count()
    }

    /// Get count of moving robots
    pub fn moving_robot_count(&self) -> usize {
        self.robots.values().filter(|r| r.state_type.starts_with("Moving")).count()
    }
}

/// Snapshot manager for taking periodic snapshots during simulation
pub struct SnapshotManager {
    /// Interval between snapshots in seconds
    pub interval_s: f64,
    /// Last snapshot time
    last_snapshot_time: f64,
    /// Stored snapshots (in-memory)
    snapshots: Vec<WorldSnapshot>,
    /// Maximum number of snapshots to keep (0 = unlimited)
    max_snapshots: usize,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new(interval_s: f64) -> Self {
        Self {
            interval_s,
            last_snapshot_time: -interval_s, // Ensure first snapshot is taken immediately
            snapshots: Vec::new(),
            max_snapshots: 0,
        }
    }

    /// Create with a maximum number of snapshots (rolling buffer)
    pub fn with_max_snapshots(interval_s: f64, max_snapshots: usize) -> Self {
        Self {
            interval_s,
            last_snapshot_time: -interval_s,
            snapshots: Vec::new(),
            max_snapshots,
        }
    }

    /// Check if a snapshot should be taken at the current time
    pub fn should_snapshot(&self, current_time_s: f64) -> bool {
        current_time_s - self.last_snapshot_time >= self.interval_s
    }

    /// Store a snapshot
    pub fn store(&mut self, snapshot: WorldSnapshot) {
        self.last_snapshot_time = snapshot.timestamp_s;
        self.snapshots.push(snapshot);

        // Enforce rolling buffer limit
        if self.max_snapshots > 0 && self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }
    }

    /// Get all snapshots
    pub fn snapshots(&self) -> &[WorldSnapshot] {
        &self.snapshots
    }

    /// Get snapshot count
    pub fn count(&self) -> usize {
        self.snapshots.len()
    }

    /// Find snapshot nearest to a time
    pub fn find_nearest(&self, time_s: f64) -> Option<&WorldSnapshot> {
        if self.snapshots.is_empty() {
            return None;
        }

        // Binary search for nearest snapshot
        let mut best_idx = 0;
        let mut best_diff = f64::MAX;

        for (i, snap) in self.snapshots.iter().enumerate() {
            let diff = (snap.timestamp_s - time_s).abs();
            if diff < best_diff {
                best_diff = diff;
                best_idx = i;
            }
        }

        Some(&self.snapshots[best_idx])
    }

    /// Find snapshot at or before a time (for replay starting point)
    pub fn find_at_or_before(&self, time_s: f64) -> Option<&WorldSnapshot> {
        self.snapshots
            .iter()
            .rev()
            .find(|s| s.timestamp_s <= time_s)
    }

    /// Clear all snapshots
    pub fn clear(&mut self) {
        self.snapshots.clear();
        self.last_snapshot_time = -self.interval_s;
    }

    /// Export all snapshots to JSON array
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.snapshots)
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new(60.0) // Default: 1 minute intervals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_manager_interval() {
        let mut manager = SnapshotManager::new(60.0);
        assert!(manager.should_snapshot(0.0)); // First snapshot always

        // Store a snapshot at time 0
        manager.store(WorldSnapshot {
            timestamp_s: 0.0,
            robots: HashMap::new(),
            stations: HashMap::new(),
            orders: HashMap::new(),
            tasks: HashMap::new(),
            pending_tasks: vec![],
            pending_events: vec![],
            node_occupancy: HashMap::new(),
        });

        assert!(!manager.should_snapshot(30.0)); // 30s after last snapshot is too soon
        assert!(manager.should_snapshot(60.0)); // 60s after is exactly the interval
    }

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = WorldSnapshot {
            timestamp_s: 100.0,
            robots: HashMap::new(),
            stations: HashMap::new(),
            orders: HashMap::new(),
            tasks: HashMap::new(),
            pending_tasks: vec![],
            pending_events: vec![],
            node_occupancy: HashMap::new(),
        };

        let json = snapshot.to_json().unwrap();
        let restored = WorldSnapshot::from_json(&json).unwrap();
        assert_eq!(restored.timestamp_s, 100.0);
    }

    #[test]
    fn test_rolling_buffer() {
        let mut manager = SnapshotManager::with_max_snapshots(1.0, 3);

        for i in 0..5 {
            manager.store(WorldSnapshot {
                timestamp_s: i as f64,
                robots: HashMap::new(),
                stations: HashMap::new(),
                orders: HashMap::new(),
                tasks: HashMap::new(),
                pending_tasks: vec![],
                pending_events: vec![],
                node_occupancy: HashMap::new(),
            });
        }

        assert_eq!(manager.count(), 3);
        // Should have snapshots 2, 3, 4
        assert_eq!(manager.snapshots()[0].timestamp_s, 2.0);
    }

    #[test]
    fn test_find_nearest() {
        let mut manager = SnapshotManager::new(60.0);

        for i in 0..5 {
            manager.store(WorldSnapshot {
                timestamp_s: i as f64 * 60.0,
                robots: HashMap::new(),
                stations: HashMap::new(),
                orders: HashMap::new(),
                tasks: HashMap::new(),
                pending_tasks: vec![],
                pending_events: vec![],
                node_occupancy: HashMap::new(),
            });
        }

        let nearest = manager.find_nearest(125.0).unwrap();
        assert_eq!(nearest.timestamp_s, 120.0);
    }
}
