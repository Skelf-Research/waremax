//! Anomaly Detection
//!
//! Detects unusual patterns using statistical methods like z-scores.

use crate::attribution::TaskAttribution;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use waremax_core::{OrderId, RobotId, SimTime, StationId};

/// Types of anomalies that can be detected
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Order with unusually long cycle time
    SlowOrder {
        order_id: OrderId,
        cycle_time_s: f64,
        expected_s: f64,
        z_score: f64,
    },
    /// Station with queue spike
    StationQueueSpike {
        station_id: StationId,
        station_name: String,
        queue_length: usize,
        typical_queue: f64,
        timestamp_s: f64,
    },
    /// Robot with unusual behavior pattern
    RobotAnomaly {
        robot_id: RobotId,
        anomaly_description: String,
        metric_value: f64,
        expected_value: f64,
    },
    /// Sudden throughput drop
    ThroughputDrop {
        timestamp_s: f64,
        actual_rate: f64,
        expected_rate: f64,
        drop_pct: f64,
    },
    /// Congestion spike at a location
    CongestionSpike {
        location_description: String,
        wait_time_s: f64,
        typical_wait_s: f64,
        timestamp_s: f64,
    },
}

impl AnomalyType {
    /// Get a severity score (0-100)
    pub fn severity(&self) -> f64 {
        match self {
            AnomalyType::SlowOrder { z_score, .. } => (z_score.abs() * 20.0).min(100.0),
            AnomalyType::StationQueueSpike {
                queue_length,
                typical_queue,
                ..
            } => {
                let ratio = *queue_length as f64 / typical_queue.max(1.0);
                ((ratio - 1.0) * 30.0).min(100.0)
            }
            AnomalyType::RobotAnomaly {
                metric_value,
                expected_value,
                ..
            } => {
                let diff = (metric_value - expected_value).abs();
                let ratio = diff / expected_value.abs().max(1.0);
                (ratio * 50.0).min(100.0)
            }
            AnomalyType::ThroughputDrop { drop_pct, .. } => drop_pct.abs().min(100.0),
            AnomalyType::CongestionSpike {
                wait_time_s,
                typical_wait_s,
                ..
            } => {
                let ratio = wait_time_s / typical_wait_s.max(0.1);
                ((ratio - 1.0) * 25.0).min(100.0)
            }
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            AnomalyType::SlowOrder { .. } => "Slow Order",
            AnomalyType::StationQueueSpike { .. } => "Station Queue Spike",
            AnomalyType::RobotAnomaly { .. } => "Robot Anomaly",
            AnomalyType::ThroughputDrop { .. } => "Throughput Drop",
            AnomalyType::CongestionSpike { .. } => "Congestion Spike",
        }
    }

    /// Get detailed description
    pub fn description(&self) -> String {
        match self {
            AnomalyType::SlowOrder {
                order_id,
                cycle_time_s,
                expected_s,
                z_score,
            } => {
                format!(
                    "Order {} took {:.1}s (expected {:.1}s, z={:.2})",
                    order_id, cycle_time_s, expected_s, z_score
                )
            }
            AnomalyType::StationQueueSpike {
                station_name,
                queue_length,
                typical_queue,
                timestamp_s,
                ..
            } => {
                format!(
                    "Station {} queue spiked to {} at t={:.0}s (typical: {:.1})",
                    station_name, queue_length, timestamp_s, typical_queue
                )
            }
            AnomalyType::RobotAnomaly {
                robot_id,
                anomaly_description,
                ..
            } => {
                format!("Robot {}: {}", robot_id, anomaly_description)
            }
            AnomalyType::ThroughputDrop {
                timestamp_s,
                actual_rate,
                expected_rate,
                drop_pct,
            } => {
                format!(
                    "Throughput dropped {:.1}% at t={:.0}s ({:.1} vs {:.1} orders/hr)",
                    drop_pct, timestamp_s, actual_rate, expected_rate
                )
            }
            AnomalyType::CongestionSpike {
                location_description,
                wait_time_s,
                typical_wait_s,
                timestamp_s,
            } => {
                format!(
                    "{} wait spiked to {:.1}s at t={:.0}s (typical: {:.1}s)",
                    location_description, wait_time_s, timestamp_s, typical_wait_s
                )
            }
        }
    }
}

