//! Testing and benchmarking framework for Waremax
//!
//! This crate provides:
//! - ScenarioBuilder for programmatic scenario creation
//! - SweepGenerator for parameter sweeps
//! - BatchRunner for parallel simulation execution
//! - Comparison framework with statistical analysis
//! - A/B testing with significance tests
//! - Benchmarking suite with regression detection

pub mod generator;
pub mod presets;
pub mod runner;
pub mod comparison;
pub mod ab_testing;
pub mod benchmark;

pub use generator::{ScenarioBuilder, SweepGenerator, SweepDimension};
pub use presets::ScenarioPreset;
pub use runner::{BatchRunner, RunResult};
pub use comparison::{AggregatedStats, MetricComparison, ComparisonReport, ScenarioComparator};
pub use ab_testing::{ABTestConfig, ABTestRunner, ABTestResult, ABConclusion, welchs_t_test};
pub use benchmark::{BenchmarkSuite, Benchmark, BenchmarkResults, BenchmarkResult, RegressionAlert, BenchmarkHistory};
