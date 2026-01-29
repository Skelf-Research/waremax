//! Simulation report generation

use serde::{Deserialize, Serialize};

/// Final simulation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationReport {
    pub duration_s: f64,
    pub events_processed: u64,
    pub orders_completed: u32,
    pub orders_late: u32,
    pub throughput_per_hour: f64,
    pub avg_cycle_time_s: f64,
    pub p95_cycle_time_s: f64,
    pub robot_utilization: f64,
    pub station_utilization: f64,
}

impl SimulationReport {
    pub fn new(
        duration_s: f64,
        events_processed: u64,
        orders_completed: u32,
        orders_late: u32,
        avg_cycle_time_s: f64,
        p95_cycle_time_s: f64,
        robot_utilization: f64,
        station_utilization: f64,
    ) -> Self {
        let duration_hours = duration_s / 3600.0;
        let throughput_per_hour = if duration_hours > 0.0 {
            orders_completed as f64 / duration_hours
        } else {
            0.0
        };

        Self {
            duration_s,
            events_processed,
            orders_completed,
            orders_late,
            throughput_per_hour,
            avg_cycle_time_s,
            p95_cycle_time_s,
            robot_utilization,
            station_utilization,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn summary(&self) -> String {
        let late_pct = if self.orders_completed > 0 {
            100.0 * self.orders_late as f64 / self.orders_completed as f64
        } else {
            0.0
        };

        format!(
            r#"
Simulation Report
=================
Duration: {:.2} hours
Events Processed: {}

Orders:
  Completed: {}
  Late: {} ({:.1}%)
  Throughput: {:.1} orders/hour

Cycle Time:
  Average: {:.1} seconds
  P95: {:.1} seconds

Utilization:
  Robots: {:.1}%
  Stations: {:.1}%
"#,
            self.duration_s / 3600.0,
            self.events_processed,
            self.orders_completed,
            self.orders_late,
            late_pct,
            self.throughput_per_hour,
            self.avg_cycle_time_s,
            self.p95_cycle_time_s,
            self.robot_utilization * 100.0,
            self.station_utilization * 100.0,
        )
    }
}

impl Default for SimulationReport {
    fn default() -> Self {
        Self {
            duration_s: 0.0,
            events_processed: 0,
            orders_completed: 0,
            orders_late: 0,
            throughput_per_hour: 0.0,
            avg_cycle_time_s: 0.0,
            p95_cycle_time_s: 0.0,
            robot_utilization: 0.0,
            station_utilization: 0.0,
        }
    }
}
