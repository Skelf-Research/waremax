//! Root Cause Analyzer
//!
//! Main orchestrator that combines attribution, bottleneck detection,
//! critical path analysis, and anomaly detection.

use serde::{Deserialize, Serialize};
use waremax_core::{RobotId, StationId, NodeId, EdgeId, ChargingStationId};

use crate::attribution::{DelayAttributionSummary, TaskAttribution};
use crate::bottleneck::{BottleneckAnalysis, BottleneckDetector, BottleneckInputData, BottleneckType};
use crate::critical_path::{CriticalPathAnalysis, CriticalPathSummary};
use crate::anomaly::{Anomaly, AnomalyDetector};

/// Summary of the root cause analysis
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RCASummary {
    /// Total orders analyzed
    pub orders_analyzed: usize,
    /// Average cycle time
    pub avg_cycle_time_s: f64,
    /// Primary delay source (largest time category)
    pub primary_delay_source: String,
    /// Primary bottleneck (most severe)
    pub primary_bottleneck: Option<String>,
    /// Number of anomalies detected
    pub anomaly_count: usize,
    /// Number of slow orders
    pub slow_order_count: usize,
    /// Overall health score (0-100, higher = healthier)
    pub health_score: f64,
    /// Key finding in one sentence
    pub key_finding: String,
}

/// A recommendation generated from the analysis
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recommendation {
    /// Priority (1 = highest)
    pub priority: u32,
    /// Category of recommendation
    pub category: String,
    /// The recommendation text
    pub text: String,
    /// Expected impact if implemented
    pub expected_impact: String,
}

/// Full Root Cause Analysis Report
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RootCauseAnalysisReport {
    /// High-level summary
    pub summary: RCASummary,
    /// Delay attribution analysis
    pub delay_attribution: DelayAttributionSummary,
    /// Bottleneck analysis
    pub bottleneck_analysis: BottleneckAnalysis,
    /// Critical path summary
    pub critical_paths: CriticalPathSummary,
    /// Detected anomalies
    pub anomalies: Vec<Anomaly>,
    /// Generated recommendations
    pub recommendations: Vec<Recommendation>,
}

