//! Statistical comparison utilities
//!
//! Provides AggregatedStats for summarizing multiple runs and
//! ScenarioComparator for comparing different configurations.

use serde::{Deserialize, Serialize};
use statrs::statistics::{Data, Distribution, Max, Min, OrderStatistics};
use std::collections::HashMap;

use crate::runner::RunResult;

/// Aggregated statistics across multiple runs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregatedStats {
    /// Arithmetic mean
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Median (50th percentile)
    pub p50: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
    /// Number of samples
    pub n: usize,
    /// 95% confidence interval lower bound
    pub ci_95_lower: f64,
    /// 95% confidence interval upper bound
    pub ci_95_upper: f64,
}

impl AggregatedStats {
    /// Create aggregated statistics from a slice of samples
    pub fn from_samples(samples: &[f64]) -> Self {
        if samples.is_empty() {
            return Self {
                mean: 0.0,
                std_dev: 0.0,
                min: 0.0,
                max: 0.0,
                p50: 0.0,
                p95: 0.0,
                p99: 0.0,
                n: 0,
                ci_95_lower: 0.0,
                ci_95_upper: 0.0,
            };
        }

        let mut data = Data::new(samples.to_vec());
        let n = samples.len();

        let mean = data.mean().unwrap_or(0.0);
        let std_dev = data.std_dev().unwrap_or(0.0);
        let min = data.min();
        let max = data.max();
        let p50 = data.median();
        let p95 = data.percentile(95);
        let p99 = data.percentile(99);

        // Calculate 95% confidence interval
        // CI = mean ± t * (std_dev / sqrt(n))
        // Using t ≈ 1.96 for large n (normal approximation)
        let t_value = if n >= 30 { 1.96 } else { 2.0 }; // Simplified
        let margin = t_value * std_dev / (n as f64).sqrt();
        let ci_95_lower = mean - margin;
        let ci_95_upper = mean + margin;

        Self {
            mean,
            std_dev,
            min,
            max,
            p50,
            p95,
            p99,
            n,
            ci_95_lower,
            ci_95_upper,
        }
    }

    /// Check if the confidence interval overlaps with another
    pub fn overlaps(&self, other: &AggregatedStats) -> bool {
        !(self.ci_95_upper < other.ci_95_lower || other.ci_95_upper < self.ci_95_lower)
    }

    /// Calculate coefficient of variation (CV = std_dev / mean)
    pub fn coefficient_of_variation(&self) -> f64 {
        if self.mean.abs() < f64::EPSILON {
            0.0
        } else {
            self.std_dev / self.mean.abs()
        }
    }
}

/// Comparison of a specific metric across configurations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricComparison {
    /// Name of the metric being compared
    pub metric_name: String,
    /// Statistics for baseline configuration
    pub baseline: AggregatedStats,
    /// Statistics for variant configuration
    pub variant: AggregatedStats,
    /// Absolute difference (variant - baseline)
    pub absolute_diff: f64,
    /// Relative difference as percentage ((variant - baseline) / baseline * 100)
    pub relative_diff_pct: f64,
    /// Whether the change is an improvement (depends on metric)
    pub is_improvement: bool,
    /// Whether the difference is statistically significant
    pub statistically_significant: bool,
    /// P-value from statistical test
    pub p_value: f64,
}

impl MetricComparison {
    /// Create a comparison between baseline and variant samples
    pub fn compare(
        metric_name: &str,
        baseline_samples: &[f64],
        variant_samples: &[f64],
        higher_is_better: bool,
    ) -> Self {
        let baseline = AggregatedStats::from_samples(baseline_samples);
        let variant = AggregatedStats::from_samples(variant_samples);

        let absolute_diff = variant.mean - baseline.mean;
        let relative_diff_pct = if baseline.mean.abs() > f64::EPSILON {
            (absolute_diff / baseline.mean) * 100.0
        } else {
            0.0
        };

        // Perform Welch's t-test
        let (_, p_value) = crate::ab_testing::welchs_t_test(baseline_samples, variant_samples);
        let statistically_significant = p_value < 0.05;

        // Determine if change is an improvement
        let is_improvement = if higher_is_better {
            absolute_diff > 0.0 && statistically_significant
        } else {
            absolute_diff < 0.0 && statistically_significant
        };

        Self {
            metric_name: metric_name.to_string(),
            baseline,
            variant,
            absolute_diff,
            relative_diff_pct,
            is_improvement,
            statistically_significant,
            p_value,
        }
    }

