//! Waremax Metrics - Metrics collection, event logging, and reporting

pub mod charts;
pub mod collector;
pub mod event_log;
pub mod export;
pub mod html_report;
pub mod pdf_report;
pub mod report;
pub mod timeseries;
pub mod trace;

pub use charts::{ChartConfig, ChartFormat, ChartGenerator};
pub use collector::{MetricsCollector, SLAMetrics};
pub use event_log::{EventLog, EventLogConfig, EventLogReader, EventLogWriter};
pub use export::{
    write_exports, write_heatmap_csv, write_robot_csv, write_station_csv, write_timeseries_csv,
    write_trace_csv, ExportOptions,
};
pub use html_report::HtmlReportGenerator;
pub use pdf_report::PdfReportGenerator;
pub use report::{
    BatteryReport, CongestionReport, EdgeCongestion, HeatmapData, NodeCongestion,
    ReliabilityReport, RobotReport, SLAReport, SimulationReport, StationReport,
};
pub use timeseries::{
    ChargingTimeSeriesData, CongestionMetrics, CongestionRanking, DataPoint, StationTimeSeriesData,
    TimeSeriesCollector,
};
pub use trace::{EventTraceCollector, TraceDetails, TraceEntry};