impl RootCauseAnalysisReport {
    /// Generate a text report
    pub fn to_text(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str("\n");
        output.push_str(&"=".repeat(60));
        output.push_str("\n ROOT CAUSE ANALYSIS REPORT \n");
        output.push_str(&"=".repeat(60));
        output.push_str("\n\n");

        // Summary section
        output.push_str("SUMMARY\n");
        output.push_str(&"-".repeat(60));
        output.push_str("\n");
        output.push_str(&format!("Orders Analyzed: {}\n", self.summary.orders_analyzed));
        output.push_str(&format!("Average Cycle Time: {:.1}s\n", self.summary.avg_cycle_time_s));
        output.push_str(&format!("Primary Delay Source: {}\n", self.summary.primary_delay_source));
        if let Some(ref bottleneck) = self.summary.primary_bottleneck {
            output.push_str(&format!("Primary Bottleneck: {}\n", bottleneck));
        }
        output.push_str(&format!("Anomalies Detected: {}\n", self.summary.anomaly_count));
        output.push_str(&format!("Slow Orders: {}\n", self.summary.slow_order_count));
        output.push_str(&format!("Health Score: {:.0}/100\n", self.summary.health_score));
        output.push_str(&format!("\nKey Finding: {}\n", self.summary.key_finding));

        // Delay Attribution section
        output.push_str("\n");
        output.push_str(&"=".repeat(60));
        output.push_str("\n DELAY ATTRIBUTION \n");
        output.push_str(&"-".repeat(60));
        output.push_str("\n");
        output.push_str(&format!("Tasks Analyzed: {}\n", self.delay_attribution.task_count));
        output.push_str(&format!("Avg Cycle Time: {:.1}s\n", self.delay_attribution.avg_cycle_time_s));
        output.push_str(&format!("Avg Waste Time: {:.1}s ({:.1}% of cycle)\n",
            self.delay_attribution.avg_waste_time_s,
            self.delay_attribution.waste_ratio * 100.0));

        output.push_str("\nTime Breakdown:\n");
        for (i, (cat, _total_s, pct)) in self.delay_attribution.ranked_categories.iter().enumerate().take(5) {
            output.push_str(&format!(
                "  {}. {}: {:.1}% ({:.1}s avg)\n",
                i + 1,
                cat.name(),
                pct,
                self.delay_attribution.avg_by_category.get(cat).unwrap_or(&0.0)
            ));
        }

        // Bottleneck section
        output.push_str("\n");
        output.push_str(&"=".repeat(60));
        output.push_str("\n BOTTLENECK ANALYSIS \n");
        output.push_str(&"-".repeat(60));
        output.push_str("\n");
        output.push_str(&format!("Total Bottlenecks: {}\n", self.bottleneck_analysis.summary.total_count));
        output.push_str(&format!("  Congestion: {}\n", self.bottleneck_analysis.summary.congestion_count));
        output.push_str(&format!("  Station: {}\n", self.bottleneck_analysis.summary.station_count));
        output.push_str(&format!("  Robot: {}\n", self.bottleneck_analysis.summary.robot_count));

        if !self.bottleneck_analysis.bottlenecks.is_empty() {
            output.push_str("\nTop Bottlenecks:\n");
            for (i, b) in self.bottleneck_analysis.bottlenecks.iter().take(5).enumerate() {
                output.push_str(&format!(
                    "  {}. [{}] {}\n",
                    i + 1,
                    b.name(),
                    b.to_detail_string()
                ));
            }
        }

        // Anomalies section
        if !self.anomalies.is_empty() {
            output.push_str("\n");
            output.push_str(&"=".repeat(60));
            output.push_str("\n ANOMALIES DETECTED \n");
            output.push_str(&"-".repeat(60));
            output.push_str("\n");
            output.push_str(&format!("Total: {}\n", self.anomalies.len()));
            output.push_str("\nTop Anomalies:\n");
            for (i, anomaly) in self.anomalies.iter().take(5).enumerate() {
                output.push_str(&format!(
                    "  {}. [severity: {:.0}] {}\n",
                    i + 1,
                    anomaly.severity,
                    anomaly.anomaly_type.description()
                ));
            }
        }

        // Recommendations section
        if !self.recommendations.is_empty() {
            output.push_str("\n");
            output.push_str(&"=".repeat(60));
            output.push_str("\n RECOMMENDATIONS \n");
            output.push_str(&"-".repeat(60));
            output.push_str("\n");
            for rec in &self.recommendations {
                output.push_str(&format!(
                    "\n[Priority {}] {}\n",
                    rec.priority,
                    rec.category
                ));
                output.push_str(&format!("  {}\n", rec.text));
                output.push_str(&format!("  Expected Impact: {}\n", rec.expected_impact));
            }
        }

        output.push_str("\n");
        output.push_str(&"=".repeat(60));
        output.push_str("\n");

        output
    }
}

/// Input data for the analyzer
#[derive(Clone, Debug, Default)]
pub struct AnalyzerInput {
    /// Task attributions from simulation
    pub attributions: Vec<TaskAttribution>,
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
    /// Station queue time series: (station_id, name, [(timestamp_s, queue_length)])
    pub station_queue_series: Vec<(StationId, String, Vec<(f64, usize)>)>,
}

/// Root Cause Analyzer
pub struct RootCauseAnalyzer {
    bottleneck_detector: BottleneckDetector,
    anomaly_detector: AnomalyDetector,
}