    /// Format the comparison as a human-readable string
    pub fn summary(&self) -> String {
        let direction = if self.absolute_diff > 0.0 { "+" } else { "" };
        let sig_marker = if self.statistically_significant {
            "*"
        } else {
            ""
        };
        let improvement_marker = if self.is_improvement {
            " (better)"
        } else if self.statistically_significant && !self.is_improvement {
            " (worse)"
        } else {
            ""
        };

        format!(
            "{}: {:.2} → {:.2} ({}{:.2}, {}{:.1}%){}{} [p={:.3}]",
            self.metric_name,
            self.baseline.mean,
            self.variant.mean,
            direction,
            self.absolute_diff,
            direction,
            self.relative_diff_pct,
            sig_marker,
            improvement_marker,
            self.p_value
        )
    }
}

/// Full comparison report between two configurations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComparisonReport {
    /// Label for baseline configuration
    pub baseline_label: String,
    /// Label for variant configuration
    pub variant_label: String,
    /// Comparisons for each metric
    pub metrics: Vec<MetricComparison>,
    /// Summary text
    pub summary: String,
}

impl ComparisonReport {
    /// Format the report as a string
    pub fn to_string(&self) -> String {
        let mut output = format!(
            "Comparison: {} vs {}\n",
            self.baseline_label, self.variant_label
        );
        output.push_str(&"=".repeat(60));
        output.push('\n');

        for comparison in &self.metrics {
            output.push_str(&comparison.summary());
            output.push('\n');
        }

        output.push_str(&"-".repeat(60));
        output.push('\n');
        output.push_str(&self.summary);

        output
    }

    /// Get metrics that show improvement
    pub fn improvements(&self) -> Vec<&MetricComparison> {
        self.metrics.iter().filter(|m| m.is_improvement).collect()
    }

    /// Get metrics that show degradation
    pub fn degradations(&self) -> Vec<&MetricComparison> {
        self.metrics
            .iter()
            .filter(|m| m.statistically_significant && !m.is_improvement)
            .collect()
    }
}

/// Compare multiple scenario results
pub struct ScenarioComparator {
    results: HashMap<String, Vec<RunResult>>,
}

impl ScenarioComparator {
    /// Create a new comparator
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    /// Add results for a labeled configuration
    pub fn add_results(&mut self, label: &str, results: Vec<RunResult>) {
        self.results.insert(label.to_string(), results);
    }

    /// Compare variant against baseline
    pub fn compare(&self, baseline_label: &str, variant_label: &str) -> Option<ComparisonReport> {
        let baseline_results = self.results.get(baseline_label)?;
        let variant_results = self.results.get(variant_label)?;

        // Extract metrics from results
        let baseline_throughput: Vec<f64> =
            baseline_results.iter().map(|r| r.throughput()).collect();
        let variant_throughput: Vec<f64> = variant_results.iter().map(|r| r.throughput()).collect();

        let baseline_p95: Vec<f64> = baseline_results
            .iter()
            .map(|r| r.p95_cycle_time())
            .collect();
        let variant_p95: Vec<f64> = variant_results.iter().map(|r| r.p95_cycle_time()).collect();

        let baseline_robot_util: Vec<f64> = baseline_results
            .iter()
            .map(|r| r.robot_utilization())
            .collect();
        let variant_robot_util: Vec<f64> = variant_results
            .iter()
            .map(|r| r.robot_utilization())
            .collect();

        let baseline_station_util: Vec<f64> = baseline_results
            .iter()
            .map(|r| r.station_utilization())
            .collect();
        let variant_station_util: Vec<f64> = variant_results
            .iter()
            .map(|r| r.station_utilization())
            .collect();

        // Create comparisons
        let mut metrics = Vec::new();

        metrics.push(MetricComparison::compare(
            "Throughput (orders/hr)",
            &baseline_throughput,
            &variant_throughput,
            true, // Higher is better
        ));

        metrics.push(MetricComparison::compare(
            "P95 Cycle Time (s)",
            &baseline_p95,
            &variant_p95,
            false, // Lower is better
        ));

        metrics.push(MetricComparison::compare(
            "Robot Utilization",
            &baseline_robot_util,
            &variant_robot_util,
            true, // Higher is better (up to a point)
        ));

        metrics.push(MetricComparison::compare(
            "Station Utilization",
            &baseline_station_util,
            &variant_station_util,
            true, // Higher is better (up to a point)
        ));

        // Generate summary
        let improvements: Vec<_> = metrics
            .iter()
            .filter(|m| m.is_improvement)
            .map(|m| m.metric_name.clone())
            .collect();

        let degradations: Vec<_> = metrics
            .iter()
            .filter(|m| {
                m.statistically_significant
                    && !m.is_improvement
                    && m.absolute_diff.abs() > f64::EPSILON
            })
            .map(|m| m.metric_name.clone())
            .collect();

        let summary = if !improvements.is_empty() && degradations.is_empty() {
            format!("Variant shows improvements in: {}", improvements.join(", "))
        } else if improvements.is_empty() && !degradations.is_empty() {
            format!("Variant shows degradation in: {}", degradations.join(", "))
        } else if !improvements.is_empty() && !degradations.is_empty() {
            format!(
                "Mixed results. Better: {}. Worse: {}",
                improvements.join(", "),
                degradations.join(", ")
            )
        } else {
            "No statistically significant differences detected".to_string()
        };

        Some(ComparisonReport {
            baseline_label: baseline_label.to_string(),
            variant_label: variant_label.to_string(),
            metrics,
            summary,
        })
    }

