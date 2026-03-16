//! Waremax Metrics - Metrics collection, event logging, and reporting

pub mod collector;
pub mod report;

pub use collector::MetricsCollector;
pub use report::SimulationReport;