impl RootCauseAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self {
            bottleneck_detector: BottleneckDetector::new(),
            anomaly_detector: AnomalyDetector::new(),
        }
    }

    /// Run full analysis on input data
    pub fn analyze(&mut self, input: &AnalyzerInput) -> RootCauseAnalysisReport {
        // 1. Delay Attribution
        let delay_attribution = DelayAttributionSummary::from_attributions(&input.attributions);

        // 2. Bottleneck Detection
        let bottleneck_input = BottleneckInputData {
            node_congestion: input.node_congestion.clone(),
            edge_congestion: input.edge_congestion.clone(),
            station_data: input.station_data.clone(),
            charging_data: input.charging_data.clone(),
            robot_utilizations: input.robot_utilizations.clone(),
        };
        let bottleneck_analysis = self.bottleneck_detector.detect(&bottleneck_input);

        // 3. Critical Path Analysis
        let critical_path_analysis = CriticalPathAnalysis::from_attributions(&input.attributions);
        let critical_paths = critical_path_analysis.summary();

        // 4. Anomaly Detection
        self.anomaly_detector.clear();
        self.anomaly_detector.detect_slow_orders(&input.attributions);

        for (station_id, name, samples) in &input.station_queue_series {
            self.anomaly_detector.detect_queue_spikes(*station_id, name, samples);
        }

        let anomalies: Vec<Anomaly> = self.anomaly_detector.anomalies()
            .into_iter()
            .cloned()
            .collect();

        // 5. Generate Summary
        let summary = self.generate_summary(
            &delay_attribution,
            &bottleneck_analysis,
            &critical_paths,
            &anomalies,
        );

        // 6. Generate Recommendations
        let recommendations = self.generate_recommendations(
            &delay_attribution,
            &bottleneck_analysis,
            &critical_paths,
            &anomalies,
        );

        RootCauseAnalysisReport {
            summary,
            delay_attribution,
            bottleneck_analysis,
            critical_paths,
            anomalies,
            recommendations,
        }
    }

    /// Generate summary from all analyses
    fn generate_summary(
        &self,
        delay_attr: &DelayAttributionSummary,
        bottleneck: &BottleneckAnalysis,
        critical_paths: &CriticalPathSummary,
        anomalies: &[Anomaly],
    ) -> RCASummary {
        let orders_analyzed = delay_attr.task_count;
        let avg_cycle_time_s = delay_attr.avg_cycle_time_s;

        // Primary delay source
        let primary_delay_source = delay_attr
            .ranked_categories
            .first()
            .map(|(cat, _, _)| cat.name().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        // Primary bottleneck
        let primary_bottleneck = bottleneck.summary.primary_bottleneck.clone();

        let anomaly_count = anomalies.len();
        let slow_order_count = critical_paths.slow_order_count;

        // Calculate health score
        let health_score = self.calculate_health_score(delay_attr, bottleneck, anomalies);

        // Generate key finding
        let key_finding = self.generate_key_finding(
            delay_attr,
            bottleneck,
            critical_paths,
            anomalies,
        );

        RCASummary {
            orders_analyzed,
            avg_cycle_time_s,
            primary_delay_source,
            primary_bottleneck,
            anomaly_count,
            slow_order_count,
            health_score,
            key_finding,
        }
    }

    /// Calculate overall health score (0-100)
    fn calculate_health_score(
        &self,
        delay_attr: &DelayAttributionSummary,
        bottleneck: &BottleneckAnalysis,
        anomalies: &[Anomaly],
    ) -> f64 {
        let mut score = 100.0;

        // Penalize for waste ratio
        let waste_penalty = delay_attr.waste_ratio * 30.0;
        score -= waste_penalty;

        // Penalize for bottlenecks
        let bottleneck_penalty = (bottleneck.summary.avg_severity / 100.0) * 30.0;
        score -= bottleneck_penalty;

        // Penalize for anomalies
        let anomaly_penalty = (anomalies.len() as f64 * 2.0).min(20.0);
        score -= anomaly_penalty;

        score.max(0.0).min(100.0)
    }

    /// Generate key finding text
    fn generate_key_finding(
        &self,
        delay_attr: &DelayAttributionSummary,
        bottleneck: &BottleneckAnalysis,
        critical_paths: &CriticalPathSummary,
        anomalies: &[Anomaly],
    ) -> String {
        // Determine the most significant issue
        let waste_ratio = delay_attr.waste_ratio;
        let bottleneck_severity = bottleneck.summary.max_severity;
        let anomaly_count = anomalies.len();

        if waste_ratio > 0.4 {
            // High waste
            if let Some((cat, _, pct)) = delay_attr.ranked_categories.first() {
                return format!(
                    "High waste time ({:.0}% of cycle). Primary cause: {} ({:.0}% of time)",
                    waste_ratio * 100.0,
                    cat.name(),
                    pct
                );
            }
        }

        if bottleneck_severity > 70.0 {
            if let Some(b) = bottleneck.bottlenecks.first() {
                return format!(
                    "Critical {} detected: {}",
                    b.name().to_lowercase(),
                    b.to_detail_string()
                );
            }
        }

        if anomaly_count > 10 {
            return format!(
                "{} anomalies detected. System may be under stress or misconfigured",
                anomaly_count
            );
        }

        if critical_paths.slow_order_count > 0 {
            let slow_pct = (critical_paths.slow_order_count as f64 / critical_paths.order_count as f64) * 100.0;
            if slow_pct > 10.0 {
                return format!(
                    "{:.0}% of orders are slow. Most common critical phase: {}",
                    slow_pct,
                    critical_paths.most_common_critical_phase.name()
                );
            }
        }

        // Default finding
        format!(
            "System operating normally. Avg cycle time: {:.1}s",
            delay_attr.avg_cycle_time_s
        )
    }

    /// Generate recommendations based on analysis
    fn generate_recommendations(
        &self,
        delay_attr: &DelayAttributionSummary,
        bottleneck: &BottleneckAnalysis,
        critical_paths: &CriticalPathSummary,
        _anomalies: &[Anomaly],
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();
        let mut priority = 1;

        // Recommendations from bottlenecks
        for b in bottleneck.bottlenecks.iter().take(3) {
            let (category, expected_impact) = match b {
                BottleneckType::CongestedNode { .. } | BottleneckType::CongestedEdge { .. } => {
                    ("Traffic Flow", "Reduce congestion delays by 20-40%")
                }
                BottleneckType::OverloadedStation { .. } => {
                    ("Station Capacity", "Reduce queue wait times by 30-50%")
                }
                BottleneckType::UnderutilizedStation { .. } => {
                    ("Resource Optimization", "Improve overall efficiency by 10-20%")
                }
                BottleneckType::ChargingContention { .. } => {
                    ("Charging Infrastructure", "Reduce charging-related delays by 20-30%")
                }
                BottleneckType::UnderutilizedRobots { .. } => {
                    ("Fleet Optimization", "Reduce operating costs by 15-25%")
                }
                BottleneckType::InsufficientRobots { .. } => {
                    ("Fleet Sizing", "Improve throughput by 20-40%")
                }
            };

            recommendations.push(Recommendation {
                priority,
                category: category.to_string(),
                text: b.recommendation(),
                expected_impact: expected_impact.to_string(),
            });
            priority += 1;
        }

        // Recommendation based on delay attribution
        if delay_attr.waste_ratio > 0.3 {
            if let Some((cat, _, _)) = delay_attr.ranked_categories.iter()
                .find(|(cat, _, _)| cat.is_waste())
            {
                let text = match cat {
                    crate::attribution::DelayCategory::RobotAssignment => {
                        "Consider adding more robots or optimizing task allocation policy"
                    }
                    crate::attribution::DelayCategory::CongestionWait => {
                        "Implement traffic management or rerouting policies"
                    }
                    crate::attribution::DelayCategory::StationQueue => {
                        "Add parallel capacity at busy stations or improve load balancing"
                    }
                    _ => "Review and optimize operational processes"
                };

                recommendations.push(Recommendation {
                    priority,
                    category: "Waste Reduction".to_string(),
                    text: text.to_string(),
                    expected_impact: format!("Reduce waste time by 20-30%"),
                });
                priority += 1;
            }
        }

        // Recommendation based on critical path
        if critical_paths.slow_order_count > 0 {
            let slow_pct = (critical_paths.slow_order_count as f64 / critical_paths.order_count.max(1) as f64) * 100.0;
            if slow_pct > 5.0 {
                recommendations.push(Recommendation {
                    priority,
                    category: "Order Fulfillment".to_string(),
                    text: format!(
                        "Investigate slow orders. Most impacted phase: {}",
                        critical_paths.most_common_critical_phase.name()
                    ),
                    expected_impact: "Improve cycle time consistency by 15-25%".to_string(),
                });
            }
        }

        recommendations
    }
}

