//! Critical Path Analysis
//!
//! Identifies the slowest phases in order processing and compares
//! actual times against expected/baseline times.

use crate::attribution::{DelayCategory, TaskAttribution};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use waremax_core::OrderId;

/// Phase in order processing for critical path analysis
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderPhase {
    /// Waiting for task creation/robot assignment
    Assignment,
    /// Robot traveling to pickup
    TravelToPickup,
    /// Robot traveling to station
    TravelToStation,
    /// Waiting in queue at station
    QueueWait,
    /// Being serviced at station
    Service,
    /// Any detour (charging, maintenance)
    Detour,
}

impl OrderPhase {
    /// Map delay categories to order phases
    pub fn from_delay_category(cat: &DelayCategory) -> Self {
        match cat {
            DelayCategory::RobotAssignment => OrderPhase::Assignment,
            DelayCategory::TravelToPickup => OrderPhase::TravelToPickup,
            DelayCategory::TravelToStation => OrderPhase::TravelToStation,
            DelayCategory::CongestionWait => OrderPhase::TravelToStation, // Count as travel
            DelayCategory::StationQueue => OrderPhase::QueueWait,
            DelayCategory::StationService => OrderPhase::Service,
            DelayCategory::ChargingDetour
            | DelayCategory::MaintenanceDetour
            | DelayCategory::FailureRecovery => OrderPhase::Detour,
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            OrderPhase::Assignment => "Assignment",
            OrderPhase::TravelToPickup => "Travel to Pickup",
            OrderPhase::TravelToStation => "Travel to Station",
            OrderPhase::QueueWait => "Queue Wait",
            OrderPhase::Service => "Service",
            OrderPhase::Detour => "Detour",
        }
    }

    /// All phases in typical order of execution
    pub fn all() -> Vec<OrderPhase> {
        vec![
            OrderPhase::Assignment,
            OrderPhase::TravelToPickup,
            OrderPhase::TravelToStation,
            OrderPhase::QueueWait,
            OrderPhase::Service,
            OrderPhase::Detour,
        ]
    }
}

/// Critical path data for a single order
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderCriticalPath {
    /// Order ID
    pub order_id: OrderId,
    /// Time breakdown by phase (seconds)
    pub phase_times: HashMap<OrderPhase, f64>,
    /// Total cycle time
    pub total_time_s: f64,
    /// Expected cycle time (if baseline provided)
    pub expected_time_s: Option<f64>,
    /// The slowest phase (critical phase)
    pub critical_phase: OrderPhase,
    /// Time spent on critical phase
    pub critical_phase_time_s: f64,
    /// Percentage of total time on critical phase
    pub critical_phase_pct: f64,
}

impl OrderCriticalPath {
    /// Create from task attribution
    pub fn from_attribution(attribution: &TaskAttribution) -> Option<Self> {
        let order_id = attribution.order_id?;
        let total_time_s = attribution.cycle_time()?;

        // Aggregate by phase
        let mut phase_times: HashMap<OrderPhase, f64> = HashMap::new();
        for (cat, time) in &attribution.time_breakdown {
            let phase = OrderPhase::from_delay_category(cat);
            *phase_times.entry(phase).or_insert(0.0) += time;
        }

        // Find critical phase
        let (critical_phase, critical_phase_time_s) = phase_times
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(p, t)| (p.clone(), *t))
            .unwrap_or((OrderPhase::Assignment, 0.0));

        let critical_phase_pct = if total_time_s > 0.0 {
            (critical_phase_time_s / total_time_s) * 100.0
        } else {
            0.0
        };

        Some(Self {
            order_id,
            phase_times,
            total_time_s,
            expected_time_s: None,
            critical_phase,
            critical_phase_time_s,
            critical_phase_pct,
        })
    }

    /// Set expected time for comparison
    pub fn with_expected_time(mut self, expected_s: f64) -> Self {
        self.expected_time_s = Some(expected_s);
        self
    }

    /// Get deviation from expected time
    pub fn deviation_s(&self) -> Option<f64> {
        self.expected_time_s.map(|exp| self.total_time_s - exp)
    }

    /// Get deviation as percentage
    pub fn deviation_pct(&self) -> Option<f64> {
        self.expected_time_s.map(|exp| {
            if exp > 0.0 {
                ((self.total_time_s - exp) / exp) * 100.0
            } else {
                0.0
            }
        })
    }

    /// Check if this order is slower than expected
    pub fn is_slow(&self, threshold_pct: f64) -> bool {
        self.deviation_pct()
            .map(|d| d > threshold_pct)
            .unwrap_or(false)
    }
}

/// Summary of critical path analysis across all orders
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CriticalPathSummary {
    /// Number of orders analyzed
    pub order_count: usize,
    /// Average time by phase
    pub avg_phase_times: HashMap<OrderPhase, f64>,
    /// Total time by phase (across all orders)
    pub total_phase_times: HashMap<OrderPhase, f64>,
    /// Percentage of total time by phase
    pub phase_percentages: HashMap<OrderPhase, f64>,
    /// Most common critical phase
    pub most_common_critical_phase: OrderPhase,
    /// Frequency of each phase being critical
    pub critical_phase_frequency: HashMap<OrderPhase, usize>,
    /// Average cycle time
    pub avg_cycle_time_s: f64,
    /// Average deviation from expected (if available)
    pub avg_deviation_s: Option<f64>,
    /// Number of slow orders
    pub slow_order_count: usize,
    /// Phases ranked by total time (descending)
    pub phases_ranked: Vec<(OrderPhase, f64, f64)>, // (phase, total_s, pct)
}

