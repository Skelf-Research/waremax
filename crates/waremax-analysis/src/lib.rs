//! Root Cause Analysis for Waremax simulations
//!
//! This crate provides intelligent analysis capabilities to identify WHY
//! performance issues occur, not just WHAT the metrics are.
//!
//! ## Features
//!
//! - **Delay Attribution**: Track time breakdown per task by category
//! - **Bottleneck Detection**: Rank congested nodes/edges, overloaded stations
//! - **Critical Path Analysis**: Identify slowest phases per order
//! - **Anomaly Detection**: Flag unusual patterns using statistical methods
//! - **Recommendations**: Generate actionable suggestions based on detected issues

pub mod analyzer;
pub mod anomaly;
pub mod attribution;
pub mod bottleneck;
pub mod critical_path;
pub mod reporter;

pub use analyzer::{
    AnalyzerInput, RCASummary, Recommendation, RootCauseAnalysisReport, RootCauseAnalyzer,
};
pub use anomaly::{Anomaly, AnomalyDetector, AnomalyType};
pub use attribution::{
    AttributionCollector, CongestionEvent, DelayAttributionSummary, DelayCategory, QueueWaitEvent,
    TaskAttribution,
};
pub use bottleneck::{BottleneckAnalysis, BottleneckDetector, BottleneckType};
pub use critical_path::{CriticalPathAnalysis, CriticalPathSummary, OrderCriticalPath};
pub use reporter::{RCAReporter, ReportFormat};
