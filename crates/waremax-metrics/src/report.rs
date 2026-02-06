//! Simulation report generation

use crate::timeseries::CongestionRanking;
use serde::{Deserialize, Serialize};

/// SLA section of the report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SLAReport {
    pub orders_on_time: u32,
    pub orders_late: u32,
    pub sla_miss_rate: f64,
    pub avg_lateness_s: f64,
    pub p95_lateness_s: f64,
    pub max_lateness_s: f64,
}

/// Congestion section of the report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CongestionReport {
    pub total_node_wait_events: u32,
    pub total_edge_wait_events: u32,
    pub total_node_wait_time_s: f64,
    pub total_edge_wait_time_s: f64,
    pub top_congested_nodes: Vec<CongestionRanking>,
    pub top_congested_edges: Vec<CongestionRanking>,
}

/// Battery/charging section of the report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatteryReport {
    pub total_charging_events: u32,
    pub total_energy_consumed_wh: f64,
    pub total_charging_time_s: f64,
    pub avg_soc_at_charge: f64,
}

/// Per-robot performance breakdown (v3)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RobotReport {
    pub robot_id: u32,
    pub tasks_completed: u32,
    pub distance_traveled_m: f64,
    pub energy_consumed_wh: f64,
    pub idle_time_s: f64,
    pub working_time_s: f64,
    pub charging_time_s: f64,
    pub maintenance_time_s: f64,
    pub failure_count: u32,
    pub utilization: f64,
}

/// Per-station performance breakdown (v3)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StationReport {
    pub station_id: u32,
    pub string_id: String,
    pub station_type: String,
    pub orders_served: u32,
    pub total_service_time_s: f64,
    pub avg_service_time_s: f64,
    pub avg_queue_length: f64,
    pub max_queue_length: usize,
    pub utilization: f64,
}

/// Reliability metrics summary (v3)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ReliabilityReport {
    pub total_failures: u32,
    pub total_maintenance_events: u32,
    pub total_repair_events: u32,
    pub actual_mtbf_hours: f64,
    pub mttr_s: f64,
    pub fleet_availability: f64,
    pub tasks_impacted_by_failures: u32,
}

/// Node congestion data for heatmap visualization (v3)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeCongestion {
    pub node_id: u32,
    pub x: f64,
    pub y: f64,
    pub total_wait_time_s: f64,
    pub wait_event_count: u32,
    pub congestion_score: f64,
}

/// Edge congestion data for heatmap visualization (v3)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeCongestion {
    pub edge_id: u32,
    pub from_node: u32,
    pub to_node: u32,
    pub total_wait_time_s: f64,
    pub wait_event_count: u32,
    pub congestion_score: f64,
}

/// Heatmap data for congestion visualization (v3)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HeatmapData {
    pub node_congestion: Vec<NodeCongestion>,
    pub edge_congestion: Vec<EdgeCongestion>,
}

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
    // v1 additions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sla: Option<SLAReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub congestion: Option<CongestionReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battery: Option<BatteryReport>,
    // v3 additions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub robot_reports: Option<Vec<RobotReport>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub station_reports: Option<Vec<StationReport>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reliability: Option<ReliabilityReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heatmap: Option<HeatmapData>,
}

