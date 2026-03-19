//! Chart generation for visualization
//!
//! v3: Generate PNG charts from simulation data using plotters

use crate::report::SimulationReport;
use crate::timeseries::TimeSeriesCollector;
use plotters::prelude::*;
use plotters_bitmap::BitMapBackend;
use std::io;
use std::path::Path;

/// Chart output format
#[derive(Clone, Debug, Default)]
pub enum ChartFormat {
    #[default]
    PNG,
}

/// Chart generation configuration
#[derive(Clone, Debug)]
pub struct ChartConfig {
    pub width: u32,
    pub height: u32,
    pub format: ChartFormat,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            format: ChartFormat::PNG,
        }
    }
}

/// Chart generator for simulation results
pub struct ChartGenerator {
    config: ChartConfig,
}

impl ChartGenerator {
    pub fn new(config: ChartConfig) -> Self {
        Self { config }
    }

    /// Generate all charts to the specified directory
    pub fn generate_all(
        &self,
        report: &SimulationReport,
        timeseries: Option<&TimeSeriesCollector>,
        output_dir: &Path,
    ) -> io::Result<Vec<String>> {
        let mut generated = Vec::new();

        // Robot utilization bar chart
        if let Some(ref robots) = report.robot_reports {
            if !robots.is_empty() {
                let path = output_dir.join("robot_utilization.png");
                if self.generate_robot_utilization_chart(robots, &path).is_ok() {
                    generated.push("robot_utilization.png".to_string());
                }
            }
        }

        // Station utilization bar chart
        if let Some(ref stations) = report.station_reports {
            if !stations.is_empty() {
                let path = output_dir.join("station_utilization.png");
                if self.generate_station_utilization_chart(stations, &path).is_ok() {
                    generated.push("station_utilization.png".to_string());
                }
            }
        }

        // Queue length time series
        if let Some(ts) = timeseries {
            let path = output_dir.join("queue_lengths.png");
            if self.generate_queue_length_chart(ts, &path).is_ok() {
                generated.push("queue_lengths.png".to_string());
            }

            // Throughput over time
            let path = output_dir.join("throughput.png");
            if self.generate_throughput_chart(ts, &path).is_ok() {
                generated.push("throughput.png".to_string());
            }
        }

        // Congestion heatmap data (simplified - just top congested nodes)
        if let Some(ref congestion) = report.congestion {
            if !congestion.top_congested_nodes.is_empty() {
                let path = output_dir.join("congestion_ranking.png");
                if self.generate_congestion_ranking_chart(&congestion.top_congested_nodes, &path).is_ok() {
                    generated.push("congestion_ranking.png".to_string());
                }
            }
        }

        Ok(generated)
    }

    /// Generate robot utilization bar chart
    fn generate_robot_utilization_chart(
        &self,
        robots: &[crate::report::RobotReport],
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(path, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let max_util = robots.iter()
            .map(|r| r.utilization)
            .fold(0.0f64, |a, b| a.max(b))
            .max(0.1) * 1.1;

        let mut chart = ChartBuilder::on(&root)
            .caption("Robot Utilization", ("sans-serif", 24))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                0usize..robots.len(),
                0.0..max_util,
            )?;

        chart.configure_mesh()
            .x_desc("Robot ID")
            .y_desc("Utilization")
            .y_label_formatter(&|y| format!("{:.0}%", y * 100.0))
            .draw()?;

        chart.draw_series(
            robots.iter().enumerate().map(|(i, r)| {
                let color = if r.utilization > 0.85 {
                    RED.mix(0.8)
                } else if r.utilization > 0.7 {
                    YELLOW.mix(0.8)
                } else {
                    GREEN.mix(0.8)
                };
                Rectangle::new(
                    [(i, 0.0), (i + 1, r.utilization)],
                    color.filled(),
                )
            }),
        )?;

        root.present()?;
        Ok(())
    }

