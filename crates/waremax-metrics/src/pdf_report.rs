//! PDF report generation
//!
//! v3: Generate PDF reports from simulation data

use crate::report::SimulationReport;
use printpdf::*;
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;

/// PDF report generator
pub struct PdfReportGenerator {
    page_width_mm: f32,
    page_height_mm: f32,
    margin_mm: f32,
    font_size: f32,
}

impl PdfReportGenerator {
    pub fn new() -> Self {
        Self {
            page_width_mm: 210.0, // A4
            page_height_mm: 297.0,
            margin_mm: 20.0,
            font_size: 12.0,
        }
    }

    /// Generate PDF report to a file
    pub fn generate_to_file(
        &self,
        report: &SimulationReport,
        output_path: &Path,
    ) -> io::Result<()> {
        let (doc, page1, layer1) = PdfDocument::new(
            "Waremax Simulation Report",
            Mm(self.page_width_mm),
            Mm(self.page_height_mm),
            "Main Layer",
        );

        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Use built-in font
        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Font error: {:?}", e)))?;
        let font_bold = doc
            .add_builtin_font(BuiltinFont::HelveticaBold)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Font error: {:?}", e)))?;

        let mut y_pos = self.page_height_mm - self.margin_mm;
        let x_pos = self.margin_mm;

        // Title
        current_layer.use_text(
            "Waremax Simulation Report",
            20.0,
            Mm(x_pos),
            Mm(y_pos),
            &font_bold,
        );
        y_pos -= 15.0;

        // Horizontal line
        self.draw_line(
            &current_layer,
            x_pos,
            y_pos,
            self.page_width_mm - self.margin_mm,
            y_pos,
        );
        y_pos -= 10.0;

        // Summary section
        current_layer.use_text("Summary", 14.0, Mm(x_pos), Mm(y_pos), &font_bold);
        y_pos -= 8.0;

        let summary_lines = [
            format!("Duration: {:.2} hours", report.duration_s / 3600.0),
            format!("Events Processed: {}", report.events_processed),
            format!("Orders Completed: {}", report.orders_completed),
            format!("Orders Late: {}", report.orders_late),
            format!("Throughput: {:.1} orders/hour", report.throughput_per_hour),
        ];

        for line in &summary_lines {
            current_layer.use_text(line, self.font_size, Mm(x_pos + 5.0), Mm(y_pos), &font);
            y_pos -= 6.0;
        }
        y_pos -= 5.0;

        // Performance section
        current_layer.use_text("Performance", 14.0, Mm(x_pos), Mm(y_pos), &font_bold);
        y_pos -= 8.0;

        let perf_lines = [
            format!("Average Cycle Time: {:.1} seconds", report.avg_cycle_time_s),
            format!("P95 Cycle Time: {:.1} seconds", report.p95_cycle_time_s),
            format!(
                "Robot Utilization: {:.1}%",
                report.robot_utilization * 100.0
            ),
            format!(
                "Station Utilization: {:.1}%",
                report.station_utilization * 100.0
            ),
        ];

        for line in &perf_lines {
            current_layer.use_text(line, self.font_size, Mm(x_pos + 5.0), Mm(y_pos), &font);
            y_pos -= 6.0;
        }
        y_pos -= 5.0;

        // SLA section if present
        if let Some(ref sla) = report.sla {
            current_layer.use_text("SLA Metrics", 14.0, Mm(x_pos), Mm(y_pos), &font_bold);
            y_pos -= 8.0;

            let sla_lines = [
                format!("On-Time Orders: {}", sla.orders_on_time),
                format!("Late Orders: {}", sla.orders_late),
                format!("SLA Miss Rate: {:.1}%", sla.sla_miss_rate * 100.0),
                format!("Average Lateness: {:.1} seconds", sla.avg_lateness_s),
                format!("P95 Lateness: {:.1} seconds", sla.p95_lateness_s),
            ];

            for line in &sla_lines {
                current_layer.use_text(line, self.font_size, Mm(x_pos + 5.0), Mm(y_pos), &font);
                y_pos -= 6.0;
            }
            y_pos -= 5.0;
        }

        // Congestion section if present
        if let Some(ref congestion) = report.congestion {
            current_layer.use_text("Congestion", 14.0, Mm(x_pos), Mm(y_pos), &font_bold);
            y_pos -= 8.0;

            let cong_lines = [
                format!("Node Wait Events: {}", congestion.total_node_wait_events),
                format!("Edge Wait Events: {}", congestion.total_edge_wait_events),
                format!(
                    "Total Node Wait Time: {:.1} seconds",
                    congestion.total_node_wait_time_s
                ),
                format!(
                    "Total Edge Wait Time: {:.1} seconds",
                    congestion.total_edge_wait_time_s
                ),
            ];

            for line in &cong_lines {
                current_layer.use_text(line, self.font_size, Mm(x_pos + 5.0), Mm(y_pos), &font);
                y_pos -= 6.0;
            }
            y_pos -= 5.0;
        }

        // Battery section if present
        if let Some(ref battery) = report.battery {
            current_layer.use_text("Battery", 14.0, Mm(x_pos), Mm(y_pos), &font_bold);
            y_pos -= 8.0;

            let batt_lines = [
                format!("Charging Events: {}", battery.total_charging_events),
                format!(
                    "Energy Consumed: {:.1} Wh",
                    battery.total_energy_consumed_wh
                ),
                format!(
                    "Total Charging Time: {:.1} seconds",
                    battery.total_charging_time_s
                ),
            ];

            for line in &batt_lines {
                current_layer.use_text(line, self.font_size, Mm(x_pos + 5.0), Mm(y_pos), &font);
                y_pos -= 6.0;
            }
            y_pos -= 5.0;
        }

        // Per-robot summary (just counts)
        if let Some(ref robots) = report.robot_reports {
            current_layer.use_text(
                &format!("Robot Summary ({} robots)", robots.len()),
                14.0,
                Mm(x_pos),
                Mm(y_pos),
                &font_bold,
            );
            y_pos -= 8.0;

            let total_tasks: u32 = robots.iter().map(|r| r.tasks_completed).sum();
            let avg_util: f64 =
                robots.iter().map(|r| r.utilization).sum::<f64>() / robots.len() as f64;
            let total_failures: u32 = robots.iter().map(|r| r.failure_count).sum();

            let robot_lines = [
                format!("Total Tasks Completed: {}", total_tasks),
                format!("Average Utilization: {:.1}%", avg_util * 100.0),
                format!("Total Failures: {}", total_failures),
            ];

            for line in &robot_lines {
                current_layer.use_text(line, self.font_size, Mm(x_pos + 5.0), Mm(y_pos), &font);
                y_pos -= 6.0;
            }
            y_pos -= 5.0;
        }

        // Per-station summary (just counts)
        if let Some(ref stations) = report.station_reports {
            current_layer.use_text(
                &format!("Station Summary ({} stations)", stations.len()),
                14.0,
                Mm(x_pos),
                Mm(y_pos),
                &font_bold,
            );
            y_pos -= 8.0;

            let total_served: u32 = stations.iter().map(|s| s.orders_served).sum();
            let avg_util: f64 =
                stations.iter().map(|s| s.utilization).sum::<f64>() / stations.len() as f64;

            let station_lines = [
                format!("Total Orders Served: {}", total_served),
                format!("Average Utilization: {:.1}%", avg_util * 100.0),
            ];

            for line in &station_lines {
                current_layer.use_text(line, self.font_size, Mm(x_pos + 5.0), Mm(y_pos), &font);
                y_pos -= 6.0;
            }
        }

        // Footer
        y_pos = self.margin_mm + 5.0;
        current_layer.use_text(
            "Generated by Waremax Simulation Engine",
            10.0,
            Mm(x_pos),
            Mm(y_pos),
            &font,
        );

        // Save the document
        let file = File::create(output_path)?;
        let mut buf_writer = BufWriter::new(file);
        doc.save(&mut buf_writer).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("PDF save error: {:?}", e))
        })?;

        Ok(())
    }

    fn draw_line(&self, layer: &PdfLayerReference, x1: f32, y1: f32, x2: f32, y2: f32) {
        let line = Line {
            points: vec![
                (Point::new(Mm(x1), Mm(y1)), false),
                (Point::new(Mm(x2), Mm(y2)), false),
            ],
            is_closed: false,
        };
        layer.add_line(line);
    }
}

impl Default for PdfReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_generator_default() {
        let gen = PdfReportGenerator::new();
        assert_eq!(gen.page_width_mm, 210.0);
        assert_eq!(gen.page_height_mm, 297.0);
    }
}
