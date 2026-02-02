//! Bottleneck Detection
//!
//! Identifies congested nodes/edges, overloaded stations, and underutilized robots.

use serde::{Deserialize, Serialize};
use waremax_core::{NodeId, EdgeId, StationId, RobotId, ChargingStationId};

/// Types of bottlenecks that can be detected
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BottleneckType {
    /// A node with high congestion score
    CongestedNode {
        node_id: NodeId,
        score: f64,
        wait_events: u32,
        total_wait_s: f64,
    },
    /// An edge with high congestion score
    CongestedEdge {
        edge_id: EdgeId,
        score: f64,
        wait_events: u32,
        total_wait_s: f64,
    },
    /// A station with high utilization and long queues
    OverloadedStation {
        station_id: StationId,
        station_name: String,
        utilization: f64,
        avg_queue: f64,
        max_queue: usize,
    },
    /// Station with very low utilization (possible poor placement)
    UnderutilizedStation {
        station_id: StationId,
        station_name: String,
        utilization: f64,
    },
    /// Charging station with contention
    ChargingContention {
        station_id: ChargingStationId,
        station_name: String,
        avg_queue: f64,
        utilization: f64,
    },
    /// Robots that are underutilized
    UnderutilizedRobots {
        robot_ids: Vec<RobotId>,
        avg_utilization: f64,
    },
    /// Too few robots for the workload
    InsufficientRobots {
        robot_count: usize,
        avg_utilization: f64,
        estimated_shortfall: usize,
    },
}

impl BottleneckType {
    /// Get a severity score (0-100, higher = more severe)
    pub fn severity(&self) -> f64 {
        match self {
            BottleneckType::CongestedNode { score, .. } => (score / 100.0).min(100.0),
            BottleneckType::CongestedEdge { score, .. } => (score / 100.0).min(100.0),
            BottleneckType::OverloadedStation { utilization, avg_queue, .. } => {
                let util_factor = if *utilization > 0.95 { 50.0 } else { utilization * 40.0 };
                let queue_factor = (*avg_queue * 5.0).min(50.0);
                util_factor + queue_factor
            }
            BottleneckType::UnderutilizedStation { utilization, .. } => {
                ((1.0 - utilization) * 50.0).min(50.0)
            }
            BottleneckType::ChargingContention { avg_queue, utilization, .. } => {
                let util_factor = if *utilization > 0.9 { 40.0 } else { utilization * 30.0 };
                let queue_factor = (*avg_queue * 10.0).min(60.0);
                util_factor + queue_factor
            }
            BottleneckType::UnderutilizedRobots { avg_utilization, robot_ids } => {
                let count_factor = (robot_ids.len() as f64 * 5.0).min(30.0);
                let util_factor = (1.0 - avg_utilization) * 40.0;
                count_factor + util_factor
            }
            BottleneckType::InsufficientRobots { avg_utilization, estimated_shortfall, .. } => {
                let util_factor = if *avg_utilization > 0.95 { 50.0 } else { 0.0 };
                let shortfall_factor = (*estimated_shortfall as f64 * 10.0).min(50.0);
                util_factor + shortfall_factor
            }
        }
    }

