//! Dashboard integration for real-time simulation visualization
//!
//! v3: Provides hooks and data structures for dashboard integration.
//! This module defines the data formats and event streaming interface
//! that can be used by external WebSocket servers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
#[cfg(test)]
use std::sync::mpsc::TryRecvError;
use waremax_core::{RobotId, NodeId, StationId, TaskId, OrderId, SimTime};

/// Dashboard event types that can be streamed to clients
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DashboardEvent {
    /// Simulation tick with current time
    Tick {
        time_s: f64,
        events_processed: u64,
    },

    /// Robot position update
    RobotMoved {
        robot_id: u32,
        from_node: u32,
        to_node: u32,
        time_s: f64,
    },

    /// Robot state change
    RobotStateChanged {
        robot_id: u32,
        old_state: String,
        new_state: String,
        time_s: f64,
    },

    /// Task assigned to robot
    TaskAssigned {
        task_id: u32,
        robot_id: u32,
        station_id: u32,
        time_s: f64,
    },

    /// Task completed
    TaskCompleted {
        task_id: u32,
        robot_id: u32,
        time_s: f64,
    },

    /// Order arrived
    OrderArrived {
        order_id: u32,
        lines: u32,
        time_s: f64,
    },

    /// Order completed
    OrderCompleted {
        order_id: u32,
        cycle_time_s: f64,
        on_time: bool,
        time_s: f64,
    },

    /// Station queue update
    StationQueueUpdate {
        station_id: u32,
        queue_length: u32,
        serving: u32,
        time_s: f64,
    },

    /// Battery level update
    BatteryUpdate {
        robot_id: u32,
        soc: f64,
        time_s: f64,
    },

    /// Robot started charging
    ChargingStarted {
        robot_id: u32,
        station_id: u32,
        soc: f64,
        time_s: f64,
    },

    /// Robot finished charging
    ChargingCompleted {
        robot_id: u32,
        station_id: u32,
        energy_wh: f64,
        time_s: f64,
    },

    /// Congestion detected at a node
    CongestionDetected {
        node_id: u32,
        robot_count: u32,
        time_s: f64,
    },

    /// Deadlock detected
    DeadlockDetected {
        robots: Vec<u32>,
        time_s: f64,
    },

    /// Deadlock resolved
    DeadlockResolved {
        robots: Vec<u32>,
        resolver: u32,
        time_s: f64,
    },

    /// Robot failure
    RobotFailed {
        robot_id: u32,
        interrupted_task: Option<u32>,
        time_s: f64,
    },

    /// Periodic metrics snapshot
    MetricsSnapshot {
        time_s: f64,
        throughput_per_hour: f64,
        robot_utilization: f64,
        station_utilization: f64,
        pending_orders: u32,
        pending_tasks: u32,
    },

    /// Simulation started
    SimulationStarted {
        duration_s: f64,
        robot_count: u32,
        station_count: u32,
    },

    /// Simulation ended
    SimulationEnded {
        total_time_s: f64,
        orders_completed: u32,
        throughput_per_hour: f64,
    },
}