/// A detected anomaly with context
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Anomaly {
    /// Type of anomaly
    pub anomaly_type: AnomalyType,
    /// When the anomaly was detected (simulation time)
    pub detected_at_s: f64,
    /// Severity score (0-100)
    pub severity: f64,
}

impl Anomaly {
    /// Create a new anomaly
    pub fn new(anomaly_type: AnomalyType, detected_at: SimTime) -> Self {
        let severity = anomaly_type.severity();
        Self {
            anomaly_type,
            detected_at_s: detected_at.as_seconds(),
            severity,
        }
    }

    /// Create with explicit severity
    pub fn with_severity(anomaly_type: AnomalyType, detected_at: SimTime, severity: f64) -> Self {
        Self {
            anomaly_type,
            detected_at_s: detected_at.as_seconds(),
            severity,
        }
    }
}

/// Statistical utilities for anomaly detection
pub struct StatUtils;

impl StatUtils {
    /// Calculate mean of samples
    pub fn mean(samples: &[f64]) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }
        samples.iter().sum::<f64>() / samples.len() as f64
    }

    /// Calculate standard deviation
    pub fn std_dev(samples: &[f64]) -> f64 {
        if samples.len() < 2 {
            return 0.0;
        }
        let mean = Self::mean(samples);
        let variance =
            samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (samples.len() - 1) as f64;
        variance.sqrt()
    }

    /// Calculate z-score for a value
    pub fn z_score(value: f64, mean: f64, std_dev: f64) -> f64 {
        if std_dev <= 0.0 {
            return 0.0;
        }
        (value - mean) / std_dev
    }

    /// Calculate median
    pub fn median(samples: &[f64]) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }
        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    }

    /// Calculate percentile (0-100)
    pub fn percentile(samples: &[f64], p: f64) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }
        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let index = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    /// Detect outliers using IQR method
    pub fn iqr_outliers(samples: &[f64], multiplier: f64) -> Vec<(usize, f64)> {
        if samples.len() < 4 {
            return Vec::new();
        }

        let q1 = Self::percentile(samples, 25.0);
        let q3 = Self::percentile(samples, 75.0);
        let iqr = q3 - q1;
        let lower_bound = q1 - multiplier * iqr;
        let upper_bound = q3 + multiplier * iqr;

        samples
            .iter()
            .enumerate()
            .filter(|(_, v)| **v < lower_bound || **v > upper_bound)
            .map(|(i, v)| (i, *v))
            .collect()
    }
}

/// Configuration for anomaly detection
#[derive(Clone, Debug)]
pub struct AnomalyConfig {
    /// Z-score threshold for slow orders
    pub slow_order_z_threshold: f64,
    /// Queue spike multiplier (times typical)
    pub queue_spike_multiplier: f64,
    /// Throughput drop percentage threshold
    pub throughput_drop_threshold_pct: f64,
    /// Congestion spike multiplier
    pub congestion_spike_multiplier: f64,
    /// Minimum samples for statistical analysis
    pub min_samples: usize,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            slow_order_z_threshold: 2.0, // 2 standard deviations
            queue_spike_multiplier: 3.0, // 3x typical queue
            throughput_drop_threshold_pct: 30.0,
            congestion_spike_multiplier: 3.0,
            min_samples: 10,
        }
    }
}

/// Anomaly detector
pub struct AnomalyDetector {
    config: AnomalyConfig,
    anomalies: Vec<Anomaly>,
}