    /// Get a human-readable name for the bottleneck type
    pub fn name(&self) -> &'static str {
        match self {
            BottleneckType::CongestedNode { .. } => "Congested Node",
            BottleneckType::CongestedEdge { .. } => "Congested Edge",
            BottleneckType::OverloadedStation { .. } => "Overloaded Station",
            BottleneckType::UnderutilizedStation { .. } => "Underutilized Station",
            BottleneckType::ChargingContention { .. } => "Charging Contention",
            BottleneckType::UnderutilizedRobots { .. } => "Underutilized Robots",
            BottleneckType::InsufficientRobots { .. } => "Insufficient Robots",
        }
    }

    /// Get a recommendation to address this bottleneck
    pub fn recommendation(&self) -> String {
        match self {
            BottleneckType::CongestedNode { node_id, .. } => {
                format!("Consider adding alternate routes around node {} or increasing its capacity", node_id)
            }
            BottleneckType::CongestedEdge { edge_id, .. } => {
                format!("Consider widening edge {} or implementing one-way traffic", edge_id)
            }
            BottleneckType::OverloadedStation { station_name, utilization, .. } => {
                if *utilization > 0.95 {
                    format!("Station {} is at capacity. Add parallel capacity or reduce service time", station_name)
                } else {
                    format!("Reduce queue buildup at {} by load balancing or adding concurrency", station_name)
                }
            }
            BottleneckType::UnderutilizedStation { station_name, .. } => {
                format!("Station {} has low utilization. Consider relocating or removing", station_name)
            }
            BottleneckType::ChargingContention { station_name, .. } => {
                format!("Charging station {} has contention. Add more bays or additional stations", station_name)
            }
            BottleneckType::UnderutilizedRobots { robot_ids, .. } => {
                format!("{} robots are underutilized. Consider reducing fleet size or rebalancing workload", robot_ids.len())
            }
            BottleneckType::InsufficientRobots { estimated_shortfall, .. } => {
                format!("Fleet appears undersized. Consider adding {} robots", estimated_shortfall)
            }
        }
    }

    /// Format as a detailed string
    pub fn to_detail_string(&self) -> String {
        match self {
            BottleneckType::CongestedNode { node_id, score, wait_events, total_wait_s } => {
                format!(
                    "Node {}: Score {:.1}, {} wait events, {:.1}s total wait",
                    node_id, score, wait_events, total_wait_s
                )
            }
            BottleneckType::CongestedEdge { edge_id, score, wait_events, total_wait_s } => {
                format!(
                    "Edge {}: Score {:.1}, {} wait events, {:.1}s total wait",
                    edge_id, score, wait_events, total_wait_s
                )
            }
            BottleneckType::OverloadedStation { station_name, utilization, avg_queue, max_queue, .. } => {
                format!(
                    "{}: {:.1}% utilization, avg queue {:.1}, max queue {}",
                    station_name, utilization * 100.0, avg_queue, max_queue
                )
            }
            BottleneckType::UnderutilizedStation { station_name, utilization, .. } => {
                format!("{}: Only {:.1}% utilization", station_name, utilization * 100.0)
            }
            BottleneckType::ChargingContention { station_name, avg_queue, utilization, .. } => {
                format!(
                    "{}: {:.1}% utilization, avg queue {:.1}",
                    station_name, utilization * 100.0, avg_queue
                )
            }
            BottleneckType::UnderutilizedRobots { robot_ids, avg_utilization } => {
                format!(
                    "{} robots at {:.1}% avg utilization",
                    robot_ids.len(), avg_utilization * 100.0
                )
            }
            BottleneckType::InsufficientRobots { robot_count, avg_utilization, estimated_shortfall } => {
                format!(
                    "{} robots at {:.1}% utilization, need ~{} more",
                    robot_count, avg_utilization * 100.0, estimated_shortfall
                )
            }
        }
    }
}

/// Input data for bottleneck detection
#[derive(Clone, Debug, Default)]
pub struct BottleneckInputData {
    /// Node congestion data: (node_id, score, wait_events, total_wait_s)
    pub node_congestion: Vec<(NodeId, f64, u32, f64)>,
    /// Edge congestion data: (edge_id, score, wait_events, total_wait_s)
    pub edge_congestion: Vec<(EdgeId, f64, u32, f64)>,
    /// Station data: (station_id, name, utilization, avg_queue, max_queue)
    pub station_data: Vec<(StationId, String, f64, f64, usize)>,
    /// Charging station data: (station_id, name, utilization, avg_queue)
    pub charging_data: Vec<(ChargingStationId, String, f64, f64)>,
    /// Robot utilizations: (robot_id, utilization)
    pub robot_utilizations: Vec<(RobotId, f64)>,
}

/// Configuration for bottleneck detection thresholds
#[derive(Clone, Debug)]
pub struct BottleneckConfig {
    /// Minimum congestion score to consider a node congested
    pub node_congestion_threshold: f64,
    /// Minimum congestion score to consider an edge congested
    pub edge_congestion_threshold: f64,
    /// Station utilization above this is considered overloaded
    pub station_overload_utilization: f64,
    /// Average queue above this is considered a problem
    pub station_queue_threshold: f64,
    /// Station utilization below this is considered underutilized
    pub station_underutilization: f64,
    /// Charging utilization above this with queue is contention
    pub charging_contention_utilization: f64,
    /// Robot utilization below this is underutilized
    pub robot_underutilization: f64,
    /// Robot utilization above this suggests need for more robots
    pub robot_overutilization: f64,
    /// Maximum number of bottlenecks to return per category
    pub max_per_category: usize,
}