impl Default for CriticalPathSummary {
    fn default() -> Self {
        Self {
            order_count: 0,
            avg_phase_times: HashMap::new(),
            total_phase_times: HashMap::new(),
            phase_percentages: HashMap::new(),
            most_common_critical_phase: OrderPhase::Assignment,
            critical_phase_frequency: HashMap::new(),
            avg_cycle_time_s: 0.0,
            avg_deviation_s: None,
            slow_order_count: 0,
            phases_ranked: Vec::new(),
        }
    }
}

impl CriticalPathSummary {
    /// Format as a human-readable string
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        let mut output = String::new();
        output.push_str("Critical Path Summary\n");
        output.push_str(&"=".repeat(50));
        output.push('\n');

        output.push_str(&format!("Orders Analyzed: {}\n", self.order_count));
        output.push_str(&format!(
            "Average Cycle Time: {:.1}s\n",
            self.avg_cycle_time_s
        ));

        if let Some(dev) = self.avg_deviation_s {
            output.push_str(&format!("Average Deviation: {:.1}s\n", dev));
        }

        output.push_str(&format!("Slow Orders: {}\n", self.slow_order_count));
        output.push_str(&format!(
            "\nMost Common Critical Phase: {}\n",
            self.most_common_critical_phase.name()
        ));

        output.push_str("\nTime Breakdown by Phase:\n");
        for (phase, total_s, pct) in &self.phases_ranked {
            let avg = self.avg_phase_times.get(phase).unwrap_or(&0.0);
            output.push_str(&format!(
                "  {}: {:.1}% ({:.1}s total, {:.1}s avg)\n",
                phase.name(),
                pct,
                total_s,
                avg
            ));
        }

        output.push_str("\nCritical Phase Frequency:\n");
        let mut freq: Vec<_> = self.critical_phase_frequency.iter().collect();
        freq.sort_by(|a, b| b.1.cmp(a.1));
        for (phase, count) in freq {
            let pct = (*count as f64 / self.order_count as f64) * 100.0;
            output.push_str(&format!(
                "  {}: {} orders ({:.1}%)\n",
                phase.name(),
                count,
                pct
            ));
        }

        output
    }
}

/// Critical path analyzer
pub struct CriticalPathAnalysis {
    /// Individual order paths
    paths: Vec<OrderCriticalPath>,
    /// Expected baseline times by some criteria (optional)
    baseline_cycle_time: Option<f64>,
    /// Threshold for slow order detection (percentage above expected)
    slow_threshold_pct: f64,
}