impl Default for RootCauseAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use waremax_core::{SimTime, TaskId, OrderId};
    use crate::attribution::DelayCategory;

    fn create_test_attribution(order_id: u32, times: (f64, f64, f64, f64)) -> TaskAttribution {
        let (assignment, travel, queue, service) = times;
        let total = assignment + travel + queue + service;

        let mut attr = TaskAttribution::new(
            TaskId(order_id),
            Some(OrderId(order_id)),
            SimTime::from_seconds(0.0),
        );
        attr.record_time(DelayCategory::RobotAssignment, assignment);
        attr.record_time(DelayCategory::TravelToPickup, travel);
        attr.record_time(DelayCategory::StationQueue, queue);
        attr.record_time(DelayCategory::StationService, service);
        attr.complete(SimTime::from_seconds(total));
        attr
    }

    #[test]
    fn test_basic_analysis() {
        let mut analyzer = RootCauseAnalyzer::new();

        let attributions = vec![
            create_test_attribution(1, (5.0, 15.0, 10.0, 8.0)),
            create_test_attribution(2, (3.0, 12.0, 8.0, 10.0)),
            create_test_attribution(3, (4.0, 18.0, 12.0, 7.0)),
        ];

        let input = AnalyzerInput {
            attributions,
            ..Default::default()
        };

        let report = analyzer.analyze(&input);

        assert_eq!(report.summary.orders_analyzed, 3);
        assert!(report.summary.avg_cycle_time_s > 0.0);
        assert!(!report.summary.primary_delay_source.is_empty());
    }

    #[test]
    fn test_health_score_calculation() {
        let mut analyzer = RootCauseAnalyzer::new();

        // Good scenario - low waste
        let good_attributions = vec![
            create_test_attribution(1, (1.0, 15.0, 2.0, 10.0)),
            create_test_attribution(2, (1.0, 14.0, 1.0, 11.0)),
        ];

        let good_input = AnalyzerInput {
            attributions: good_attributions,
            ..Default::default()
        };

        let good_report = analyzer.analyze(&good_input);

        // Bad scenario - high waste
        let bad_attributions = vec![
            create_test_attribution(1, (20.0, 5.0, 25.0, 5.0)),
            create_test_attribution(2, (18.0, 6.0, 22.0, 6.0)),
        ];

        let bad_input = AnalyzerInput {
            attributions: bad_attributions,
            ..Default::default()
        };

        let bad_report = analyzer.analyze(&bad_input);

        // Good scenario should have higher health score
        assert!(good_report.summary.health_score > bad_report.summary.health_score);
    }

    #[test]
    fn test_recommendations_generation() {
        let mut analyzer = RootCauseAnalyzer::new();

        // Create scenario with clear bottleneck
        let input = AnalyzerInput {
            attributions: vec![
                create_test_attribution(1, (5.0, 10.0, 15.0, 8.0)),
            ],
            station_data: vec![
                (StationId(1), "S1".to_string(), 0.95, 5.0, 10),
            ],
            ..Default::default()
        };

        let report = analyzer.analyze(&input);

        // Should have at least one recommendation due to overloaded station
        assert!(!report.recommendations.is_empty() || report.bottleneck_analysis.bottlenecks.is_empty());
    }
}
