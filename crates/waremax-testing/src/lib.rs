//! Testing and benchmarking framework for Waremax
//!
//! This crate provides:
//! - ScenarioBuilder for programmatic scenario creation
//! - SweepGenerator for parameter sweeps
//! - BatchRunner for parallel simulation execution
//! - Comparison framework with statistical analysis
//! - A/B testing with significance tests
//! - Benchmarking suite with regression detection

pub mod ab_testing;
pub mod benchmark;
pub mod comparison;
pub mod generator;
pub mod presets;
pub mod runner;

pub use ab_testing::{welchs_t_test, ABConclusion, ABTestConfig, ABTestResult, ABTestRunner};
pub use benchmark::{
    Benchmark, BenchmarkHistory, BenchmarkResult, BenchmarkResults, BenchmarkSuite, RegressionAlert,
};
pub use comparison::{AggregatedStats, ComparisonReport, MetricComparison, ScenarioComparator};
pub use generator::{ScenarioBuilder, SweepDimension, SweepGenerator};
pub use presets::ScenarioPreset;
pub use runner::{BatchRunner, RunResult};
