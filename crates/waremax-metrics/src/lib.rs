//! Waremax Metrics - Metrics collection, event logging, and reporting

pub mod collector;
pub mod export;
pub mod report;
pub mod timeseries;
pub mod trace;

pub use collector::{MetricsCollector, SLAMetrics};
pub use export::{
    write_exports, write_robot_csv, write_station_csv, write_heatmap_csv,
    write_timeseries_csv, write_trace_csv, ExportOptions,
};
pub use report::{
    SimulationReport, SLAReport, CongestionReport, BatteryReport,
    RobotReport, StationReport, ReliabilityReport, HeatmapData,
    NodeCongestion, EdgeCongestion,
};
pub use timeseries::{
    TimeSeriesCollector, StationTimeSeriesData, ChargingTimeSeriesData,
    CongestionMetrics, CongestionRanking, DataPoint,
};
pub use trace::{EventTraceCollector, TraceEntry, TraceDetails};
