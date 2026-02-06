//! A/B testing framework with statistical significance tests
//!
//! Provides ABTestRunner for comparing two configurations with
//! Welch's t-test for statistical significance.

use serde::{Deserialize, Serialize};
use statrs::distribution::{ContinuousCDF, StudentsT};
use std::collections::HashMap;

use crate::comparison::AggregatedStats;
use crate::runner::{BatchRunner, RunResult};
use waremax_config::ScenarioConfig;

/// Configuration for an A/B test
#[derive(Clone)]
pub struct ABTestConfig {
    /// Baseline scenario configuration
    pub baseline_config: ScenarioConfig,
    /// Variant scenario configuration
    pub variant_config: ScenarioConfig,
    /// Number of replications per variant
    pub replications: u32,
    /// Significance level (default: 0.05)
    pub alpha: f64,
    /// Minimum effect size to detect (optional)
    pub minimum_effect_size: Option<f64>,
}

impl ABTestConfig {
    /// Create a new A/B test configuration
    pub fn new(baseline: ScenarioConfig, variant: ScenarioConfig) -> Self {
        Self {
            baseline_config: baseline,
            variant_config: variant,
            replications: 10,
            alpha: 0.05,
            minimum_effect_size: None,
        }
    }

    /// Set number of replications
    pub fn replications(mut self, n: u32) -> Self {
        self.replications = n;
        self
    }

    /// Set significance level (alpha)
    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = alpha;
        self
    }

    /// Set minimum effect size for power analysis
    pub fn minimum_effect_size(mut self, effect: f64) -> Self {
        self.minimum_effect_size = Some(effect);
        self
    }
}

/// Metric comparison in an A/B test
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ABMetricComparison {
    /// Name of the metric
    pub metric_name: String,
    /// Baseline statistics
    pub baseline_stats: AggregatedStats,
    /// Variant statistics
    pub variant_stats: AggregatedStats,
    /// Absolute difference (variant - baseline)
    pub absolute_diff: f64,
    /// Relative difference as percentage
    pub relative_diff_pct: f64,
    /// T-statistic from Welch's t-test
    pub t_statistic: f64,
    /// P-value from the test
    pub p_value: f64,
    /// Whether the difference is statistically significant
    pub is_significant: bool,
    /// Whether higher values are better for this metric
    pub higher_is_better: bool,
}

/// Conclusion of an A/B test
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ABConclusion {
    /// Variant is better on specified metrics
    VariantBetter { metrics: Vec<String> },
    /// Baseline is better on specified metrics
    BaselineBetter { metrics: Vec<String> },
    /// No statistically significant difference detected
    NoSignificantDifference,
    /// Mixed results - some metrics better, some worse
    Mixed {
        better: Vec<String>,
        worse: Vec<String>,
    },
}

/// Result of an A/B test
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ABTestResult {
    /// Statistics for baseline configuration by metric
    pub baseline_stats: HashMap<String, AggregatedStats>,
    /// Statistics for variant configuration by metric
    pub variant_stats: HashMap<String, AggregatedStats>,
    /// Detailed comparisons per metric
    pub comparisons: Vec<ABMetricComparison>,
    /// Overall conclusion
    pub conclusion: ABConclusion,
    /// Significance level used
    pub alpha: f64,
    /// Number of replications
    pub replications: u32,
}

impl ABTestResult {
    /// Get metrics where variant is significantly better
    pub fn variant_wins(&self) -> Vec<&ABMetricComparison> {
        self.comparisons
            .iter()
            .filter(|c| {
                c.is_significant
                    && ((c.higher_is_better && c.absolute_diff > 0.0)
                        || (!c.higher_is_better && c.absolute_diff < 0.0))
            })
            .collect()
    }

    /// Get metrics where baseline is significantly better
    pub fn baseline_wins(&self) -> Vec<&ABMetricComparison> {
        self.comparisons
            .iter()
            .filter(|c| {
                c.is_significant
                    && ((c.higher_is_better && c.absolute_diff < 0.0)
                        || (!c.higher_is_better && c.absolute_diff > 0.0))
            })
            .collect()
    }