    /// Generate station utilization bar chart
    fn generate_station_utilization_chart(
        &self,
        stations: &[crate::report::StationReport],
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(path, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let max_util = stations.iter()
            .map(|s| s.utilization)
            .fold(0.0f64, |a, b| a.max(b))
            .max(0.1) * 1.1;

        let mut chart = ChartBuilder::on(&root)
            .caption("Station Utilization", ("sans-serif", 24))
            .margin(20)
            .x_label_area_size(60)
            .y_label_area_size(50)
            .build_cartesian_2d(
                0usize..stations.len(),
                0.0..max_util,
            )?;

        chart.configure_mesh()
            .x_desc("Station")
            .y_desc("Utilization")
            .y_label_formatter(&|y| format!("{:.0}%", y * 100.0))
            .draw()?;

        chart.draw_series(
            stations.iter().enumerate().map(|(i, s)| {
                let color = if s.utilization > 0.9 {
                    RED.mix(0.8)
                } else if s.utilization > 0.75 {
                    YELLOW.mix(0.8)
                } else {
                    BLUE.mix(0.8)
                };
                Rectangle::new(
                    [(i, 0.0), (i + 1, s.utilization)],
                    color.filled(),
                )
            }),
        )?;

        root.present()?;
        Ok(())
    }

    /// Generate queue length time series chart
    fn generate_queue_length_chart(
        &self,
        timeseries: &TimeSeriesCollector,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(path, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        // Collect all queue data
        let station_data = timeseries.get_all_station_data();
        if station_data.is_empty() {
            return Ok(());
        }

        // Find max time and max queue length
        let mut max_time = 0.0f64;
        let mut max_queue = 0usize;
        for (_, data) in station_data {
            for point in &data.queue_length {
                max_time = max_time.max(point.time_s);
                max_queue = max_queue.max(point.value);
            }
        }

        if max_time == 0.0 || max_queue == 0 {
            return Ok(());
        }

        let mut chart = ChartBuilder::on(&root)
            .caption("Station Queue Lengths Over Time", ("sans-serif", 24))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                0.0..max_time,
                0usize..(max_queue + 1),
            )?;

        chart.configure_mesh()
            .x_desc("Time (seconds)")
            .y_desc("Queue Length")
            .draw()?;

        // Plot each station's queue length
        let colors = [BLUE, RED, GREEN, CYAN, MAGENTA];
        for (idx, (station_id, data)) in station_data.iter().enumerate() {
            let color = colors[idx % colors.len()];
            let points: Vec<(f64, usize)> = data.queue_length
                .iter()
                .map(|p| (p.time_s, p.value))
                .collect();

            if !points.is_empty() {
                chart.draw_series(LineSeries::new(points, color.stroke_width(2)))?
                    .label(format!("Station {}", station_id.0))
                    .legend(move |(x, y)| Rectangle::new(
                        [(x, y - 5), (x + 10, y + 5)],
                        color.filled(),
                    ));
            }
        }

        chart.configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()?;

        root.present()?;
        Ok(())
    }

    /// Generate throughput over time chart
    fn generate_throughput_chart(
        &self,
        timeseries: &TimeSeriesCollector,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(path, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        // Aggregate throughput across all stations
        let station_data = timeseries.get_all_station_data();
        if station_data.is_empty() {
            return Ok(());
        }

        // Aggregate by time bucket
        let mut throughput_by_time: std::collections::BTreeMap<i64, u32> = std::collections::BTreeMap::new();
        for (_, data) in station_data {
            for point in &data.throughput {
                let bucket = (point.time_s / 60.0) as i64; // Per-minute buckets
                *throughput_by_time.entry(bucket).or_insert(0) += point.value;
            }
        }

        if throughput_by_time.is_empty() {
            return Ok(());
        }

        let max_throughput = *throughput_by_time.values().max().unwrap_or(&1);
        let max_time = *throughput_by_time.keys().max().unwrap_or(&1);

        let mut chart = ChartBuilder::on(&root)
            .caption("Throughput Over Time", ("sans-serif", 24))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                0i64..(max_time + 1),
                0u32..(max_throughput + 1),
            )?;

        chart.configure_mesh()
            .x_desc("Time (minutes)")
            .y_desc("Orders Completed")
            .draw()?;

        let points: Vec<(i64, u32)> = throughput_by_time.into_iter().collect();
        chart.draw_series(LineSeries::new(points, BLUE.stroke_width(2)))?;

        root.present()?;
        Ok(())
    }

    /// Generate congestion ranking chart
    fn generate_congestion_ranking_chart(
        &self,
        rankings: &[crate::timeseries::CongestionRanking],
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(path, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let top_n = rankings.iter().take(10).collect::<Vec<_>>();
        if top_n.is_empty() {
            return Ok(());
        }

        let max_score = top_n.iter()
            .map(|r| r.score)
            .fold(0.0f64, |a, b| a.max(b))
            .max(1.0) * 1.1;

        let mut chart = ChartBuilder::on(&root)
            .caption("Top Congested Locations", ("sans-serif", 24))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(
                0usize..top_n.len(),
                0.0..max_score,
            )?;

        chart.configure_mesh()
            .x_desc("Rank")
            .y_desc("Congestion Score")
            .draw()?;

        chart.draw_series(
            top_n.iter().enumerate().map(|(i, r)| {
                let intensity = (r.score / max_score).min(1.0);
                let color = RGBColor(
                    (255.0 * intensity) as u8,
                    (255.0 * (1.0 - intensity * 0.7)) as u8,
                    0,
                );
                Rectangle::new(
                    [(i, 0.0), (i + 1, r.score)],
                    color.filled(),
                )
            }),
        )?;

        root.present()?;
        Ok(())
    }
}

impl Default for ChartGenerator {
    fn default() -> Self {
        Self::new(ChartConfig::default())
    }
}

/// Generate a simple bar chart as PNG bytes (for embedding in HTML)
pub fn bar_chart_to_png_bytes(
    width: u32,
    height: u32,
    title: &str,
    data: &[(String, f64)],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if data.is_empty() {
        // Return empty buffer for empty data
        return Ok(vec![0u8; (width * height * 3) as usize]);
    }

    let mut buffer = vec![0u8; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let max_val = data.iter().map(|(_, v)| *v).fold(0.0f64, |a, b| a.max(b)).max(0.1) * 1.1;

        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("sans-serif", 20))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(0usize..data.len(), 0.0..max_val)?;

        chart.configure_mesh().draw()?;

        chart.draw_series(
            data.iter().enumerate().map(|(i, (_, v))| {
                Rectangle::new([(i, 0.0), (i + 1, *v)], BLUE.filled())
            }),
        )?;

        root.present()?;
    }
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_config_default() {
        let config = ChartConfig::default();
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
    }
}