impl Default for BottleneckConfig {
    fn default() -> Self {
        Self {
            node_congestion_threshold: 50.0,
            edge_congestion_threshold: 50.0,
            station_overload_utilization: 0.85,
            station_queue_threshold: 2.0,
            station_underutilization: 0.3,
            charging_contention_utilization: 0.8,
            robot_underutilization: 0.4,
            robot_overutilization: 0.9,
            max_per_category: 10,
        }
    }
}

/// Full bottleneck analysis results
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BottleneckAnalysis {
    /// All detected bottlenecks, sorted by severity
    pub bottlenecks: Vec<BottleneckType>,
    /// Summary statistics
    pub summary: BottleneckSummary,
}

/// Summary of bottleneck analysis
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BottleneckSummary {
    /// Total number of bottlenecks detected
    pub total_count: usize,
    /// Number of congestion bottlenecks (nodes + edges)
    pub congestion_count: usize,
    /// Number of station bottlenecks
    pub station_count: usize,
    /// Number of robot-related bottlenecks
    pub robot_count: usize,
    /// Average severity score
    pub avg_severity: f64,
    /// Maximum severity score
    pub max_severity: f64,
    /// Primary bottleneck type (most severe)
    pub primary_bottleneck: Option<String>,
}

impl BottleneckAnalysis {
    /// Format as a human-readable report
    pub fn to_string(&self) -> String {
        let mut output = String::new();
        output.push_str("Bottleneck Analysis\n");
        output.push_str(&"=".repeat(50));
        output.push('\n');

        output.push_str(&format!("Total Bottlenecks: {}\n", self.summary.total_count));
        output.push_str(&format!("  Congestion: {}\n", self.summary.congestion_count));
        output.push_str(&format!("  Station: {}\n", self.summary.station_count));
        output.push_str(&format!("  Robot: {}\n", self.summary.robot_count));

        if let Some(ref primary) = self.summary.primary_bottleneck {
            output.push_str(&format!("\nPrimary Issue: {}\n", primary));
        }

        output.push_str(&format!("Avg Severity: {:.1}/100\n", self.summary.avg_severity));
        output.push_str(&format!("Max Severity: {:.1}/100\n", self.summary.max_severity));

        if !self.bottlenecks.is_empty() {
            output.push_str(&"\n".to_string());
            output.push_str(&"-".repeat(50));
            output.push_str("\nTop Bottlenecks:\n");

            for (i, bottleneck) in self.bottlenecks.iter().take(10).enumerate() {
                output.push_str(&format!(
                    "{}. [{}] {} (severity: {:.1})\n",
                    i + 1,
                    bottleneck.name(),
                    bottleneck.to_detail_string(),
                    bottleneck.severity()
                ));
                output.push_str(&format!("   Recommendation: {}\n", bottleneck.recommendation()));
            }
        }

        output
    }

    /// Get recommendations for all bottlenecks
    pub fn recommendations(&self) -> Vec<String> {
        self.bottlenecks.iter().map(|b| b.recommendation()).collect()
    }

    /// Get top N bottlenecks by severity
    pub fn top_bottlenecks(&self, n: usize) -> &[BottleneckType] {
        let len = self.bottlenecks.len().min(n);
        &self.bottlenecks[..len]
    }
}

/// Bottleneck detector
pub struct BottleneckDetector {
    config: BottleneckConfig,
}