impl DashboardEvent {
    /// Get the event time
    pub fn time_s(&self) -> f64 {
        match self {
            DashboardEvent::Tick { time_s, .. } => *time_s,
            DashboardEvent::RobotMoved { time_s, .. } => *time_s,
            DashboardEvent::RobotStateChanged { time_s, .. } => *time_s,
            DashboardEvent::TaskAssigned { time_s, .. } => *time_s,
            DashboardEvent::TaskCompleted { time_s, .. } => *time_s,
            DashboardEvent::OrderArrived { time_s, .. } => *time_s,
            DashboardEvent::OrderCompleted { time_s, .. } => *time_s,
            DashboardEvent::StationQueueUpdate { time_s, .. } => *time_s,
            DashboardEvent::BatteryUpdate { time_s, .. } => *time_s,
            DashboardEvent::ChargingStarted { time_s, .. } => *time_s,
            DashboardEvent::ChargingCompleted { time_s, .. } => *time_s,
            DashboardEvent::CongestionDetected { time_s, .. } => *time_s,
            DashboardEvent::DeadlockDetected { time_s, .. } => *time_s,
            DashboardEvent::DeadlockResolved { time_s, .. } => *time_s,
            DashboardEvent::RobotFailed { time_s, .. } => *time_s,
            DashboardEvent::MetricsSnapshot { time_s, .. } => *time_s,
            DashboardEvent::SimulationStarted { .. } => 0.0,
            DashboardEvent::SimulationEnded { total_time_s, .. } => *total_time_s,
        }
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Dashboard hook for capturing simulation events
pub struct DashboardHook {
    /// Event sender (to be consumed by WebSocket server or other consumers)
    sender: Sender<DashboardEvent>,
    /// Whether the hook is enabled
    enabled: bool,
    /// Tick interval (only send tick events at this interval)
    tick_interval_s: f64,
    /// Last tick time
    last_tick_s: f64,
    /// Filter for which event types to emit
    event_filter: DashboardEventFilter,
}

/// Filter for dashboard events
#[derive(Clone, Debug)]
pub struct DashboardEventFilter {
    pub ticks: bool,
    pub robot_moves: bool,
    pub robot_state_changes: bool,
    pub task_events: bool,
    pub order_events: bool,
    pub station_events: bool,
    pub battery_events: bool,
    pub congestion_events: bool,
    pub metrics_snapshots: bool,
}

impl Default for DashboardEventFilter {
    fn default() -> Self {
        Self {
            ticks: true,
            robot_moves: true,
            robot_state_changes: true,
            task_events: true,
            order_events: true,
            station_events: true,
            battery_events: true,
            congestion_events: true,
            metrics_snapshots: true,
        }
    }
}

impl DashboardEventFilter {
    /// Create filter that only includes essential events
    pub fn essential() -> Self {
        Self {
            ticks: true,
            robot_moves: false,
            robot_state_changes: false,
            task_events: true,
            order_events: true,
            station_events: false,
            battery_events: false,
            congestion_events: true,
            metrics_snapshots: true,
        }
    }

    /// Create filter that includes all events
    pub fn all() -> Self {
        Self::default()
    }
}

impl DashboardHook {
    /// Create a new dashboard hook and return the receiver for events
    pub fn new() -> (Self, Receiver<DashboardEvent>) {
        let (sender, receiver) = mpsc::channel();
        (
            Self {
                sender,
                enabled: true,
                tick_interval_s: 1.0,
                last_tick_s: 0.0,
                event_filter: DashboardEventFilter::default(),
            },
            receiver,
        )
    }

    /// Create with custom tick interval
    pub fn with_tick_interval(tick_interval_s: f64) -> (Self, Receiver<DashboardEvent>) {
        let (sender, receiver) = mpsc::channel();
        (
            Self {
                sender,
                enabled: true,
                tick_interval_s,
                last_tick_s: 0.0,
                event_filter: DashboardEventFilter::default(),
            },
            receiver,
        )
    }

    /// Set event filter
    pub fn set_filter(&mut self, filter: DashboardEventFilter) {
        self.event_filter = filter;
    }

    /// Enable or disable the hook
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if hook is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Emit an event (internal use)
    fn emit(&self, event: DashboardEvent) {
        if self.enabled {
            // Ignore send errors (receiver may have been dropped)
            let _ = self.sender.send(event);
        }
    }

    // === Event emission methods ===

    /// Emit a tick event (if interval has passed)
    pub fn tick(&mut self, time_s: f64, events_processed: u64) {
        if !self.event_filter.ticks {
            return;
        }
        if time_s - self.last_tick_s >= self.tick_interval_s {
            self.last_tick_s = time_s;
            self.emit(DashboardEvent::Tick { time_s, events_processed });
        }
    }

    /// Emit robot moved event
    pub fn robot_moved(&self, robot_id: RobotId, from_node: NodeId, to_node: NodeId, time: SimTime) {
        if !self.event_filter.robot_moves {
            return;
        }
        self.emit(DashboardEvent::RobotMoved {
            robot_id: robot_id.0,
            from_node: from_node.0,
            to_node: to_node.0,
            time_s: time.as_seconds(),
        });
    }

    /// Emit robot state changed event
    pub fn robot_state_changed(&self, robot_id: RobotId, old_state: &str, new_state: &str, time: SimTime) {
        if !self.event_filter.robot_state_changes {
            return;
        }
        self.emit(DashboardEvent::RobotStateChanged {
            robot_id: robot_id.0,
            old_state: old_state.to_string(),
            new_state: new_state.to_string(),
            time_s: time.as_seconds(),
        });
    }

    /// Emit task assigned event
    pub fn task_assigned(&self, task_id: TaskId, robot_id: RobotId, station_id: StationId, time: SimTime) {
        if !self.event_filter.task_events {
            return;
        }
        self.emit(DashboardEvent::TaskAssigned {
            task_id: task_id.0,
            robot_id: robot_id.0,
            station_id: station_id.0,
            time_s: time.as_seconds(),
        });
    }

    /// Emit task completed event
    pub fn task_completed(&self, task_id: TaskId, robot_id: RobotId, time: SimTime) {
        if !self.event_filter.task_events {
            return;
        }
        self.emit(DashboardEvent::TaskCompleted {
            task_id: task_id.0,
            robot_id: robot_id.0,
            time_s: time.as_seconds(),
        });
    }

    /// Emit order arrived event
    pub fn order_arrived(&self, order_id: OrderId, lines: u32, time: SimTime) {
        if !self.event_filter.order_events {
            return;
        }
        self.emit(DashboardEvent::OrderArrived {
            order_id: order_id.0,
            lines,
            time_s: time.as_seconds(),
        });
    }

    /// Emit order completed event
    pub fn order_completed(&self, order_id: OrderId, cycle_time_s: f64, on_time: bool, time: SimTime) {
        if !self.event_filter.order_events {
            return;
        }
        self.emit(DashboardEvent::OrderCompleted {
            order_id: order_id.0,
            cycle_time_s,
            on_time,
            time_s: time.as_seconds(),
        });
    }

    /// Emit station queue update event
    pub fn station_queue_update(&self, station_id: StationId, queue_length: u32, serving: u32, time: SimTime) {
        if !self.event_filter.station_events {
            return;
        }
        self.emit(DashboardEvent::StationQueueUpdate {
            station_id: station_id.0,
            queue_length,
            serving,
            time_s: time.as_seconds(),
        });
    }

    /// Emit battery update event
    pub fn battery_update(&self, robot_id: RobotId, soc: f64, time: SimTime) {
        if !self.event_filter.battery_events {
            return;
        }
        self.emit(DashboardEvent::BatteryUpdate {
            robot_id: robot_id.0,
            soc,
            time_s: time.as_seconds(),
        });
    }

    /// Emit metrics snapshot event
    pub fn metrics_snapshot(
        &self,
        time: SimTime,
        throughput_per_hour: f64,
        robot_utilization: f64,
        station_utilization: f64,
        pending_orders: u32,
        pending_tasks: u32,
    ) {
        if !self.event_filter.metrics_snapshots {
            return;
        }
        self.emit(DashboardEvent::MetricsSnapshot {
            time_s: time.as_seconds(),
            throughput_per_hour,
            robot_utilization,
            station_utilization,
            pending_orders,
            pending_tasks,
        });
    }

    /// Emit simulation started event
    pub fn simulation_started(&self, duration_s: f64, robot_count: u32, station_count: u32) {
        self.emit(DashboardEvent::SimulationStarted {
            duration_s,
            robot_count,
            station_count,
        });
    }

    /// Emit simulation ended event
    pub fn simulation_ended(&self, total_time_s: f64, orders_completed: u32, throughput_per_hour: f64) {
        self.emit(DashboardEvent::SimulationEnded {
            total_time_s,
            orders_completed,
            throughput_per_hour,
        });
    }

    /// Emit deadlock detected event
    pub fn deadlock_detected(&self, robots: &[RobotId], time: SimTime) {
        if !self.event_filter.congestion_events {
            return;
        }
        self.emit(DashboardEvent::DeadlockDetected {
            robots: robots.iter().map(|r| r.0).collect(),
            time_s: time.as_seconds(),
        });
    }

    /// Emit deadlock resolved event
    pub fn deadlock_resolved(&self, robots: &[RobotId], resolver: RobotId, time: SimTime) {
        if !self.event_filter.congestion_events {
            return;
        }
        self.emit(DashboardEvent::DeadlockResolved {
            robots: robots.iter().map(|r| r.0).collect(),
            resolver: resolver.0,
            time_s: time.as_seconds(),
        });
    }

    /// Emit robot failed event
    pub fn robot_failed(&self, robot_id: RobotId, interrupted_task: Option<TaskId>, time: SimTime) {
        self.emit(DashboardEvent::RobotFailed {
            robot_id: robot_id.0,
            interrupted_task: interrupted_task.map(|t| t.0),
            time_s: time.as_seconds(),
        });
    }
}

impl Default for DashboardHook {
    fn default() -> Self {
        Self::new().0
    }
}

/// Dashboard state that can be queried
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardState {
    /// Current simulation time
    pub time_s: f64,
    /// Robot positions (robot_id -> node_id)
    pub robot_positions: HashMap<u32, u32>,
    /// Robot states (robot_id -> state_name)
    pub robot_states: HashMap<u32, String>,
    /// Station queues (station_id -> queue_length)
    pub station_queues: HashMap<u32, u32>,
    /// Pending orders count
    pub pending_orders: u32,
    /// Pending tasks count
    pub pending_tasks: u32,
    /// Orders completed
    pub orders_completed: u32,
    /// Throughput (orders/hour)
    pub throughput_per_hour: f64,
}

impl DashboardState {
    /// Create empty state
    pub fn new() -> Self {
        Self {
            time_s: 0.0,
            robot_positions: HashMap::new(),
            robot_states: HashMap::new(),
            station_queues: HashMap::new(),
            pending_orders: 0,
            pending_tasks: 0,
            orders_completed: 0,
            throughput_per_hour: 0.0,
        }
    }

    /// Convert to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl Default for DashboardState {
    fn default() -> Self {
        Self::new()
    }
}

/// Event buffer for batching dashboard events
pub struct DashboardEventBuffer {
    events: Vec<DashboardEvent>,
    max_size: usize,
}

impl DashboardEventBuffer {
    /// Create a new event buffer
    pub fn new(max_size: usize) -> Self {
        Self {
            events: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Add an event to the buffer
    pub fn push(&mut self, event: DashboardEvent) {
        if self.events.len() >= self.max_size {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    /// Drain all events from the buffer
    pub fn drain(&mut self) -> Vec<DashboardEvent> {
        std::mem::take(&mut self.events)
    }

    /// Get all events (without draining)
    pub fn events(&self) -> &[DashboardEvent] {
        &self.events
    }

    /// Get events as JSON array
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.events)
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get buffer size
    pub fn len(&self) -> usize {
        self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_hook_creation() {
        let (hook, _receiver) = DashboardHook::new();
        assert!(hook.is_enabled());
    }

    #[test]
    fn test_dashboard_event_serialization() {
        let event = DashboardEvent::RobotMoved {
            robot_id: 1,
            from_node: 10,
            to_node: 11,
            time_s: 5.5,
        };

        let json = event.to_json().unwrap();
        assert!(json.contains("RobotMoved"));
        assert!(json.contains("\"robot_id\":1"));
    }

    #[test]
    fn test_dashboard_event_time() {
        let event = DashboardEvent::Tick {
            time_s: 100.0,
            events_processed: 500,
        };
        assert_eq!(event.time_s(), 100.0);
    }

    #[test]
    fn test_event_filter() {
        let (mut hook, receiver) = DashboardHook::new();

        // With filter allowing moves
        hook.robot_moved(RobotId(1), NodeId(10), NodeId(11), SimTime::from_seconds(1.0));
        assert!(receiver.try_recv().is_ok());

        // Disable robot moves
        let mut filter = DashboardEventFilter::default();
        filter.robot_moves = false;
        hook.set_filter(filter);

        hook.robot_moved(RobotId(2), NodeId(20), NodeId(21), SimTime::from_seconds(2.0));
        assert!(matches!(receiver.try_recv(), Err(TryRecvError::Empty)));
    }

    #[test]
    fn test_event_buffer() {
        let mut buffer = DashboardEventBuffer::new(3);

        buffer.push(DashboardEvent::Tick { time_s: 1.0, events_processed: 10 });
        buffer.push(DashboardEvent::Tick { time_s: 2.0, events_processed: 20 });
        buffer.push(DashboardEvent::Tick { time_s: 3.0, events_processed: 30 });
        buffer.push(DashboardEvent::Tick { time_s: 4.0, events_processed: 40 });

        assert_eq!(buffer.len(), 3);
        // First event should have been dropped
        assert_eq!(buffer.events()[0].time_s(), 2.0);
    }
}
