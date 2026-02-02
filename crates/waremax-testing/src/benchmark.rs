//! Benchmarking suite with regression detection
//!
//! Provides BenchmarkSuite for running named benchmarks and
//! BenchmarkHistory for tracking performance over time.

use std::collections::HashMap;
use std::io::{self, BufReader, BufWriter};
use std::fs::File;
use std::path::Path;
use serde::{Deserialize, Serialize};
use chrono::Utc;

use waremax_config::ScenarioConfig;
use crate::runner::BatchRunner;
use crate::comparison::AggregatedStats;
use crate::presets::ScenarioPreset;

/// A single benchmark definition
#[derive(Clone)]
pub struct Benchmark {
    /// Name of the benchmark
    pub name: String,
    /// Scenario configuration
    pub scenario: ScenarioConfig,
    /// Expected throughput (for regression detection)
    pub expected_throughput: Option<f64>,
    /// Expected P95 latency (for regression detection)
    pub expected_latency_p95: Option<f64>,
    /// Tolerance for regression as percentage (default: 5%)
    pub regression_tolerance_pct: f64,
}

impl Benchmark {
    /// Create a new benchmark
    pub fn new(name: &str, scenario: ScenarioConfig) -> Self {
        Self {
            name: name.to_string(),
            scenario,
            expected_throughput: None,
            expected_latency_p95: None,
            regression_tolerance_pct: 5.0,
        }
    }

    /// Set expected throughput for regression detection
    pub fn expect_throughput(mut self, throughput: f64) -> Self {
        self.expected_throughput = Some(throughput);
        self
    }

    /// Set expected P95 latency for regression detection
    pub fn expect_latency_p95(mut self, latency: f64) -> Self {
        self.expected_latency_p95 = Some(latency);
        self
    }

    /// Set regression tolerance percentage
    pub fn tolerance(mut self, pct: f64) -> Self {
        self.regression_tolerance_pct = pct;
        self
    }
}