impl AnomalyDetector {
    /// Create a new detector with default config
    pub fn new() -> Self {
        Self {
            config: AnomalyConfig::default(),
            anomalies: Vec::new(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: AnomalyConfig) -> Self {
        Self {
            config,
            anomalies: Vec::new(),
        }
    }

    /// Detect slow orders from task attributions
    pub fn detect_slow_orders(&mut self, attributions: &[TaskAttribution]) {
        // Extract cycle times for completed orders
        let cycle_times: Vec<(OrderId, f64, f64)> = attributions
            .iter()
            .filter_map(|attr| {
                let order_id = attr.order_id?;
                let cycle_time = attr.cycle_time()?;
                let completed_at_s = attr.completed_at_s?;
                Some((order_id, cycle_time, completed_at_s))
            })
            .collect();

        if cycle_times.len() < self.config.min_samples {
            return;
        }

        let times: Vec<f64> = cycle_times.iter().map(|(_, t, _)| *t).collect();
        let mean = StatUtils::mean(&times);
        let std_dev = StatUtils::std_dev(&times);

        for (order_id, cycle_time, completed_at_s) in cycle_times {
            let z_score = StatUtils::z_score(cycle_time, mean, std_dev);
            if z_score > self.config.slow_order_z_threshold {
                self.anomalies.push(Anomaly::new(
                    AnomalyType::SlowOrder {
                        order_id,
                        cycle_time_s: cycle_time,
                        expected_s: mean,
                        z_score,
                    },
                    SimTime::from_seconds(completed_at_s),
                ));
            }
        }
    }

    /// Detect station queue spikes from time series data
    pub fn detect_queue_spikes(
        &mut self,
        station_id: StationId,
        station_name: &str,
        queue_samples: &[(f64, usize)], // (timestamp_s, queue_length)
    ) {
        if queue_samples.len() < self.config.min_samples {
            return;
        }

        let queues: Vec<f64> = queue_samples.iter().map(|(_, q)| *q as f64).collect();
        let typical = StatUtils::mean(&queues);
        let threshold = typical * self.config.queue_spike_multiplier;

        for (timestamp_s, queue_length) in queue_samples {
            if *queue_length as f64 > threshold && *queue_length > 2 {
                self.anomalies.push(Anomaly::new(
                    AnomalyType::StationQueueSpike {
                        station_id,
                        station_name: station_name.to_string(),
                        queue_length: *queue_length,
                        typical_queue: typical,
                        timestamp_s: *timestamp_s,
                    },
                    SimTime::from_seconds(*timestamp_s),
                ));
            }
        }
    }

    /// Detect throughput drops from time series data
    pub fn detect_throughput_drops(
        &mut self,
        throughput_samples: &[(f64, f64)], // (timestamp_s, orders_per_hour)
    ) {
        if throughput_samples.len() < self.config.min_samples {
            return;
        }

        let rates: Vec<f64> = throughput_samples.iter().map(|(_, r)| *r).collect();
        let baseline = StatUtils::percentile(&rates, 75.0); // Use 75th percentile as baseline

        for (timestamp_s, rate) in throughput_samples {
            if baseline > 0.0 && *rate < baseline {
                let drop_pct = ((baseline - rate) / baseline) * 100.0;
                if drop_pct > self.config.throughput_drop_threshold_pct {
                    self.anomalies.push(Anomaly::new(
                        AnomalyType::ThroughputDrop {
                            timestamp_s: *timestamp_s,
                            actual_rate: *rate,
                            expected_rate: baseline,
                            drop_pct,
                        },
                        SimTime::from_seconds(*timestamp_s),
                    ));
                }
            }
        }
    }

    /// Add a custom anomaly
    pub fn add_anomaly(&mut self, anomaly: Anomaly) {
        self.anomalies.push(anomaly);
    }

    /// Get all detected anomalies sorted by severity
    pub fn anomalies(&self) -> Vec<&Anomaly> {
        let mut sorted: Vec<_> = self.anomalies.iter().collect();
        sorted.sort_by(|a, b| {
            b.severity
                .partial_cmp(&a.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }

    /// Get anomalies filtered by minimum severity
    pub fn anomalies_above_severity(&self, min_severity: f64) -> Vec<&Anomaly> {
        self.anomalies
            .iter()
            .filter(|a| a.severity >= min_severity)
            .collect()
    }

    /// Get count of anomalies by type
    pub fn count_by_type(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for anomaly in &self.anomalies {
            *counts
                .entry(anomaly.anomaly_type.name().to_string())
                .or_insert(0) += 1;
        }
        counts
    }

    /// Clear all detected anomalies
    pub fn clear(&mut self) {
        self.anomalies.clear();
    }

    /// Total number of anomalies
    pub fn count(&self) -> usize {
        self.anomalies.len()
    }

    /// Generate summary report
    pub fn summary_report(&self) -> String {
        let mut output = String::new();
        output.push_str("Anomaly Detection Summary\n");
        output.push_str(&"=".repeat(50));
        output.push('\n');

        output.push_str(&format!(
            "Total Anomalies Detected: {}\n",
            self.anomalies.len()
        ));

        let counts = self.count_by_type();
        if !counts.is_empty() {
            output.push_str("\nBy Type:\n");
            for (name, count) in &counts {
                output.push_str(&format!("  {}: {}\n", name, count));
            }
        }

        if !self.anomalies.is_empty() {
            output.push_str("\nTop Anomalies:\n");
            for (i, anomaly) in self.anomalies().iter().take(10).enumerate() {
                output.push_str(&format!(
                    "{}. [severity: {:.1}] {}\n",
                    i + 1,
                    anomaly.severity,
                    anomaly.anomaly_type.description()
                ));
            }
        }

        output
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribution::DelayCategory;
    use waremax_core::TaskId;

    #[test]
    fn test_stat_utils_mean() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((StatUtils::mean(&samples) - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_stat_utils_std_dev() {
        let samples = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let std_dev = StatUtils::std_dev(&samples);
        assert!(std_dev > 0.0);
    }

    #[test]
    fn test_stat_utils_z_score() {
        let z = StatUtils::z_score(15.0, 10.0, 2.5);
        assert!((z - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_stat_utils_median() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((StatUtils::median(&samples) - 3.0).abs() < 0.001);

        let even = vec![1.0, 2.0, 3.0, 4.0];
        assert!((StatUtils::median(&even) - 2.5).abs() < 0.001);
    }

    #[test]
    fn test_detect_slow_orders() {
        fn create_attribution(order_id: u32, cycle_time: f64) -> TaskAttribution {
            let mut attr = TaskAttribution::new(
                TaskId(order_id),
                Some(OrderId(order_id)),
                SimTime::from_seconds(0.0),
            );
            attr.record_time(DelayCategory::TravelToPickup, cycle_time * 0.4);
            attr.record_time(DelayCategory::StationService, cycle_time * 0.6);
            attr.complete(SimTime::from_seconds(cycle_time));
            attr
        }

        let mut attributions = Vec::new();
        // Normal orders around 30s
        for i in 0..20 {
            attributions.push(create_attribution(i, 28.0 + (i as f64 % 5.0)));
        }
        // Add slow outliers
        attributions.push(create_attribution(100, 80.0));
        attributions.push(create_attribution(101, 90.0));

        let mut detector = AnomalyDetector::new();
        detector.detect_slow_orders(&attributions);

        // Should detect the slow orders
        let anomalies = detector.anomalies();
        assert!(anomalies.len() >= 1);

        // Check that slow orders were flagged
        let slow_orders: Vec<_> = anomalies
            .iter()
            .filter_map(|a| match &a.anomaly_type {
                AnomalyType::SlowOrder { order_id, .. } => Some(order_id.0),
                _ => None,
            })
            .collect();
        assert!(slow_orders.contains(&100) || slow_orders.contains(&101));
    }

    #[test]
    fn test_detect_queue_spikes() {
        let mut detector = AnomalyDetector::new();

        // Normal queue around 2
        let mut samples: Vec<(f64, usize)> = (0..20).map(|i| (i as f64 * 60.0, 2)).collect();
        // Add spikes
        samples.push((1200.0, 10));
        samples.push((1260.0, 12));

        detector.detect_queue_spikes(StationId(1), "S1", &samples);

        let anomalies = detector.anomalies();
        assert!(anomalies.len() >= 1);

        // Verify queue spike detected
        assert!(anomalies
            .iter()
            .any(|a| matches!(a.anomaly_type, AnomalyType::StationQueueSpike { .. })));
    }

    #[test]
    fn test_anomaly_severity() {
        let slow = AnomalyType::SlowOrder {
            order_id: OrderId(1),
            cycle_time_s: 100.0,
            expected_s: 30.0,
            z_score: 3.5,
        };
        assert!(slow.severity() > 50.0);

        let queue = AnomalyType::StationQueueSpike {
            station_id: StationId(1),
            station_name: "S1".to_string(),
            queue_length: 15,
            typical_queue: 3.0,
            timestamp_s: 1000.0,
        };
        assert!(queue.severity() > 0.0);
    }
}