    /// Format the result as a human-readable string
    pub fn summary(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!(
            "A/B Test Results (α={}, n={})\n",
            self.alpha, self.replications
        ));
        output.push_str(&"=".repeat(60));
        output.push('\n');

        for comparison in &self.comparisons {
            let direction = if comparison.absolute_diff > 0.0 {
                "+"
            } else {
                ""
            };
            let sig_marker = if comparison.is_significant { "*" } else { "" };
            output.push_str(&format!(
                "{}: {:.2} → {:.2} ({}{:.2}, {}{:.1}%){} p={:.4}\n",
                comparison.metric_name,
                comparison.baseline_stats.mean,
                comparison.variant_stats.mean,
                direction,
                comparison.absolute_diff,
                direction,
                comparison.relative_diff_pct,
                sig_marker,
                comparison.p_value
            ));
        }

        output.push_str(&"-".repeat(60));
        output.push('\n');
        output.push_str(&format!("Conclusion: {:?}\n", self.conclusion));

        output
    }
}

/// Runs A/B tests with statistical rigor
pub struct ABTestRunner {
    config: ABTestConfig,
}

impl ABTestRunner {
    /// Create a new A/B test runner
    pub fn new(config: ABTestConfig) -> Self {
        Self { config }
    }

    /// Run the A/B test
    pub fn run(&self) -> ABTestResult {
        // Generate seeds for replications
        let seeds: Vec<u64> = (0..self.config.replications)
            .map(|i| 1000 + i as u64)
            .collect();

        // Run baseline scenarios
        let baseline_scenarios: Vec<(String, ScenarioConfig)> = seeds
            .iter()
            .map(|&seed| {
                let mut config = self.config.baseline_config.clone();
                config.seed = seed;
                (format!("baseline_seed={}", seed), config)
            })
            .collect();

        let baseline_runner = BatchRunner::new(baseline_scenarios);
        let baseline_results = baseline_runner.run();

        // Run variant scenarios
        let variant_scenarios: Vec<(String, ScenarioConfig)> = seeds
            .iter()
            .map(|&seed| {
                let mut config = self.config.variant_config.clone();
                config.seed = seed;
                (format!("variant_seed={}", seed), config)
            })
            .collect();

        let variant_runner = BatchRunner::new(variant_scenarios);
        let variant_results = variant_runner.run();

        // Extract metrics and compute statistics
        let metrics = self.extract_and_compare(&baseline_results, &variant_results);

        // Determine conclusion
        let conclusion = self.determine_conclusion(&metrics);

        // Build stats maps
        let mut baseline_stats = HashMap::new();
        let mut variant_stats = HashMap::new();
        for m in &metrics {
            baseline_stats.insert(m.metric_name.clone(), m.baseline_stats.clone());
            variant_stats.insert(m.metric_name.clone(), m.variant_stats.clone());
        }

        ABTestResult {
            baseline_stats,
            variant_stats,
            comparisons: metrics,
            conclusion,
            alpha: self.config.alpha,
            replications: self.config.replications,
        }
    }

    /// Extract metrics and compare them
    fn extract_and_compare(
        &self,
        baseline: &[RunResult],
        variant: &[RunResult],
    ) -> Vec<ABMetricComparison> {
        let mut comparisons = Vec::new();

        // Throughput (higher is better)
        let baseline_throughput: Vec<f64> = baseline.iter().map(|r| r.throughput()).collect();
        let variant_throughput: Vec<f64> = variant.iter().map(|r| r.throughput()).collect();
        comparisons.push(self.compare_metric(
            "Throughput (orders/hr)",
            &baseline_throughput,
            &variant_throughput,
            true,
        ));

        // P95 Cycle Time (lower is better)
        let baseline_p95: Vec<f64> = baseline.iter().map(|r| r.p95_cycle_time()).collect();
        let variant_p95: Vec<f64> = variant.iter().map(|r| r.p95_cycle_time()).collect();
        comparisons.push(self.compare_metric(
            "P95 Cycle Time (s)",
            &baseline_p95,
            &variant_p95,
            false,
        ));

        // Robot Utilization (higher is better)
        let baseline_robot_util: Vec<f64> =
            baseline.iter().map(|r| r.robot_utilization()).collect();
        let variant_robot_util: Vec<f64> = variant.iter().map(|r| r.robot_utilization()).collect();
        comparisons.push(self.compare_metric(
            "Robot Utilization",
            &baseline_robot_util,
            &variant_robot_util,
            true,
        ));

        // Station Utilization (higher is better)
        let baseline_station_util: Vec<f64> =
            baseline.iter().map(|r| r.station_utilization()).collect();
        let variant_station_util: Vec<f64> =
            variant.iter().map(|r| r.station_utilization()).collect();
        comparisons.push(self.compare_metric(
            "Station Utilization",
            &baseline_station_util,
            &variant_station_util,
            true,
        ));

        comparisons
    }

