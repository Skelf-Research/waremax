//! CSV and file export utilities for simulation reports

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use waremax_core::StationId;

use crate::report::{
    EdgeCongestion, HeatmapData, NodeCongestion, RobotReport, SimulationReport, StationReport,
};
use crate::timeseries::{StationTimeSeriesData, TimeSeriesCollector};
use crate::trace::{EventTraceCollector, TraceDetails, TraceEntry};

/// Write per-robot breakdown to CSV
pub fn write_robot_csv(path: &Path, robots: &[RobotReport]) -> io::Result<()> {
    let mut file = File::create(path)?;

    // Header
    writeln!(
        file,
        "robot_id,tasks_completed,distance_traveled_m,energy_consumed_wh,idle_time_s,working_time_s,charging_time_s,maintenance_time_s,failure_count,utilization"
    )?;

    // Data rows
    for robot in robots {
        writeln!(
            file,
            "{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{},{:.4}",
            robot.robot_id,
            robot.tasks_completed,
            robot.distance_traveled_m,
            robot.energy_consumed_wh,
            robot.idle_time_s,
            robot.working_time_s,
            robot.charging_time_s,
            robot.maintenance_time_s,
            robot.failure_count,
            robot.utilization
        )?;
    }

    Ok(())
}

/// Write per-station breakdown to CSV
pub fn write_station_csv(path: &Path, stations: &[StationReport]) -> io::Result<()> {
    let mut file = File::create(path)?;

    // Header
    writeln!(
        file,
        "station_id,string_id,station_type,orders_served,total_service_time_s,avg_service_time_s,avg_queue_length,max_queue_length,utilization"
    )?;

    // Data rows
    for station in stations {
        writeln!(
            file,
            "{},{},{},{},{:.2},{:.2},{:.2},{},{:.4}",
            station.station_id,
            station.string_id,
            station.station_type,
            station.orders_served,
            station.total_service_time_s,
            station.avg_service_time_s,
            station.avg_queue_length,
            station.max_queue_length,
            station.utilization
        )?;
    }

    Ok(())
}

/// Write node congestion heatmap data to CSV
pub fn write_node_heatmap_csv(path: &Path, nodes: &[NodeCongestion]) -> io::Result<()> {
    let mut file = File::create(path)?;

    // Header
    writeln!(
        file,
        "node_id,x,y,total_wait_time_s,wait_event_count,congestion_score"
    )?;

    // Data rows
    for node in nodes {
        writeln!(
            file,
            "{},{:.2},{:.2},{:.2},{},{:.2}",
            node.node_id,
            node.x,
            node.y,
            node.total_wait_time_s,
            node.wait_event_count,
            node.congestion_score
        )?;
    }

    Ok(())
}

/// Write edge congestion heatmap data to CSV
pub fn write_edge_heatmap_csv(path: &Path, edges: &[EdgeCongestion]) -> io::Result<()> {
    let mut file = File::create(path)?;

    // Header
    writeln!(
        file,
        "edge_id,from_node,to_node,total_wait_time_s,wait_event_count,congestion_score"
    )?;

    // Data rows
    for edge in edges {
        writeln!(
            file,
            "{},{},{},{:.2},{},{:.2}",
            edge.edge_id,
            edge.from_node,
            edge.to_node,
            edge.total_wait_time_s,
            edge.wait_event_count,
            edge.congestion_score
        )?;
    }

    Ok(())
}

/// Write heatmap data to CSV (creates two files: nodes and edges)
pub fn write_heatmap_csv(
    node_path: &Path,
    edge_path: &Path,
    heatmap: &HeatmapData,
) -> io::Result<()> {
    write_node_heatmap_csv(node_path, &heatmap.node_congestion)?;
    write_edge_heatmap_csv(edge_path, &heatmap.edge_congestion)?;
    Ok(())
}

/// Write time-series data to CSV
pub fn write_timeseries_csv(
    path: &Path,
    station_series: &HashMap<StationId, StationTimeSeriesData>,
) -> io::Result<()> {
    let mut file = File::create(path)?;

    // Header
    writeln!(file, "station_id,time_s,queue_length,utilization")?;

    // Collect and sort all data points
    let mut all_points: Vec<(u32, f64, usize, f64)> = Vec::new();

    for (station_id, series) in station_series {
        for point in &series.queue_length {
            all_points.push((station_id.0, point.time_s, point.value, 0.0));
        }
    }

    // Sort by time
    all_points.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Write data rows
    for (station_id, time_s, queue_length, utilization) in all_points {
        writeln!(
            file,
            "{},{:.2},{},{:.4}",
            station_id, time_s, queue_length, utilization
        )?;
    }

    Ok(())
}

