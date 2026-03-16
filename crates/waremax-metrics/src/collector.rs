//! Metrics collection

use waremax_core::{SimTime, ScheduledEvent, RobotId, StationId, OrderId};
use std::collections::HashMap;

/// Collects metrics during simulation
#[derive(Clone, Default)]
pub struct MetricsCollector {
    // Order tracking
    order_completion_times: Vec<f64>,
    orders_completed: u32,
    orders_late: u32,

    // Robot tracking
    robot_stats: HashMap<RobotId, RobotStats>,

    // Station tracking
    station_stats: HashMap<StationId, StationStats>,

    // Event counts
    event_counts: HashMap<String, u64>,

    // sled database path (optional)
    db_path: Option<String>,
}

#[derive(Clone, Default)]
struct RobotStats {
    tasks_completed: u32,
    total_distance: f64,
}

#[derive(Clone, Default)]
struct StationStats {
    total_served: u32,
    queue_samples: Vec<(f64, usize)>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_db(db_path: &str) -> Self {
        Self {
            db_path: Some(db_path.to_string()),
            ..Self::default()
        }
    }

    pub fn record_event(&mut self, event: &ScheduledEvent) {
        let event_type = event.event.event_type_name().to_string();
        *self.event_counts.entry(event_type).or_insert(0) += 1;
    }

    pub fn record_order_complete(&mut self, cycle_time: SimTime, is_late: bool) {
        self.order_completion_times.push(cycle_time.as_seconds());
        self.orders_completed += 1;
        if is_late {
            self.orders_late += 1;
        }
    }

    pub fn record_task_complete(&mut self, robot_id: RobotId) {
        self.robot_stats.entry(robot_id).or_default().tasks_completed += 1;
    }

    pub fn record_station_service(&mut self, station_id: StationId) {
        self.station_stats.entry(station_id).or_default().total_served += 1;
    }

    pub fn record_queue_length(&mut self, station_id: StationId, time: SimTime, length: usize) {
        self.station_stats
            .entry(station_id)
            .or_default()
            .queue_samples
            .push((time.as_seconds(), length));
    }

    pub fn orders_completed(&self) -> u32 {
        self.orders_completed
    }

    pub fn orders_late(&self) -> u32 {
        self.orders_late
    }

    pub fn avg_cycle_time(&self) -> f64 {
        if self.order_completion_times.is_empty() {
            0.0
        } else {
            self.order_completion_times.iter().sum::<f64>() / self.order_completion_times.len() as f64
        }
    }

    pub fn p95_cycle_time(&self) -> f64 {
        if self.order_completion_times.is_empty() {
            0.0
        } else {
            let mut sorted = self.order_completion_times.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let idx = (sorted.len() as f64 * 0.95) as usize;
            sorted.get(idx.min(sorted.len() - 1)).copied().unwrap_or(0.0)
        }
    }

    pub fn total_events(&self) -> u64 {
        self.event_counts.values().sum()
    }
}
