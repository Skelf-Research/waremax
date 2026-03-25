//! Metrics collection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use waremax_core::{RobotId, ScheduledEvent, SimTime, StationId};

/// SLA (Service Level Agreement) metrics for order tracking
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SLAMetrics {
    pub orders_on_time: u32,
    pub orders_late: u32,
    /// Lateness distribution in seconds (positive = late, negative = early)
    pub lateness_distribution: Vec<f64>,
}

impl SLAMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an order completion with lateness (positive = late, negative = early)
    pub fn record_completion(&mut self, lateness_s: f64) {
        self.lateness_distribution.push(lateness_s);
        if lateness_s > 0.0 {
            self.orders_late += 1;
        } else {
            self.orders_on_time += 1;
        }
    }

    /// Total orders tracked
    pub fn total_orders(&self) -> u32 {
        self.orders_on_time + self.orders_late
    }

    /// SLA miss rate (0.0 to 1.0)
    pub fn sla_miss_rate(&self) -> f64 {
        let total = self.total_orders();
        if total == 0 {
            0.0
        } else {
            self.orders_late as f64 / total as f64
        }
    }

    /// SLA achievement rate (0.0 to 1.0)
    pub fn sla_achievement_rate(&self) -> f64 {
        1.0 - self.sla_miss_rate()
    }

    /// Average lateness in seconds (only for late orders)
    pub fn avg_lateness_s(&self) -> f64 {
        let late_only: Vec<f64> = self
            .lateness_distribution
            .iter()
            .filter(|&&l| l > 0.0)
            .copied()
            .collect();

        if late_only.is_empty() {
            0.0
        } else {
            late_only.iter().sum::<f64>() / late_only.len() as f64
        }
    }

    /// P95 lateness in seconds (95th percentile of late orders)
    pub fn p95_lateness_s(&self) -> f64 {
        let mut late_only: Vec<f64> = self
            .lateness_distribution
            .iter()
            .filter(|&&l| l > 0.0)
            .copied()
            .collect();

        if late_only.is_empty() {
            return 0.0;
        }

        late_only.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((late_only.len() as f64) * 0.95) as usize;
        late_only
            .get(idx.min(late_only.len() - 1))
            .copied()
            .unwrap_or(0.0)
    }

    /// Maximum lateness in seconds
    pub fn max_lateness_s(&self) -> f64 {
        self.lateness_distribution
            .iter()
            .filter(|&&l| l > 0.0)
            .copied()
            .fold(0.0, f64::max)
    }

    /// Average earliness in seconds (only for on-time orders)
    pub fn avg_earliness_s(&self) -> f64 {
        let early_only: Vec<f64> = self
            .lateness_distribution
            .iter()
            .filter(|&&l| l <= 0.0)
            .map(|&l| -l) // Convert to positive
            .collect();

        if early_only.is_empty() {
            0.0
        } else {
            early_only.iter().sum::<f64>() / early_only.len() as f64
        }
    }
}

/// Collects metrics during simulation
#[derive(Clone, Default)]
pub struct MetricsCollector {
    // Order tracking
    order_completion_times: Vec<f64>,
    orders_completed: u32,
    orders_late: u32,

    // SLA tracking (v1)
    pub sla_metrics: SLAMetrics,

    // Robot tracking
    robot_stats: HashMap<RobotId, RobotStats>,

    // Station tracking
    station_stats: HashMap<StationId, StationStats>,

    // Event counts
    event_counts: HashMap<String, u64>,

    // sled database path (optional)
    _db_path: Option<String>,

    // v1: Charging SOC samples for average calculation
    charging_soc_samples: Vec<f64>,

    // v3: Failure and maintenance tracking
    failure_count: u32,
    maintenance_count: u32,
    repair_count: u32,
    repair_durations: Vec<f64>,
    maintenance_durations: Vec<f64>,
    tasks_impacted_by_failures: u32,
}

#[derive(Clone, Default)]
struct RobotStats {
    tasks_completed: u32,
    _total_distance: f64,
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
            _db_path: Some(db_path.to_string()),
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

    /// Record order completion with SLA tracking
    /// lateness_s: positive = late, negative = early
    pub fn record_order_with_sla(&mut self, cycle_time: SimTime, lateness_s: f64) {
        self.order_completion_times.push(cycle_time.as_seconds());
        self.orders_completed += 1;
        if lateness_s > 0.0 {
            self.orders_late += 1;
        }
        self.sla_metrics.record_completion(lateness_s);
    }