    /// Compare all variants against a baseline
    pub fn compare_all_to_baseline(&self, baseline_label: &str) -> Vec<ComparisonReport> {
        self.results
            .keys()
            .filter(|k| *k != baseline_label)
            .filter_map(|variant_label| self.compare(baseline_label, variant_label))
            .collect()
    }

    /// Generate ranking table for a specific metric
    pub fn rank_by_metric(
        &self,
        extractor: fn(&RunResult) -> f64,
        higher_is_better: bool,
    ) -> Vec<(String, AggregatedStats)> {
        let mut rankings: Vec<(String, AggregatedStats)> = self
            .results
            .iter()
            .map(|(label, results)| {
                let samples: Vec<f64> = results.iter().map(|r| extractor(r)).collect();
                (label.clone(), AggregatedStats::from_samples(&samples))
            })
            .collect();

        rankings.sort_by(|a, b| {
            if higher_is_better {
                b.1.mean
                    .partial_cmp(&a.1.mean)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                a.1.mean
                    .partial_cmp(&b.1.mean)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        rankings
    }

    /// Rank by throughput (higher is better)
    pub fn rank_by_throughput(&self) -> Vec<(String, AggregatedStats)> {
        self.rank_by_metric(|r| r.throughput(), true)
    }

    /// Rank by P95 cycle time (lower is better)
    pub fn rank_by_latency(&self) -> Vec<(String, AggregatedStats)> {
        self.rank_by_metric(|r| r.p95_cycle_time(), false)
    }

    /// Get the label of the best configuration by throughput
    pub fn best_by_throughput(&self) -> Option<String> {
        self.rank_by_throughput()
            .first()
            .map(|(label, _)| label.clone())
    }

    /// Get all configuration labels
    pub fn labels(&self) -> Vec<&String> {
        self.results.keys().collect()
    }
}

impl Default for ScenarioComparator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregated_stats_basic() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = AggregatedStats::from_samples(&samples);

        assert_eq!(stats.n, 5);
        assert!((stats.mean - 3.0).abs() < f64::EPSILON);
        assert!((stats.min - 1.0).abs() < f64::EPSILON);
        assert!((stats.max - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_aggregated_stats_empty() {
        let samples: Vec<f64> = vec![];
        let stats = AggregatedStats::from_samples(&samples);

        assert_eq!(stats.n, 0);
        assert_eq!(stats.mean, 0.0);
    }

    #[test]
    fn test_metric_comparison() {
        let baseline = vec![10.0, 11.0, 10.5, 10.2, 10.8];
        let variant = vec![12.0, 12.5, 11.8, 12.2, 12.1];

        let comparison = MetricComparison::compare("Test", &baseline, &variant, true);

        assert!(comparison.absolute_diff > 0.0);
        assert!(comparison.relative_diff_pct > 0.0);
    }

    #[test]
    fn test_coefficient_of_variation() {
        let samples = vec![100.0, 100.0, 100.0, 100.0];
        let stats = AggregatedStats::from_samples(&samples);
        assert!(stats.coefficient_of_variation().abs() < 0.001);

        let varied = vec![50.0, 100.0, 150.0, 200.0];
        let stats2 = AggregatedStats::from_samples(&varied);
        assert!(stats2.coefficient_of_variation() > 0.0);
    }
}