/// A suite of benchmarks
pub struct BenchmarkSuite {
    /// Name of the suite
    name: String,
    /// Benchmarks in the suite
    benchmarks: Vec<Benchmark>,
    /// Number of replications per benchmark
    replications: u32,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            benchmarks: Vec::new(),
            replications: 5,
        }
    }

    /// Set number of replications
    pub fn replications(mut self, n: u32) -> Self {
        self.replications = n;
        self
    }

    /// Add a benchmark
    pub fn add(mut self, name: &str, scenario: ScenarioConfig) -> Self {
        self.benchmarks.push(Benchmark::new(name, scenario));
        self
    }

    /// Add a benchmark with expectations for regression detection
    pub fn add_with_expectations(
        mut self,
        name: &str,
        scenario: ScenarioConfig,
        expected_throughput: f64,
        expected_p95: f64,
    ) -> Self {
        self.benchmarks.push(
            Benchmark::new(name, scenario)
                .expect_throughput(expected_throughput)
                .expect_latency_p95(expected_p95)
        );
        self
    }

    /// Add a benchmark from a preset
    pub fn add_preset(mut self, preset: ScenarioPreset) -> Self {
        self.benchmarks.push(Benchmark::new(preset.name(), preset.config()));
        self
    }

    /// Run all benchmarks
    pub fn run(&self) -> BenchmarkResults {
        let mut results = Vec::new();
        let mut regressions = Vec::new();

        for benchmark in &self.benchmarks {
            // Generate seeds for replications
            let seeds: Vec<u64> = (0..self.replications)
                .map(|i| 42 + i as u64)
                .collect();

            // Create scenarios with different seeds
            let scenarios: Vec<(String, ScenarioConfig)> = seeds.iter()
                .map(|&seed| {
                    let mut config = benchmark.scenario.clone();
                    config.seed = seed;
                    (format!("{}_seed={}", benchmark.name, seed), config)
                })
                .collect();

            // Run in parallel
            let runner = BatchRunner::new(scenarios);
            let run_results = runner.run();

            // Compute statistics
            let throughput_samples: Vec<f64> = run_results.iter()
                .map(|r| r.throughput())
                .collect();
            let throughput_stats = AggregatedStats::from_samples(&throughput_samples);

            let latency_samples: Vec<f64> = run_results.iter()
                .map(|r| r.p95_cycle_time())
                .collect();
            let latency_stats = AggregatedStats::from_samples(&latency_samples);

            let util_samples: Vec<f64> = run_results.iter()
                .map(|r| r.robot_utilization())
                .collect();
            let util_stats = AggregatedStats::from_samples(&util_samples);

            // Check for regressions
            let mut passed = true;

            if let Some(expected) = benchmark.expected_throughput {
                let tolerance = expected * benchmark.regression_tolerance_pct / 100.0;
                if throughput_stats.mean < expected - tolerance {
                    let deviation = (expected - throughput_stats.mean) / expected * 100.0;
                    regressions.push(RegressionAlert {
                        benchmark: benchmark.name.clone(),
                        metric: "throughput".to_string(),
                        expected,
                        actual: throughput_stats.mean,
                        deviation_pct: deviation,
                    });
                    passed = false;
                }
            }

            if let Some(expected) = benchmark.expected_latency_p95 {
                let tolerance = expected * benchmark.regression_tolerance_pct / 100.0;
                if latency_stats.mean > expected + tolerance {
                    let deviation = (latency_stats.mean - expected) / expected * 100.0;
                    regressions.push(RegressionAlert {
                        benchmark: benchmark.name.clone(),
                        metric: "latency_p95".to_string(),
                        expected,
                        actual: latency_stats.mean,
                        deviation_pct: deviation,
                    });
                    passed = false;
                }
            }

            results.push(BenchmarkResult {
                name: benchmark.name.clone(),
                throughput: throughput_stats,
                latency_p95: latency_stats,
                utilization: util_stats,
                passed_expectations: passed,
                replications: self.replications,
            });
        }

        BenchmarkResults {
            suite_name: self.name.clone(),
            results,
            regressions,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    /// Create default benchmark suite with standard presets
    pub fn default_suite() -> Self {
        Self::new("default")
            .replications(5)
            .add_preset(ScenarioPreset::Minimal)
            .add_preset(ScenarioPreset::Quick)
            .add_preset(ScenarioPreset::Standard)
    }
}

/// Results of running a benchmark suite
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkResults {
    /// Name of the suite
    pub suite_name: String,
    /// Individual benchmark results
    pub results: Vec<BenchmarkResult>,
    /// Detected regressions
    pub regressions: Vec<RegressionAlert>,
    /// Timestamp when benchmarks were run
    pub timestamp: String,
}

impl BenchmarkResults {
    /// Check if any regressions were detected
    pub fn has_regressions(&self) -> bool {
        !self.regressions.is_empty()
    }

    /// Get all passing benchmarks
    pub fn passed(&self) -> Vec<&BenchmarkResult> {
        self.results.iter().filter(|r| r.passed_expectations).collect()
    }

    /// Get all failing benchmarks
    pub fn failed(&self) -> Vec<&BenchmarkResult> {
        self.results.iter().filter(|r| !r.passed_expectations).collect()
    }

    /// Format results as a human-readable string
    pub fn summary(&self) -> String {
        let mut output = format!("Benchmark Suite: {}\n", self.suite_name);
        output.push_str(&format!("Run at: {}\n", self.timestamp));
        output.push_str(&"=".repeat(70));
        output.push('\n');

        for result in &self.results {
            let status = if result.passed_expectations { "PASS" } else { "FAIL" };
            output.push_str(&format!(
                "[{}] {}\n",
                status, result.name
            ));
            output.push_str(&format!(
                "    Throughput: {:.1} ± {:.1} orders/hr\n",
                result.throughput.mean, result.throughput.std_dev
            ));
            output.push_str(&format!(
                "    P95 Latency: {:.2} ± {:.2} s\n",
                result.latency_p95.mean, result.latency_p95.std_dev
            ));
            output.push_str(&format!(
                "    Utilization: {:.1}% ± {:.1}%\n",
                result.utilization.mean * 100.0, result.utilization.std_dev * 100.0
            ));
        }

        if !self.regressions.is_empty() {
            output.push_str(&"-".repeat(70));
            output.push('\n');
            output.push_str("REGRESSIONS DETECTED:\n");
            for reg in &self.regressions {
                output.push_str(&format!(
                    "  {} - {}: expected {:.2}, got {:.2} ({:+.1}%)\n",
                    reg.benchmark, reg.metric, reg.expected, reg.actual, reg.deviation_pct
                ));
            }
        }

        output
    }

    /// Save results to JSON file
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    /// Load results from JSON file
    pub fn load(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let results = serde_json::from_reader(reader)?;
        Ok(results)
    }
}

/// Result of a single benchmark
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Throughput statistics (orders/hour)
    pub throughput: AggregatedStats,
    /// P95 cycle time statistics
    pub latency_p95: AggregatedStats,
    /// Robot utilization statistics
    pub utilization: AggregatedStats,
    /// Whether expectations were met
    pub passed_expectations: bool,
    /// Number of replications
    pub replications: u32,
}

/// Alert for a detected regression
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegressionAlert {
    /// Benchmark name
    pub benchmark: String,
    /// Metric that regressed
    pub metric: String,
    /// Expected value
    pub expected: f64,
    /// Actual value
    pub actual: f64,
    /// Deviation percentage (positive = worse)
    pub deviation_pct: f64,
}

/// Historical benchmark data for trend analysis
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct BenchmarkHistory {
    /// Historical entries (timestamp, results)
    entries: Vec<BenchmarkResults>,
    /// Maximum entries to keep
    max_entries: usize,
}

impl BenchmarkHistory {
    /// Create new history tracker
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    /// Load history from file
    pub fn load(path: &Path) -> io::Result<Self> {
        if !path.exists() {
            return Ok(Self::new(100));
        }
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let history = serde_json::from_reader(reader)?;
        Ok(history)
    }