impl SimulationReport {
    #[allow(clippy::too_many_arguments)]
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
            sla: None,
            congestion: None,
            battery: None,
            robot_reports: None,
            station_reports: None,
            reliability: None,
            heatmap: None,
        }
    }

    /// Set SLA report section
    pub fn with_sla(mut self, sla: SLAReport) -> Self {
        self.sla = Some(sla);
        self
    }

    /// Set congestion report section
    pub fn with_congestion(mut self, congestion: CongestionReport) -> Self {
        self.congestion = Some(congestion);
        self
    }

    /// Set battery report section
    pub fn with_battery(mut self, battery: BatteryReport) -> Self {
        self.battery = Some(battery);
        self
    }

    /// Set per-robot reports (v3)
    pub fn with_robot_reports(mut self, reports: Vec<RobotReport>) -> Self {
        self.robot_reports = Some(reports);
        self
    }

    /// Set per-station reports (v3)
    pub fn with_station_reports(mut self, reports: Vec<StationReport>) -> Self {
        self.station_reports = Some(reports);
        self
    }

    /// Set reliability report (v3)
    pub fn with_reliability(mut self, reliability: ReliabilityReport) -> Self {
        self.reliability = Some(reliability);
        self
    }

    /// Set heatmap data (v3)
    pub fn with_heatmap(mut self, heatmap: HeatmapData) -> Self {
        self.heatmap = Some(heatmap);
        self
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

        let mut output = format!(
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
        );

        // Add SLA section if present
        if let Some(ref sla) = self.sla {
            output.push_str(&format!(
                r#"
SLA Metrics:
  On-Time: {} ({:.1}%)
  Late: {} ({:.1}%)
  Avg Lateness: {:.1}s
  P95 Lateness: {:.1}s
  Max Lateness: {:.1}s
"#,
                sla.orders_on_time,
                (1.0 - sla.sla_miss_rate) * 100.0,
                sla.orders_late,
                sla.sla_miss_rate * 100.0,
                sla.avg_lateness_s,
                sla.p95_lateness_s,
                sla.max_lateness_s,
            ));
        }

        // Add congestion section if present
        if let Some(ref congestion) = self.congestion {
            output.push_str(&format!(
                r#"
Congestion:
  Node Wait Events: {}
  Edge Wait Events: {}
  Total Node Wait: {:.1}s
  Total Edge Wait: {:.1}s
"#,
                congestion.total_node_wait_events,
                congestion.total_edge_wait_events,
                congestion.total_node_wait_time_s,
                congestion.total_edge_wait_time_s,
            ));

            if !congestion.top_congested_nodes.is_empty() {
                output.push_str("  Top Congested Nodes:\n");
                for (i, node) in congestion.top_congested_nodes.iter().take(5).enumerate() {
                    if let Some(id) = node.node_id {
                        output.push_str(&format!(
                            "    {}. Node {} (score: {:.1}, waits: {})\n",
                            i + 1,
                            id.0,
                            node.score,
                            node.wait_events
                        ));
                    }
                }
            }
        }

        // Add battery section if present
        if let Some(ref battery) = self.battery {
            output.push_str(&format!(
                r#"
Battery:
  Charging Events: {}
  Energy Consumed: {:.1} Wh
  Total Charging Time: {:.1}s
  Avg SOC at Charge: {:.1}%
"#,
                battery.total_charging_events,
                battery.total_energy_consumed_wh,
                battery.total_charging_time_s,
                battery.avg_soc_at_charge * 100.0,
            ));
        }

        // Add reliability section if present (v3)
        if let Some(ref reliability) = self.reliability {
            output.push_str(&format!(
                r#"
Reliability:
  Failures: {}
  Maintenance Events: {}
  Repair Events: {}
  MTBF: {:.1} hours
  MTTR: {:.1}s
  Fleet Availability: {:.1}%
  Tasks Impacted: {}
"#,
                reliability.total_failures,
                reliability.total_maintenance_events,
                reliability.total_repair_events,
                reliability.actual_mtbf_hours,
                reliability.mttr_s,
                reliability.fleet_availability * 100.0,
                reliability.tasks_impacted_by_failures,
            ));
        }

        // Add per-robot summary if present (v3)
        if let Some(ref robots) = self.robot_reports {
            output.push_str(&format!("\nPer-Robot Summary ({} robots):\n", robots.len()));
            for robot in robots.iter().take(5) {
                output.push_str(&format!(
                    "  Robot {}: {} tasks, {:.0}m traveled, {:.1}% utilization\n",
                    robot.robot_id,
                    robot.tasks_completed,
                    robot.distance_traveled_m,
                    robot.utilization * 100.0,
                ));
            }
            if robots.len() > 5 {
                output.push_str(&format!("  ... and {} more robots\n", robots.len() - 5));
            }
        }

        // Add per-station summary if present (v3)
        if let Some(ref stations) = self.station_reports {
            output.push_str(&format!(
                "\nPer-Station Summary ({} stations):\n",
                stations.len()
            ));
            for station in stations.iter().take(5) {
                output.push_str(&format!(
                    "  {} ({}): {} served, avg queue {:.1}, {:.1}% utilization\n",
                    station.string_id,
                    station.station_type,
                    station.orders_served,
                    station.avg_queue_length,
                    station.utilization * 100.0,
                ));
            }
            if stations.len() > 5 {
                output.push_str(&format!("  ... and {} more stations\n", stations.len() - 5));
            }
        }

        output
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
            sla: None,
            congestion: None,
            battery: None,
            robot_reports: None,
            station_reports: None,
            reliability: None,
            heatmap: None,
        }
    }
}