    /// Compare a single metric between baseline and variant
    fn compare_metric(
        &self,
        name: &str,
        baseline_samples: &[f64],
        variant_samples: &[f64],
        higher_is_better: bool,
    ) -> ABMetricComparison {
        let baseline_stats = AggregatedStats::from_samples(baseline_samples);
        let variant_stats = AggregatedStats::from_samples(variant_samples);

        let absolute_diff = variant_stats.mean - baseline_stats.mean;
        let relative_diff_pct = if baseline_stats.mean.abs() > f64::EPSILON {
            (absolute_diff / baseline_stats.mean) * 100.0
        } else {
            0.0
        };

        let (t_statistic, p_value) = welchs_t_test(baseline_samples, variant_samples);
        let is_significant = p_value < self.config.alpha;

        ABMetricComparison {
            metric_name: name.to_string(),
            baseline_stats,
            variant_stats,
            absolute_diff,
            relative_diff_pct,
            t_statistic,
            p_value,
            is_significant,
            higher_is_better,
        }
    }

    /// Determine the overall conclusion
    fn determine_conclusion(&self, comparisons: &[ABMetricComparison]) -> ABConclusion {
        let mut variant_better: Vec<String> = Vec::new();
        let mut baseline_better: Vec<String> = Vec::new();

        for c in comparisons {
            if !c.is_significant {
                continue;
            }

            let variant_wins = if c.higher_is_better {
                c.absolute_diff > 0.0
            } else {
                c.absolute_diff < 0.0
            };

            if variant_wins {
                variant_better.push(c.metric_name.clone());
            } else {
                baseline_better.push(c.metric_name.clone());
            }
        }

        match (variant_better.is_empty(), baseline_better.is_empty()) {
            (true, true) => ABConclusion::NoSignificantDifference,
            (false, true) => ABConclusion::VariantBetter {
                metrics: variant_better,
            },
            (true, false) => ABConclusion::BaselineBetter {
                metrics: baseline_better,
            },
            (false, false) => ABConclusion::Mixed {
                better: variant_better,
                worse: baseline_better,
            },
        }
    }

    /// Calculate required sample size for desired statistical power
    ///
    /// Uses the formula for two-sample t-test:
    /// n = 2 * ((z_alpha + z_beta)^2 * sigma^2) / delta^2
    pub fn required_sample_size(
        baseline_mean: f64,
        baseline_std: f64,
        minimum_effect: f64,
        alpha: f64,
        power: f64,
    ) -> u32 {
        // Z-scores for alpha and power
        let z_alpha = z_score_for_alpha(alpha / 2.0); // Two-tailed
        let z_beta = z_score_for_power(power);

        let delta = minimum_effect * baseline_mean;
        if delta.abs() < f64::EPSILON {
            return 100; // Default if effect size is zero
        }

        let variance = baseline_std * baseline_std;
        let n = 2.0 * (z_alpha + z_beta).powi(2) * variance / (delta * delta);

        (n.ceil() as u32).max(5) // At least 5 samples
    }
}

