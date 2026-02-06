//! HTML report generation
//!
//! v3: Generate interactive HTML reports from simulation data

use crate::charts::{ChartConfig, ChartGenerator};
use crate::report::SimulationReport;
use crate::timeseries::TimeSeriesCollector;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use std::io;
use std::path::Path;
use tera::{Context, Tera};

/// HTML report generator
pub struct HtmlReportGenerator {
    template: String,
    include_charts: bool,
}

impl HtmlReportGenerator {
    pub fn new() -> Self {
        Self {
            template: DEFAULT_TEMPLATE.to_string(),
            include_charts: true,
        }
    }

    /// Set whether to include embedded charts
    pub fn with_charts(mut self, include: bool) -> Self {
        self.include_charts = include;
        self
    }

    /// Generate HTML report to a file
    pub fn generate_to_file(
        &self,
        report: &SimulationReport,
        timeseries: Option<&TimeSeriesCollector>,
        output_path: &Path,
    ) -> io::Result<()> {
        let html = self.generate(report, timeseries)?;
        std::fs::write(output_path, html)
    }

    /// Generate HTML report as a string
    pub fn generate(
        &self,
        report: &SimulationReport,
        timeseries: Option<&TimeSeriesCollector>,
    ) -> io::Result<String> {
        let mut tera = Tera::default();
        tera.add_raw_template("report", &self.template)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut context = Context::new();

        // Basic metrics
        context.insert("duration_hours", &(report.duration_s / 3600.0));
        context.insert("events_processed", &report.events_processed);
        context.insert("orders_completed", &report.orders_completed);
        context.insert("orders_late", &report.orders_late);
        context.insert(
            "throughput_per_hour",
            &format!("{:.1}", report.throughput_per_hour),
        );
        context.insert(
            "avg_cycle_time_s",
            &format!("{:.1}", report.avg_cycle_time_s),
        );
        context.insert(
            "p95_cycle_time_s",
            &format!("{:.1}", report.p95_cycle_time_s),
        );
        context.insert(
            "robot_utilization",
            &format!("{:.1}", report.robot_utilization * 100.0),
        );
        context.insert(
            "station_utilization",
            &format!("{:.1}", report.station_utilization * 100.0),
        );

        // SLA section
        if let Some(ref sla) = report.sla {
            context.insert("has_sla", &true);
            context.insert("sla_on_time", &sla.orders_on_time);
            context.insert("sla_late", &sla.orders_late);
            context.insert(
                "sla_miss_rate",
                &format!("{:.1}", sla.sla_miss_rate * 100.0),
            );
            context.insert("sla_avg_lateness", &format!("{:.1}", sla.avg_lateness_s));
        } else {
            context.insert("has_sla", &false);
        }

        // Congestion section
        if let Some(ref congestion) = report.congestion {
            context.insert("has_congestion", &true);
            context.insert("congestion_node_events", &congestion.total_node_wait_events);
            context.insert("congestion_edge_events", &congestion.total_edge_wait_events);
            context.insert(
                "congestion_node_time",
                &format!("{:.1}", congestion.total_node_wait_time_s),
            );
            context.insert(
                "congestion_edge_time",
                &format!("{:.1}", congestion.total_edge_wait_time_s),
            );
        } else {
            context.insert("has_congestion", &false);
        }

        // Battery section
        if let Some(ref battery) = report.battery {
            context.insert("has_battery", &true);
            context.insert("battery_events", &battery.total_charging_events);
            context.insert(
                "battery_energy",
                &format!("{:.1}", battery.total_energy_consumed_wh),
            );
            context.insert(
                "battery_charge_time",
                &format!("{:.1}", battery.total_charging_time_s),
            );
        } else {
            context.insert("has_battery", &false);
        }

        // Robot reports
        if let Some(ref robots) = report.robot_reports {
            context.insert("has_robots", &true);
            let robot_data: Vec<_> = robots
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.robot_id,
                        "tasks": r.tasks_completed,
                        "distance": format!("{:.0}", r.distance_traveled_m),
                        "utilization": format!("{:.1}", r.utilization * 100.0),
                        "failures": r.failure_count,
                    })
                })
                .collect();
            context.insert("robots", &robot_data);
        } else {
            context.insert("has_robots", &false);
        }

        // Station reports
        if let Some(ref stations) = report.station_reports {
            context.insert("has_stations", &true);
            let station_data: Vec<_> = stations
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "id": s.string_id,
                        "type": s.station_type,
                        "served": s.orders_served,
                        "avg_queue": format!("{:.1}", s.avg_queue_length),
                        "utilization": format!("{:.1}", s.utilization * 100.0),
                    })
                })
                .collect();
            context.insert("stations", &station_data);
        } else {
            context.insert("has_stations", &false);
        }

        // Generate charts if enabled and timeseries data available
        if self.include_charts {
            // Chart data is generated for inline Chart.js (using JS library, not plotters)

            // Generate robot utilization chart data for inline Chart.js
            if let Some(ref robots) = report.robot_reports {
                let labels: Vec<String> =
                    robots.iter().map(|r| format!("R{}", r.robot_id)).collect();
                let values: Vec<f64> = robots.iter().map(|r| r.utilization * 100.0).collect();
                context.insert(
                    "robot_chart_labels",
                    &serde_json::to_string(&labels).unwrap_or_default(),
                );
                context.insert(
                    "robot_chart_data",
                    &serde_json::to_string(&values).unwrap_or_default(),
                );
                context.insert("has_robot_chart", &true);
            } else {
                context.insert("has_robot_chart", &false);
            }

            // Generate station utilization chart data
            if let Some(ref stations) = report.station_reports {
                let labels: Vec<String> = stations.iter().map(|s| s.string_id.clone()).collect();
                let values: Vec<f64> = stations.iter().map(|s| s.utilization * 100.0).collect();
                context.insert(
                    "station_chart_labels",
                    &serde_json::to_string(&labels).unwrap_or_default(),
                );
                context.insert(
                    "station_chart_data",
                    &serde_json::to_string(&values).unwrap_or_default(),
                );
                context.insert("has_station_chart", &true);
            } else {
                context.insert("has_station_chart", &false);
            }

            // Queue length time series for Chart.js
            if let Some(ts) = timeseries {
                let station_data = ts.get_all_station_data();
                if !station_data.is_empty() {
                    let mut all_times: Vec<f64> = Vec::new();
                    let mut datasets: Vec<serde_json::Value> = Vec::new();
                    let colors = ["#3498db", "#e74c3c", "#2ecc71", "#9b59b6", "#f1c40f"];

                    for (idx, (station_id, data)) in station_data.iter().enumerate() {
                        let times: Vec<f64> =
                            data.queue_length.iter().map(|p| p.time_s / 60.0).collect();
                        let values: Vec<usize> =
                            data.queue_length.iter().map(|p| p.value).collect();
                        all_times.extend(times.clone());

                        datasets.push(serde_json::json!({
                            "label": format!("Station {}", station_id.0),
                            "data": values,
                            "borderColor": colors[idx % colors.len()],
                            "fill": false,
                        }));
                    }

                    context.insert(
                        "queue_chart_labels",
                        &serde_json::to_string(&all_times).unwrap_or_default(),
                    );
                    context.insert(
                        "queue_chart_datasets",
                        &serde_json::to_string(&datasets).unwrap_or_default(),
                    );
                    context.insert("has_queue_chart", &true);
                } else {
                    context.insert("has_queue_chart", &false);
                }
            } else {
                context.insert("has_queue_chart", &false);
            }
        } else {
            context.insert("has_robot_chart", &false);
            context.insert("has_station_chart", &false);
            context.insert("has_queue_chart", &false);
        }

        tera.render("report", &context)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