    /// Save history to file
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    /// Add new results to history
    pub fn add(&mut self, results: BenchmarkResults) {
        self.entries.push(results);
        while self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the latest results
    pub fn latest(&self) -> Option<&BenchmarkResults> {
        self.entries.last()
    }

    /// Detect regressions by comparing latest results to historical baseline
    pub fn detect_regressions(&self, latest: &BenchmarkResults, threshold_pct: f64) -> Vec<RegressionAlert> {
        if self.entries.is_empty() {
            return Vec::new();
        }

        let mut alerts = Vec::new();

        // Compute historical baseline (average of last N runs)
        let baseline_runs = self.entries.iter().rev().take(5).collect::<Vec<_>>();
        let mut baseline_throughput: HashMap<String, Vec<f64>> = HashMap::new();
        let mut baseline_latency: HashMap<String, Vec<f64>> = HashMap::new();

        for run in &baseline_runs {
            for result in &run.results {
                baseline_throughput.entry(result.name.clone())
                    .or_default()
                    .push(result.throughput.mean);
                baseline_latency.entry(result.name.clone())
                    .or_default()
                    .push(result.latency_p95.mean);
            }
        }

        // Compare latest to baseline
        for result in &latest.results {
            if let Some(historical) = baseline_throughput.get(&result.name) {
                let baseline_avg = historical.iter().sum::<f64>() / historical.len() as f64;
                let deviation = (baseline_avg - result.throughput.mean) / baseline_avg * 100.0;

                if deviation > threshold_pct {
                    alerts.push(RegressionAlert {
                        benchmark: result.name.clone(),
                        metric: "throughput".to_string(),
                        expected: baseline_avg,
                        actual: result.throughput.mean,
                        deviation_pct: deviation,
                    });
                }
            }

            if let Some(historical) = baseline_latency.get(&result.name) {
                let baseline_avg = historical.iter().sum::<f64>() / historical.len() as f64;
                let deviation = (result.latency_p95.mean - baseline_avg) / baseline_avg * 100.0;

                if deviation > threshold_pct {
                    alerts.push(RegressionAlert {
                        benchmark: result.name.clone(),
                        metric: "latency_p95".to_string(),
                        expected: baseline_avg,
                        actual: result.latency_p95.mean,
                        deviation_pct: deviation,
                    });
                }
            }
        }

        alerts
    }

    /// Get trend data for a specific benchmark and metric
    pub fn trend(&self, benchmark: &str, metric: &str) -> Vec<(String, f64)> {
        self.entries.iter()
            .filter_map(|run| {
                let result = run.results.iter().find(|r| r.name == benchmark)?;
                let value = match metric {
                    "throughput" => result.throughput.mean,
                    "latency_p95" => result.latency_p95.mean,
                    "utilization" => result.utilization.mean,
                    _ => return None,
                };
                Some((run.timestamp.clone(), value))
            })
            .collect()
    }

    /// Get all entries
    pub fn entries(&self) -> &[BenchmarkResults] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_new() {
        let config = ScenarioPreset::Minimal.config();
        let bench = Benchmark::new("test", config)
            .expect_throughput(100.0)
            .expect_latency_p95(5.0)
            .tolerance(10.0);

        assert_eq!(bench.name, "test");
        assert_eq!(bench.expected_throughput, Some(100.0));
        assert_eq!(bench.expected_latency_p95, Some(5.0));
        assert_eq!(bench.regression_tolerance_pct, 10.0);
    }

    #[test]
    fn test_benchmark_suite_creation() {
        let suite = BenchmarkSuite::new("test_suite")
            .replications(3)
            .add_preset(ScenarioPreset::Minimal);

        assert_eq!(suite.name, "test_suite");
        assert_eq!(suite.replications, 3);
        assert_eq!(suite.benchmarks.len(), 1);
    }

    #[test]
    fn test_default_suite() {
        let suite = BenchmarkSuite::default_suite();
        assert_eq!(suite.name, "default");
        assert!(!suite.benchmarks.is_empty());
    }

    #[test]
    fn test_regression_alert() {
        let alert = RegressionAlert {
            benchmark: "test".to_string(),
            metric: "throughput".to_string(),
            expected: 100.0,
            actual: 90.0,
            deviation_pct: 10.0,
        };

        assert_eq!(alert.benchmark, "test");
        assert_eq!(alert.deviation_pct, 10.0);
    }

    #[test]
    fn test_benchmark_history() {
        let mut history = BenchmarkHistory::new(10);
        assert!(history.is_empty());

        let results = BenchmarkResults {
            suite_name: "test".to_string(),
            results: vec![],
            regressions: vec![],
            timestamp: Utc::now().to_rfc3339(),
        };

        history.add(results);
        assert_eq!(history.len(), 1);
        assert!(!history.is_empty());
    }

    #[test]
    fn test_history_max_entries() {
        let mut history = BenchmarkHistory::new(3);

        for i in 0..5 {
            history.add(BenchmarkResults {
                suite_name: format!("test_{}", i),
                results: vec![],
                regressions: vec![],
                timestamp: Utc::now().to_rfc3339(),
            });
        }

        assert_eq!(history.len(), 3);
        assert_eq!(history.entries()[0].suite_name, "test_2");
    }
}