impl CriticalPathAnalysis {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            baseline_cycle_time: None,
            slow_threshold_pct: 50.0, // 50% above expected
        }
    }

    /// Set baseline expected cycle time
    pub fn with_baseline(mut self, baseline_s: f64) -> Self {
        self.baseline_cycle_time = Some(baseline_s);
        self
    }

    /// Set slow order threshold
    pub fn with_slow_threshold(mut self, threshold_pct: f64) -> Self {
        self.slow_threshold_pct = threshold_pct;
        self
    }

    /// Analyze from task attributions
    pub fn from_attributions(attributions: &[TaskAttribution]) -> Self {
        let mut analyzer = Self::new();

        for attr in attributions {
            if let Some(path) = OrderCriticalPath::from_attribution(attr) {
                // Will set baseline later if available
                analyzer.paths.push(path);
            }
        }

        // Calculate baseline as median cycle time if not set
        if !analyzer.paths.is_empty() {
            let mut times: Vec<f64> = analyzer.paths.iter().map(|p| p.total_time_s).collect();
            times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let median = times[times.len() / 2];
            analyzer.baseline_cycle_time = Some(median);

            // Set expected time on all paths
            for path in &mut analyzer.paths {
                path.expected_time_s = Some(median);
            }
        }

        analyzer
    }

    /// Get all order paths
    pub fn paths(&self) -> &[OrderCriticalPath] {
        &self.paths
    }

    /// Get slow orders
    pub fn slow_orders(&self) -> Vec<&OrderCriticalPath> {
        self.paths
            .iter()
            .filter(|p| p.is_slow(self.slow_threshold_pct))
            .collect()
    }

    /// Generate summary
    pub fn summary(&self) -> CriticalPathSummary {
        if self.paths.is_empty() {
            return CriticalPathSummary::default();
        }

        let order_count = self.paths.len();

        // Aggregate phase times
        let mut total_phase_times: HashMap<OrderPhase, f64> = HashMap::new();
        let mut critical_phase_frequency: HashMap<OrderPhase, usize> = HashMap::new();
        let mut total_cycle_time = 0.0;
        let mut total_deviation = 0.0;
        let mut deviation_count = 0;
        let mut slow_order_count = 0;

        for path in &self.paths {
            total_cycle_time += path.total_time_s;

            for (phase, time) in &path.phase_times {
                *total_phase_times.entry(phase.clone()).or_insert(0.0) += time;
            }

            *critical_phase_frequency
                .entry(path.critical_phase.clone())
                .or_insert(0) += 1;

            if let Some(dev) = path.deviation_s() {
                total_deviation += dev;
                deviation_count += 1;
            }

            if path.is_slow(self.slow_threshold_pct) {
                slow_order_count += 1;
            }
        }

        // Calculate averages and percentages
        let avg_cycle_time_s = total_cycle_time / order_count as f64;
        let grand_total: f64 = total_phase_times.values().sum();

        let avg_phase_times: HashMap<OrderPhase, f64> = total_phase_times
            .iter()
            .map(|(p, t)| (p.clone(), t / order_count as f64))
            .collect();

        let phase_percentages: HashMap<OrderPhase, f64> = if grand_total > 0.0 {
            total_phase_times
                .iter()
                .map(|(p, t)| (p.clone(), (t / grand_total) * 100.0))
                .collect()
        } else {
            HashMap::new()
        };

        // Find most common critical phase
        let most_common_critical_phase = critical_phase_frequency
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(phase, _)| phase.clone())
            .unwrap_or(OrderPhase::Assignment);

        let avg_deviation_s = if deviation_count > 0 {
            Some(total_deviation / deviation_count as f64)
        } else {
            None
        };

        // Rank phases by total time
        let mut phases_ranked: Vec<(OrderPhase, f64, f64)> = total_phase_times
            .iter()
            .map(|(p, t)| {
                let pct = if grand_total > 0.0 {
                    (t / grand_total) * 100.0
                } else {
                    0.0
                };
                (p.clone(), *t, pct)
            })
            .collect();
        phases_ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        CriticalPathSummary {
            order_count,
            avg_phase_times,
            total_phase_times,
            phase_percentages,
            most_common_critical_phase,
            critical_phase_frequency,
            avg_cycle_time_s,
            avg_deviation_s,
            slow_order_count,
            phases_ranked,
        }
    }
}

impl Default for CriticalPathAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use waremax_core::{SimTime, TaskId};

    fn create_test_attribution(
        task_id: u32,
        order_id: u32,
        assignment: f64,
        travel: f64,
        queue: f64,
        service: f64,
    ) -> TaskAttribution {
        let mut attr = TaskAttribution::new(
            TaskId(task_id),
            Some(OrderId(order_id)),
            SimTime::from_seconds(0.0),
        );
        attr.record_time(DelayCategory::RobotAssignment, assignment);
        attr.record_time(DelayCategory::TravelToPickup, travel);
        attr.record_time(DelayCategory::StationQueue, queue);
        attr.record_time(DelayCategory::StationService, service);
        attr.complete(SimTime::from_seconds(assignment + travel + queue + service));
        attr
    }

    #[test]
    fn test_order_critical_path() {
        let attr = create_test_attribution(1, 100, 5.0, 15.0, 10.0, 8.0);
        let path = OrderCriticalPath::from_attribution(&attr).unwrap();

        assert_eq!(path.order_id, OrderId(100));
        assert!((path.total_time_s - 38.0).abs() < 0.01);
        // Travel should be critical (15s is largest)
        assert_eq!(path.critical_phase, OrderPhase::TravelToPickup);
    }

    #[test]
    fn test_critical_path_analysis() {
        let attributions = vec![
            create_test_attribution(1, 100, 5.0, 15.0, 10.0, 8.0), // 38s, travel critical
            create_test_attribution(2, 101, 3.0, 10.0, 20.0, 8.0), // 41s, queue critical
            create_test_attribution(3, 102, 8.0, 12.0, 5.0, 10.0), // 35s, travel critical
        ];

        let analysis = CriticalPathAnalysis::from_attributions(&attributions);
        let summary = analysis.summary();

        assert_eq!(summary.order_count, 3);
        // Either TravelToPickup or QueueWait should be most common
        // In this case, travel is critical for 2 orders
        assert!(
            summary
                .critical_phase_frequency
                .get(&OrderPhase::TravelToPickup)
                .unwrap_or(&0)
                >= &1
        );
    }

    #[test]
    fn test_slow_order_detection() {
        let attributions = vec![
            create_test_attribution(1, 100, 5.0, 10.0, 5.0, 8.0), // 28s - baseline
            create_test_attribution(2, 101, 5.0, 10.0, 5.0, 8.0), // 28s
            create_test_attribution(3, 102, 10.0, 30.0, 20.0, 15.0), // 75s - slow
        ];

        let analysis =
            CriticalPathAnalysis::from_attributions(&attributions).with_slow_threshold(50.0);

        // Median is 28s, so 75s is >50% above baseline
        let slow = analysis.slow_orders();
        assert_eq!(slow.len(), 1);
        assert_eq!(slow[0].order_id, OrderId(102));
    }
}