impl Default for HtmlReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Default HTML template
const DEFAULT_TEMPLATE: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Waremax Simulation Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        :root {
            --bg-color: #f5f5f5;
            --card-bg: #ffffff;
            --text-color: #333333;
            --border-color: #e0e0e0;
            --primary-color: #3498db;
            --success-color: #2ecc71;
            --warning-color: #f1c40f;
            --danger-color: #e74c3c;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: var(--bg-color);
            color: var(--text-color);
            line-height: 1.6;
            padding: 20px;
        }
        .container { max-width: 1200px; margin: 0 auto; }
        h1 { color: var(--primary-color); margin-bottom: 20px; }
        h2 { color: var(--text-color); margin: 20px 0 10px; border-bottom: 2px solid var(--primary-color); padding-bottom: 5px; }
        .card {
            background: var(--card-bg);
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; }
        .metric { text-align: center; padding: 15px; }
        .metric-value { font-size: 2em; font-weight: bold; color: var(--primary-color); }
        .metric-label { color: #666; font-size: 0.9em; }
        table { width: 100%; border-collapse: collapse; margin: 10px 0; }
        th, td { padding: 10px; text-align: left; border-bottom: 1px solid var(--border-color); }
        th { background: var(--bg-color); font-weight: 600; }
        tr:hover { background: var(--bg-color); }
        .chart-container { position: relative; height: 300px; margin: 20px 0; }
        .status-good { color: var(--success-color); }
        .status-warn { color: var(--warning-color); }
        .status-bad { color: var(--danger-color); }
        footer { text-align: center; padding: 20px; color: #666; font-size: 0.9em; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Waremax Simulation Report</h1>

        <div class="card">
            <h2>Summary</h2>
            <div class="grid">
                <div class="metric">
                    <div class="metric-value">{{ duration_hours | round(precision=2) }}h</div>
                    <div class="metric-label">Duration</div>
                </div>
                <div class="metric">
                    <div class="metric-value">{{ orders_completed }}</div>
                    <div class="metric-label">Orders Completed</div>
                </div>
                <div class="metric">
                    <div class="metric-value">{{ throughput_per_hour }}</div>
                    <div class="metric-label">Orders/Hour</div>
                </div>
                <div class="metric">
                    <div class="metric-value">{{ avg_cycle_time_s }}s</div>
                    <div class="metric-label">Avg Cycle Time</div>
                </div>
            </div>
        </div>

        <div class="card">
            <h2>Utilization</h2>
            <div class="grid">
                <div class="metric">
                    <div class="metric-value">{{ robot_utilization }}%</div>
                    <div class="metric-label">Robot Utilization</div>
                </div>
                <div class="metric">
                    <div class="metric-value">{{ station_utilization }}%</div>
                    <div class="metric-label">Station Utilization</div>
                </div>
                <div class="metric">
                    <div class="metric-value">{{ p95_cycle_time_s }}s</div>
                    <div class="metric-label">P95 Cycle Time</div>
                </div>
                <div class="metric">
                    <div class="metric-value">{{ orders_late }}</div>
                    <div class="metric-label">Late Orders</div>
                </div>
            </div>
        </div>

        {% if has_sla %}
        <div class="card">
            <h2>SLA Performance</h2>
            <div class="grid">
                <div class="metric">
                    <div class="metric-value status-good">{{ sla_on_time }}</div>
                    <div class="metric-label">On-Time Orders</div>
                </div>
                <div class="metric">
                    <div class="metric-value status-bad">{{ sla_late }}</div>
                    <div class="metric-label">Late Orders</div>
                </div>
                <div class="metric">
                    <div class="metric-value">{{ sla_miss_rate }}%</div>
                    <div class="metric-label">SLA Miss Rate</div>
                </div>
                <div class="metric">
                    <div class="metric-value">{{ sla_avg_lateness }}s</div>
                    <div class="metric-label">Avg Lateness</div>
                </div>
            </div>
        </div>
        {% endif %}

        {% if has_congestion %}
        <div class="card">
            <h2>Congestion</h2>
            <div class="grid">
                <div class="metric">
                    <div class="metric-value">{{ congestion_node_events }}</div>
                    <div class="metric-label">Node Wait Events</div>
                </div>
                <div class="metric">
                    <div class="metric-value">{{ congestion_edge_events }}</div>
                    <div class="metric-label">Edge Wait Events</div>
                </div>
                <div class="metric">
                    <div class="metric-value">{{ congestion_node_time }}s</div>
                    <div class="metric-label">Total Node Wait</div>
                </div>
                <div class="metric">
                    <div class="metric-value">{{ congestion_edge_time }}s</div>
                    <div class="metric-label">Total Edge Wait</div>
                </div>
            </div>
        </div>
        {% endif %}

        {% if has_robot_chart %}
        <div class="card">
            <h2>Robot Utilization Chart</h2>
            <div class="chart-container">
                <canvas id="robotChart"></canvas>
            </div>
        </div>
        {% endif %}

        {% if has_station_chart %}
        <div class="card">
            <h2>Station Utilization Chart</h2>
            <div class="chart-container">
                <canvas id="stationChart"></canvas>
            </div>
        </div>
        {% endif %}

        {% if has_robots %}
        <div class="card">
            <h2>Per-Robot Performance</h2>
            <table>
                <thead>
                    <tr>
                        <th>Robot</th>
                        <th>Tasks</th>
                        <th>Distance (m)</th>
                        <th>Utilization</th>
                        <th>Failures</th>
                    </tr>
                </thead>
                <tbody>
                {% for robot in robots %}
                    <tr>
                        <td>Robot {{ robot.id }}</td>
                        <td>{{ robot.tasks }}</td>
                        <td>{{ robot.distance }}</td>
                        <td>{{ robot.utilization }}%</td>
                        <td>{{ robot.failures }}</td>
                    </tr>
                {% endfor %}
                </tbody>
            </table>
        </div>
        {% endif %}

        {% if has_stations %}
        <div class="card">
            <h2>Per-Station Performance</h2>
            <table>
                <thead>
                    <tr>
                        <th>Station</th>
                        <th>Type</th>
                        <th>Orders Served</th>
                        <th>Avg Queue</th>
                        <th>Utilization</th>
                    </tr>
                </thead>
                <tbody>
                {% for station in stations %}
                    <tr>
                        <td>{{ station.id }}</td>
                        <td>{{ station.type }}</td>
                        <td>{{ station.served }}</td>
                        <td>{{ station.avg_queue }}</td>
                        <td>{{ station.utilization }}%</td>
                    </tr>
                {% endfor %}
                </tbody>
            </table>
        </div>
        {% endif %}

        <footer>
            Generated by Waremax Simulation Engine
        </footer>
    </div>

    <script>
    {% if has_robot_chart %}
    new Chart(document.getElementById('robotChart'), {
        type: 'bar',
        data: {
            labels: {{ robot_chart_labels | safe }},
            datasets: [{
                label: 'Utilization %',
                data: {{ robot_chart_data | safe }},
                backgroundColor: 'rgba(52, 152, 219, 0.7)',
                borderColor: 'rgba(52, 152, 219, 1)',
                borderWidth: 1
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: { y: { beginAtZero: true, max: 100 } }
        }
    });
    {% endif %}

    {% if has_station_chart %}
    new Chart(document.getElementById('stationChart'), {
        type: 'bar',
        data: {
            labels: {{ station_chart_labels | safe }},
            datasets: [{
                label: 'Utilization %',
                data: {{ station_chart_data | safe }},
                backgroundColor: 'rgba(46, 204, 113, 0.7)',
                borderColor: 'rgba(46, 204, 113, 1)',
                borderWidth: 1
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: { y: { beginAtZero: true, max: 100 } }
        }
    });
    {% endif %}
    </script>
</body>
</html>
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_generator_default() {
        let gen = HtmlReportGenerator::new();
        let report = SimulationReport::default();
        let result = gen.generate(&report, None);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("Waremax Simulation Report"));
    }
}