    pub fn record_task_complete(&mut self, robot_id: RobotId) {
        self.robot_stats
            .entry(robot_id)
            .or_default()
            .tasks_completed += 1;
    }

    pub fn record_station_service(&mut self, station_id: StationId) {
        self.station_stats
            .entry(station_id)
            .or_default()
            .total_served += 1;
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
            self.order_completion_times.iter().sum::<f64>()
                / self.order_completion_times.len() as f64
        }
    }

    pub fn p95_cycle_time(&self) -> f64 {
        if self.order_completion_times.is_empty() {
            0.0
        } else {
            let mut sorted = self.order_completion_times.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let idx = (sorted.len() as f64 * 0.95) as usize;
            sorted
                .get(idx.min(sorted.len() - 1))
                .copied()
                .unwrap_or(0.0)
        }
    }

    pub fn total_events(&self) -> u64 {
        self.event_counts.values().sum()
    }

    /// Record SOC when a robot starts charging (v1)
    pub fn record_charging_start_soc(&mut self, soc: f64) {
        self.charging_soc_samples.push(soc);
    }

    /// Get average SOC at charge start (v1)
    pub fn avg_soc_at_charge(&self) -> f64 {
        if self.charging_soc_samples.is_empty() {
            0.0
        } else {
            self.charging_soc_samples.iter().sum::<f64>() / self.charging_soc_samples.len() as f64
        }
    }

    // === v3: Failure and Maintenance Metrics ===

    /// Record a robot failure
    pub fn record_robot_failure(&mut self) {
        self.failure_count += 1;
        self.tasks_impacted_by_failures += 1;
    }

    /// Record scheduled maintenance start
    pub fn record_maintenance_start(&mut self) {
        // Maintenance count is recorded on completion
    }

    /// Record scheduled maintenance completion
    pub fn record_maintenance_end(&mut self, duration_s: f64) {
        self.maintenance_count += 1;
        self.maintenance_durations.push(duration_s);
    }

    /// Record repair (failure recovery) start
    pub fn record_repair_start(&mut self) {
        // Repair count is recorded on completion
    }

    /// Record repair completion
    pub fn record_repair_end(&mut self, duration_s: f64) {
        self.repair_count += 1;
        self.repair_durations.push(duration_s);
    }

    /// Get total failure count
    pub fn failure_count(&self) -> u32 {
        self.failure_count
    }

    /// Get total maintenance count
    pub fn maintenance_count(&self) -> u32 {
        self.maintenance_count
    }

    /// Get total repair count
    pub fn repair_count(&self) -> u32 {
        self.repair_count
    }

    /// Get tasks impacted by failures
    pub fn tasks_impacted_by_failures(&self) -> u32 {
        self.tasks_impacted_by_failures
    }

    /// Calculate actual Mean Time Between Failures (hours)
    /// Requires total operating hours across the fleet
    pub fn actual_mtbf_hours(&self, total_operating_hours: f64) -> f64 {
        if self.failure_count == 0 {
            total_operating_hours // No failures = MTBF is entire operating time
        } else {
            total_operating_hours / self.failure_count as f64
        }
    }

    /// Calculate Mean Time To Repair (seconds)
    pub fn mttr_s(&self) -> f64 {
        if self.repair_durations.is_empty() {
            0.0
        } else {
            self.repair_durations.iter().sum::<f64>() / self.repair_durations.len() as f64
        }
    }

    /// Calculate average maintenance duration (seconds)
    pub fn avg_maintenance_duration_s(&self) -> f64 {
        if self.maintenance_durations.is_empty() {
            0.0
        } else {
            self.maintenance_durations.iter().sum::<f64>() / self.maintenance_durations.len() as f64
        }
    }

    /// Calculate fleet availability (fraction of time robots were operational)
    /// Returns value between 0.0 and 1.0
    pub fn fleet_availability(&self, total_sim_time_s: f64, robot_count: u32) -> f64 {
        if robot_count == 0 || total_sim_time_s <= 0.0 {
            return 1.0;
        }

        let total_fleet_time = total_sim_time_s * robot_count as f64;
        let total_downtime: f64 = self.repair_durations.iter().sum::<f64>()
            + self.maintenance_durations.iter().sum::<f64>();

        if total_downtime >= total_fleet_time {
            0.0
        } else {
            1.0 - (total_downtime / total_fleet_time)
        }
    }
}