/// Welch's t-test for comparing two samples with potentially unequal variances
///
/// Returns (t_statistic, p_value)
pub fn welchs_t_test(a: &[f64], b: &[f64]) -> (f64, f64) {
    if a.is_empty() || b.is_empty() {
        return (0.0, 1.0);
    }

    let n1 = a.len() as f64;
    let n2 = b.len() as f64;

    let mean1 = a.iter().sum::<f64>() / n1;
    let mean2 = b.iter().sum::<f64>() / n2;

    let var1 = if a.len() > 1 {
        a.iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / (n1 - 1.0)
    } else {
        0.0
    };

    let var2 = if b.len() > 1 {
        b.iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / (n2 - 1.0)
    } else {
        0.0
    };

    // Avoid division by zero
    if var1.abs() < f64::EPSILON && var2.abs() < f64::EPSILON {
        return (0.0, 1.0);
    }

    let se1 = var1 / n1;
    let se2 = var2 / n2;
    let se_diff = (se1 + se2).sqrt();

    if se_diff.abs() < f64::EPSILON {
        return (0.0, 1.0);
    }

    let t_statistic = (mean1 - mean2) / se_diff;

    // Welch-Satterthwaite degrees of freedom
    let df_num = (se1 + se2).powi(2);
    let df_denom = if n1 > 1.0 && n2 > 1.0 {
        (se1.powi(2) / (n1 - 1.0)) + (se2.powi(2) / (n2 - 1.0))
    } else {
        1.0
    };

    let df = if df_denom.abs() < f64::EPSILON {
        1.0
    } else {
        df_num / df_denom
    };

    // Calculate p-value using Student's t-distribution
    let p_value = match StudentsT::new(0.0, 1.0, df) {
        Ok(t_dist) => {
            let cdf = t_dist.cdf(t_statistic.abs());
            2.0 * (1.0 - cdf) // Two-tailed
        }
        Err(_) => 1.0,
    };

    (t_statistic, p_value)
}

/// Get Z-score for a given alpha (significance level)
fn z_score_for_alpha(alpha: f64) -> f64 {
    // Approximation using inverse normal CDF
    // For common values:
    // alpha=0.025 (two-tailed 0.05) → z ≈ 1.96
    // alpha=0.005 (two-tailed 0.01) → z ≈ 2.576
    if alpha <= 0.005 {
        2.576
    } else if alpha <= 0.01 {
        2.326
    } else if alpha <= 0.025 {
        1.96
    } else if alpha <= 0.05 {
        1.645
    } else {
        1.282
    }
}

/// Get Z-score for a given power level
fn z_score_for_power(power: f64) -> f64 {
    // Common power values:
    // power=0.80 → z ≈ 0.84
    // power=0.90 → z ≈ 1.28
    // power=0.95 → z ≈ 1.645
    if power >= 0.95 {
        1.645
    } else if power >= 0.90 {
        1.282
    } else if power >= 0.80 {
        0.842
    } else {
        0.524
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welchs_t_test_identical() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let (t, p) = welchs_t_test(&a, &b);
        assert!(t.abs() < f64::EPSILON);
        assert!(p > 0.99);
    }

    #[test]
    fn test_welchs_t_test_different() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![10.0, 11.0, 12.0, 13.0, 14.0];

        let (t, p) = welchs_t_test(&a, &b);
        assert!(t.abs() > 5.0);
        assert!(p < 0.01);
    }

    #[test]
    fn test_welchs_t_test_empty() {
        let a: Vec<f64> = vec![];
        let b = vec![1.0, 2.0, 3.0];

        let (_, p) = welchs_t_test(&a, &b);
        assert!((p - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_required_sample_size() {
        let n = ABTestRunner::required_sample_size(
            100.0, // baseline mean
            10.0,  // baseline std
            0.10,  // 10% effect
            0.05,  // alpha
            0.80,  // power
        );

        // Should need reasonable sample size
        assert!(n >= 5);
        assert!(n <= 1000);
    }

    #[test]
    fn test_ab_conclusion_variants() {
        // Test all conclusion types
        let no_diff = ABConclusion::NoSignificantDifference;
        assert_eq!(no_diff, ABConclusion::NoSignificantDifference);

        let variant_better = ABConclusion::VariantBetter {
            metrics: vec!["throughput".to_string()],
        };
        if let ABConclusion::VariantBetter { metrics } = variant_better {
            assert_eq!(metrics.len(), 1);
        }

        let mixed = ABConclusion::Mixed {
            better: vec!["throughput".to_string()],
            worse: vec!["latency".to_string()],
        };
        if let ABConclusion::Mixed { better, worse } = mixed {
            assert_eq!(better.len(), 1);
            assert_eq!(worse.len(), 1);
        }
    }
}