impl BottleneckDetector {
    /// Create a new detector with default config
    pub fn new() -> Self {
        Self {
            config: BottleneckConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: BottleneckConfig) -> Self {
        Self { config }
    }

    /// Detect bottlenecks from input data
    pub fn detect(&self, input: &BottleneckInputData) -> BottleneckAnalysis {
        let mut bottlenecks = Vec::new();

        // Detect congested nodes
        self.detect_congested_nodes(&input.node_congestion, &mut bottlenecks);

        // Detect congested edges
        self.detect_congested_edges(&input.edge_congestion, &mut bottlenecks);

        // Detect station issues
        self.detect_station_issues(&input.station_data, &mut bottlenecks);

        // Detect charging contention
        self.detect_charging_issues(&input.charging_data, &mut bottlenecks);

        // Detect robot issues
        self.detect_robot_issues(&input.robot_utilizations, &mut bottlenecks);

        // Sort by severity
        bottlenecks.sort_by(|a, b| {
            b.severity().partial_cmp(&a.severity()).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Generate summary
        let summary = self.generate_summary(&bottlenecks);

        BottleneckAnalysis {
            bottlenecks,
            summary,
        }
    }

    fn detect_congested_nodes(
        &self,
        nodes: &[(NodeId, f64, u32, f64)],
        bottlenecks: &mut Vec<BottleneckType>,
    ) {
        let mut congested: Vec<_> = nodes
            .iter()
            .filter(|(_, score, _, _)| *score >= self.config.node_congestion_threshold)
            .cloned()
            .collect();

        congested.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (node_id, score, wait_events, total_wait_s) in congested.into_iter().take(self.config.max_per_category) {
            bottlenecks.push(BottleneckType::CongestedNode {
                node_id,
                score,
                wait_events,
                total_wait_s,
            });
        }
    }

    fn detect_congested_edges(
        &self,
        edges: &[(EdgeId, f64, u32, f64)],
        bottlenecks: &mut Vec<BottleneckType>,
    ) {
        let mut congested: Vec<_> = edges
            .iter()
            .filter(|(_, score, _, _)| *score >= self.config.edge_congestion_threshold)
            .cloned()
            .collect();

        congested.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (edge_id, score, wait_events, total_wait_s) in congested.into_iter().take(self.config.max_per_category) {
            bottlenecks.push(BottleneckType::CongestedEdge {
                edge_id,
                score,
                wait_events,
                total_wait_s,
            });
        }
    }

    fn detect_station_issues(
        &self,
        stations: &[(StationId, String, f64, f64, usize)],
        bottlenecks: &mut Vec<BottleneckType>,
    ) {
        for (station_id, name, utilization, avg_queue, max_queue) in stations {
            // Check for overloaded stations
            if *utilization >= self.config.station_overload_utilization
                || *avg_queue >= self.config.station_queue_threshold
            {
                bottlenecks.push(BottleneckType::OverloadedStation {
                    station_id: *station_id,
                    station_name: name.clone(),
                    utilization: *utilization,
                    avg_queue: *avg_queue,
                    max_queue: *max_queue,
                });
            }
            // Check for underutilized stations
            else if *utilization < self.config.station_underutilization {
                bottlenecks.push(BottleneckType::UnderutilizedStation {
                    station_id: *station_id,
                    station_name: name.clone(),
                    utilization: *utilization,
                });
            }
        }
    }

    fn detect_charging_issues(
        &self,
        charging: &[(ChargingStationId, String, f64, f64)],
        bottlenecks: &mut Vec<BottleneckType>,
    ) {
        for (station_id, name, utilization, avg_queue) in charging {
            if *utilization >= self.config.charging_contention_utilization && *avg_queue > 0.5 {
                bottlenecks.push(BottleneckType::ChargingContention {
                    station_id: *station_id,
                    station_name: name.clone(),
                    avg_queue: *avg_queue,
                    utilization: *utilization,
                });
            }
        }
    }

    fn detect_robot_issues(
        &self,
        robots: &[(RobotId, f64)],
        bottlenecks: &mut Vec<BottleneckType>,
    ) {
        if robots.is_empty() {
            return;
        }

        let avg_utilization: f64 = robots.iter().map(|(_, u)| u).sum::<f64>() / robots.len() as f64;

        // Check for underutilized robots
        let underutilized: Vec<RobotId> = robots
            .iter()
            .filter(|(_, u)| *u < self.config.robot_underutilization)
            .map(|(id, _)| *id)
            .collect();

        if !underutilized.is_empty() {
            let under_avg: f64 = robots
                .iter()
                .filter(|(_, u)| *u < self.config.robot_underutilization)
                .map(|(_, u)| u)
                .sum::<f64>()
                / underutilized.len() as f64;

            bottlenecks.push(BottleneckType::UnderutilizedRobots {
                robot_ids: underutilized,
                avg_utilization: under_avg,
            });
        }

        // Check if fleet is undersized (all robots overutilized)
        if avg_utilization >= self.config.robot_overutilization {
            // Estimate how many more robots might be needed
            // If at 95% utilization, need ~5% more capacity
            let capacity_ratio = avg_utilization / 0.75; // Target 75% utilization
            let estimated_shortfall = ((robots.len() as f64 * capacity_ratio) - robots.len() as f64)
                .ceil() as usize;

            if estimated_shortfall > 0 {
                bottlenecks.push(BottleneckType::InsufficientRobots {
                    robot_count: robots.len(),
                    avg_utilization,
                    estimated_shortfall,
                });
            }
        }
    }

    fn generate_summary(&self, bottlenecks: &[BottleneckType]) -> BottleneckSummary {
        let total_count = bottlenecks.len();

        let congestion_count = bottlenecks
            .iter()
            .filter(|b| matches!(b, BottleneckType::CongestedNode { .. } | BottleneckType::CongestedEdge { .. }))
            .count();

        let station_count = bottlenecks
            .iter()
            .filter(|b| {
                matches!(
                    b,
                    BottleneckType::OverloadedStation { .. }
                        | BottleneckType::UnderutilizedStation { .. }
                        | BottleneckType::ChargingContention { .. }
                )
            })
            .count();

        let robot_count = bottlenecks
            .iter()
            .filter(|b| {
                matches!(
                    b,
                    BottleneckType::UnderutilizedRobots { .. } | BottleneckType::InsufficientRobots { .. }
                )
            })
            .count();

        let severities: Vec<f64> = bottlenecks.iter().map(|b| b.severity()).collect();
        let avg_severity = if severities.is_empty() {
            0.0
        } else {
            severities.iter().sum::<f64>() / severities.len() as f64
        };
        let max_severity = severities.iter().cloned().fold(0.0, f64::max);

        let primary_bottleneck = bottlenecks.first().map(|b| {
            format!("{}: {}", b.name(), b.to_detail_string())
        });

        BottleneckSummary {
            total_count,
            congestion_count,
            station_count,
            robot_count,
            avg_severity,
            max_severity,
            primary_bottleneck,
        }
    }
}

impl Default for BottleneckDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bottleneck_severity() {
        let node = BottleneckType::CongestedNode {
            node_id: NodeId(1),
            score: 150.0,
            wait_events: 10,
            total_wait_s: 50.0,
        };
        assert!(node.severity() > 0.0);

        let station = BottleneckType::OverloadedStation {
            station_id: StationId(1),
            station_name: "S1".to_string(),
            utilization: 0.98,
            avg_queue: 5.0,
            max_queue: 10,
        };
        assert!(station.severity() > 50.0);
    }

    #[test]
    fn test_detect_congested_nodes() {
        let detector = BottleneckDetector::new();

        let input = BottleneckInputData {
            node_congestion: vec![
                (NodeId(1), 100.0, 10, 20.0),
                (NodeId(2), 30.0, 3, 6.0),  // Below threshold
                (NodeId(3), 200.0, 20, 40.0),
            ],
            ..Default::default()
        };

        let analysis = detector.detect(&input);

        // Should detect 2 congested nodes (those above threshold of 50)
        let congested: Vec<_> = analysis
            .bottlenecks
            .iter()
            .filter(|b| matches!(b, BottleneckType::CongestedNode { .. }))
            .collect();
        assert_eq!(congested.len(), 2);
    }

    #[test]
    fn test_detect_station_issues() {
        let detector = BottleneckDetector::new();

        let input = BottleneckInputData {
            station_data: vec![
                (StationId(1), "S1".to_string(), 0.95, 3.0, 8),  // Overloaded
                (StationId(2), "S2".to_string(), 0.50, 0.5, 2),  // OK
                (StationId(3), "S3".to_string(), 0.20, 0.1, 1),  // Underutilized
            ],
            ..Default::default()
        };

        let analysis = detector.detect(&input);

        // Should detect 1 overloaded and 1 underutilized
        let overloaded = analysis
            .bottlenecks
            .iter()
            .any(|b| matches!(b, BottleneckType::OverloadedStation { station_id, .. } if station_id.0 == 1));
        let underutilized = analysis
            .bottlenecks
            .iter()
            .any(|b| matches!(b, BottleneckType::UnderutilizedStation { station_id, .. } if station_id.0 == 3));

        assert!(overloaded);
        assert!(underutilized);
    }

    #[test]
    fn test_detect_insufficient_robots() {
        let detector = BottleneckDetector::new();

        let input = BottleneckInputData {
            robot_utilizations: vec![
                (RobotId(0), 0.95),
                (RobotId(1), 0.92),
                (RobotId(2), 0.94),
            ],
            ..Default::default()
        };

        let analysis = detector.detect(&input);

        let insufficient = analysis
            .bottlenecks
            .iter()
            .any(|b| matches!(b, BottleneckType::InsufficientRobots { .. }));
        assert!(insufficient);
    }

    #[test]
    fn test_recommendations() {
        let node = BottleneckType::CongestedNode {
            node_id: NodeId(42),
            score: 100.0,
            wait_events: 10,
            total_wait_s: 30.0,
        };
        assert!(node.recommendation().contains("42"));
    }
}