/// Write event trace to CSV
pub fn write_trace_csv(path: &Path, entries: &[TraceEntry]) -> io::Result<()> {
    let mut file = File::create(path)?;

    // Header
    writeln!(file, "timestamp,event_type,details")?;

    // Data rows
    for entry in entries {
        let details_str = match &entry.details {
            TraceDetails::RobotMove {
                robot_id,
                from_node,
                to_node,
            } => format!("robot={} from={} to={}", robot_id, from_node, to_node),
            TraceDetails::TaskAssign { task_id, robot_id } => {
                format!("task={} robot={}", task_id, robot_id)
            }
            TraceDetails::TaskComplete { task_id, robot_id } => {
                format!("task={} robot={}", task_id, robot_id)
            }
            TraceDetails::OrderComplete {
                order_id,
                cycle_time_s,
                is_late,
            } => format!(
                "order={} cycle_time={:.2}s late={}",
                order_id, cycle_time_s, is_late
            ),
            TraceDetails::StationService {
                station_id,
                robot_id,
                duration_s,
            } => format!(
                "station={} robot={} duration={:.2}s",
                station_id, robot_id, duration_s
            ),
            TraceDetails::RobotFailure { robot_id } => format!("robot={}", robot_id),
            TraceDetails::RobotMaintenance {
                robot_id,
                station_id,
                is_repair,
            } => format!(
                "robot={} station={} repair={}",
                robot_id, station_id, is_repair
            ),
            TraceDetails::ChargingStart {
                robot_id,
                station_id,
                soc,
            } => format!("robot={} station={} soc={:.2}", robot_id, station_id, soc),
            TraceDetails::ChargingEnd {
                robot_id,
                energy_wh,
            } => {
                format!("robot={} energy={:.2}Wh", robot_id, energy_wh)
            }
            TraceDetails::Generic { message } => message.clone(),
        };

        writeln!(
            file,
            "{:.3},{},\"{}\"",
            entry.timestamp, entry.event_type, details_str
        )?;
    }

    Ok(())
}

/// Export options for write_exports
#[derive(Clone, Debug, Default)]
pub struct ExportOptions {
    pub robots: bool,
    pub stations: bool,
    pub heatmap: bool,
    pub timeseries: bool,
    pub trace: bool,
    pub json: bool,
}

impl ExportOptions {
    pub fn all() -> Self {
        Self {
            robots: true,
            stations: true,
            heatmap: true,
            timeseries: true,
            trace: true,
            json: true,
        }
    }
}

/// Master export function - writes all enabled exports to the output directory
pub fn write_exports(
    output_dir: &Path,
    report: &SimulationReport,
    timeseries: Option<&TimeSeriesCollector>,
    trace: Option<&EventTraceCollector>,
    options: &ExportOptions,
) -> io::Result<()> {
    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    // Write JSON report
    if options.json {
        let json_path = output_dir.join("report.json");
        let mut file = File::create(&json_path)?;
        file.write_all(report.to_json().as_bytes())?;
    }

    // Write per-robot CSV
    if options.robots {
        if let Some(ref robots) = report.robot_reports {
            let path = output_dir.join("robots.csv");
            write_robot_csv(&path, robots)?;
        }
    }

    // Write per-station CSV
    if options.stations {
        if let Some(ref stations) = report.station_reports {
            let path = output_dir.join("stations.csv");
            write_station_csv(&path, stations)?;
        }
    }

    // Write heatmap CSVs
    if options.heatmap {
        if let Some(ref heatmap) = report.heatmap {
            let node_path = output_dir.join("node_congestion.csv");
            let edge_path = output_dir.join("edge_congestion.csv");
            write_heatmap_csv(&node_path, &edge_path, heatmap)?;
        }
    }

    // Write time-series CSV
    if options.timeseries {
        if let Some(ts) = timeseries {
            let path = output_dir.join("timeseries.csv");
            write_timeseries_csv(&path, &ts.station_series)?;
        }
    }

    // Write trace CSV
    if options.trace {
        if let Some(tc) = trace {
            if !tc.is_empty() {
                let path = output_dir.join("trace.csv");
                write_trace_csv(&path, &tc.to_vec())?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::tempdir;

    #[test]
    fn test_write_robot_csv() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("robots.csv");

        let robots = vec![
            RobotReport {
                robot_id: 0,
                tasks_completed: 10,
                distance_traveled_m: 500.0,
                energy_consumed_wh: 50.0,
                idle_time_s: 100.0,
                working_time_s: 800.0,
                charging_time_s: 100.0,
                maintenance_time_s: 0.0,
                failure_count: 0,
                utilization: 0.8,
            },
            RobotReport {
                robot_id: 1,
                tasks_completed: 8,
                distance_traveled_m: 400.0,
                energy_consumed_wh: 40.0,
                idle_time_s: 200.0,
                working_time_s: 700.0,
                charging_time_s: 100.0,
                maintenance_time_s: 0.0,
                failure_count: 1,
                utilization: 0.7,
            },
        ];

        write_robot_csv(&path, &robots).unwrap();

        let mut content = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        assert!(content.contains("robot_id,tasks_completed"));
        assert!(content.contains("0,10,500.00"));
        assert!(content.contains("1,8,400.00"));
    }

    #[test]
    fn test_write_station_csv() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("stations.csv");

        let stations = vec![StationReport {
            station_id: 0,
            string_id: "PICK_1".to_string(),
            station_type: "Pick".to_string(),
            orders_served: 50,
            total_service_time_s: 400.0,
            avg_service_time_s: 8.0,
            avg_queue_length: 1.5,
            max_queue_length: 5,
            utilization: 0.75,
        }];

        write_station_csv(&path, &stations).unwrap();

        let mut content = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        assert!(content.contains("station_id,string_id"));
        assert!(content.contains("0,PICK_1,Pick,50"));
    }
}
