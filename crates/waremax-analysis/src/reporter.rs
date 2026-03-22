//! Report Generation
//!
//! Generates RCA reports in different formats (text, JSON).

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::analyzer::RootCauseAnalysisReport;

/// Output format for reports
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    /// Plain text format
    Text,
    /// JSON format
    Json,
    /// Compact text format (summary only)
    Compact,
}

impl Default for ReportFormat {
    fn default() -> Self {
        ReportFormat::Text
    }
}

impl ReportFormat {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "text" | "txt" => Some(ReportFormat::Text),
            "json" => Some(ReportFormat::Json),
            "compact" | "summary" => Some(ReportFormat::Compact),
            _ => None,
        }
    }
}

/// RCA Report Generator
pub struct RCAReporter {
    format: ReportFormat,
    include_recommendations: bool,
    include_anomalies: bool,
    max_bottlenecks: usize,
    max_anomalies: usize,
}

impl RCAReporter {
    /// Create a new reporter with default settings
    pub fn new() -> Self {
        Self {
            format: ReportFormat::Text,
            include_recommendations: true,
            include_anomalies: true,
            max_bottlenecks: 10,
            max_anomalies: 10,
        }
    }

    /// Set output format
    pub fn with_format(mut self, format: ReportFormat) -> Self {
        self.format = format;
        self
    }

    /// Set whether to include recommendations
    pub fn with_recommendations(mut self, include: bool) -> Self {
        self.include_recommendations = include;
        self
    }

    /// Set whether to include anomalies
    pub fn with_anomalies(mut self, include: bool) -> Self {
        self.include_anomalies = include;
        self
    }

    /// Set maximum bottlenecks to include
    pub fn with_max_bottlenecks(mut self, max: usize) -> Self {
        self.max_bottlenecks = max;
        self
    }

    /// Set maximum anomalies to include
    pub fn with_max_anomalies(mut self, max: usize) -> Self {
        self.max_anomalies = max;
        self
    }

    /// Generate report as string
    pub fn generate(&self, report: &RootCauseAnalysisReport) -> String {
        match self.format {
            ReportFormat::Text => self.generate_text(report),
            ReportFormat::Json => self.generate_json(report),
            ReportFormat::Compact => self.generate_compact(report),
        }
    }

    /// Write report to file
    pub fn write_to_file(
        &self,
        report: &RootCauseAnalysisReport,
        path: &Path,
    ) -> std::io::Result<()> {
        let content = self.generate(report);
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Write report to writer
    pub fn write_to<W: Write>(
        &self,
        report: &RootCauseAnalysisReport,
        writer: &mut W,
    ) -> std::io::Result<()> {
        let content = self.generate(report);
        writer.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Generate detailed text report
    fn generate_text(&self, report: &RootCauseAnalysisReport) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&self.text_header());

        // Summary section
        output.push_str(&self.text_summary(&report.summary));

        // Delay Attribution
        output.push_str(&self.text_delay_attribution(&report.delay_attribution));

        // Bottlenecks
        output.push_str(&self.text_bottlenecks(&report.bottleneck_analysis));

        // Critical Paths
        output.push_str(&self.text_critical_paths(&report.critical_paths));

        // Anomalies
        if self.include_anomalies && !report.anomalies.is_empty() {
            output.push_str(&self.text_anomalies(&report.anomalies));
        }

        // Recommendations
        if self.include_recommendations && !report.recommendations.is_empty() {
            output.push_str(&self.text_recommendations(&report.recommendations));
        }

        // Footer
        output.push_str(&self.text_footer());

        output
    }

    fn text_header(&self) -> String {
        let mut s = String::new();
        s.push_str("\n");
        s.push_str(&"=".repeat(70));
        s.push_str("\n");
        s.push_str("                     ROOT CAUSE ANALYSIS REPORT\n");
        s.push_str(&"=".repeat(70));
        s.push_str("\n\n");
        s
    }

    fn text_footer(&self) -> String {
        let mut s = String::new();
        s.push_str("\n");
        s.push_str(&"=".repeat(70));
        s.push_str("\n                         END OF REPORT\n");
        s.push_str(&"=".repeat(70));
        s.push_str("\n");
        s
    }

    fn text_summary(&self, summary: &crate::analyzer::RCASummary) -> String {
        let mut s = String::new();
        s.push_str("EXECUTIVE SUMMARY\n");
        s.push_str(&"-".repeat(70));
        s.push_str("\n\n");

        s.push_str(&format!(
            "  Health Score:         {:.0}/100",
            summary.health_score
        ));
        if summary.health_score >= 80.0 {
            s.push_str(" (Good)\n");
        } else if summary.health_score >= 60.0 {
            s.push_str(" (Fair)\n");
        } else {
            s.push_str(" (Poor)\n");
        }

        s.push_str(&format!(
            "  Orders Analyzed:      {}\n",
            summary.orders_analyzed
        ));
        s.push_str(&format!(
            "  Avg Cycle Time:       {:.1}s\n",
            summary.avg_cycle_time_s
        ));
        s.push_str(&format!(
            "  Primary Delay Source: {}\n",
            summary.primary_delay_source
        ));

        if let Some(ref bottleneck) = summary.primary_bottleneck {
            s.push_str(&format!("  Primary Bottleneck:   {}\n", bottleneck));
        }

        s.push_str(&format!(
            "  Anomalies Detected:   {}\n",
            summary.anomaly_count
        ));
        s.push_str(&format!(
            "  Slow Orders:          {}\n",
            summary.slow_order_count
        ));

        s.push_str(&format!(
            "\n  Key Finding:\n    {}\n\n",
            summary.key_finding
        ));

        s
    }

    fn text_delay_attribution(&self, attr: &crate::attribution::DelayAttributionSummary) -> String {
        let mut s = String::new();
        s.push_str("DELAY ATTRIBUTION\n");
        s.push_str(&"-".repeat(70));
        s.push_str("\n\n");

        s.push_str(&format!("  Tasks Analyzed:   {}\n", attr.task_count));
        s.push_str(&format!(
            "  Avg Cycle Time:   {:.1}s\n",
            attr.avg_cycle_time_s
        ));
        s.push_str(&format!(
            "  Avg Waste Time:   {:.1}s ({:.1}%)\n\n",
            attr.avg_waste_time_s,
            attr.waste_ratio * 100.0
        ));

        s.push_str("  Time Breakdown:\n");
        s.push_str("  +----------------------+--------+---------+---------+\n");
        s.push_str("  | Category             |  Pct   | Avg (s) | Total   |\n");
        s.push_str("  +----------------------+--------+---------+---------+\n");

        for (cat, total_s, pct) in attr.ranked_categories.iter().take(8) {
            let avg = attr.avg_by_category.get(cat).unwrap_or(&0.0);
            s.push_str(&format!(
                "  | {:<20} | {:>5.1}% | {:>7.1} | {:>7.0} |\n",
                truncate_str(cat.name(), 20),
                pct,
                avg,
                total_s
            ));
        }
        s.push_str("  +----------------------+--------+---------+---------+\n\n");

        s
    }

    fn text_bottlenecks(&self, analysis: &crate::bottleneck::BottleneckAnalysis) -> String {
        let mut s = String::new();
        s.push_str("BOTTLENECK ANALYSIS\n");
        s.push_str(&"-".repeat(70));
        s.push_str("\n\n");

        s.push_str(&format!(
            "  Total Bottlenecks: {}\n",
            analysis.summary.total_count
        ));
        s.push_str(&format!(
            "    Congestion:  {}\n",
            analysis.summary.congestion_count
        ));
        s.push_str(&format!(
            "    Station:     {}\n",
            analysis.summary.station_count
        ));
        s.push_str(&format!(
            "    Robot:       {}\n",
            analysis.summary.robot_count
        ));
        s.push_str(&format!(
            "  Avg Severity:    {:.1}/100\n",
            analysis.summary.avg_severity
        ));
        s.push_str(&format!(
            "  Max Severity:    {:.1}/100\n\n",
            analysis.summary.max_severity
        ));

        if !analysis.bottlenecks.is_empty() {
            s.push_str("  Top Bottlenecks:\n");
            for (i, b) in analysis
                .bottlenecks
                .iter()
                .take(self.max_bottlenecks)
                .enumerate()
            {
                s.push_str(&format!(
                    "  {}. [{:>5.1}] {}: {}\n",
                    i + 1,
                    b.severity(),
                    b.name(),
                    b.to_detail_string()
                ));
            }
            s.push('\n');
        }

        s
    }

    fn text_critical_paths(&self, summary: &crate::critical_path::CriticalPathSummary) -> String {
        let mut s = String::new();
        s.push_str("CRITICAL PATH ANALYSIS\n");
        s.push_str(&"-".repeat(70));
        s.push_str("\n\n");

        s.push_str(&format!(
            "  Orders Analyzed:          {}\n",
            summary.order_count
        ));
        s.push_str(&format!(
            "  Avg Cycle Time:           {:.1}s\n",
            summary.avg_cycle_time_s
        ));
        s.push_str(&format!(
            "  Slow Orders:              {}\n",
            summary.slow_order_count
        ));
        s.push_str(&format!(
            "  Most Common Critical:     {}\n\n",
            summary.most_common_critical_phase.name()
        ));

        if !summary.phases_ranked.is_empty() {
            s.push_str("  Phase Breakdown:\n");
            for (phase, _total_s, pct) in summary.phases_ranked.iter().take(6) {
                let avg = summary.avg_phase_times.get(phase).unwrap_or(&0.0);
                s.push_str(&format!(
                    "    {:<20}: {:>5.1}% ({:.1}s avg)\n",
                    phase.name(),
                    pct,
                    avg
                ));
            }
            s.push('\n');
        }

        s
    }

    fn text_anomalies(&self, anomalies: &[crate::anomaly::Anomaly]) -> String {
        let mut s = String::new();
        s.push_str("ANOMALIES DETECTED\n");
        s.push_str(&"-".repeat(70));
        s.push_str("\n\n");

        s.push_str(&format!("  Total Anomalies: {}\n\n", anomalies.len()));

        if !anomalies.is_empty() {
            s.push_str("  Details:\n");
            for (i, anomaly) in anomalies.iter().take(self.max_anomalies).enumerate() {
                s.push_str(&format!(
                    "  {}. [severity: {:>5.1}] {}\n",
                    i + 1,
                    anomaly.severity,
                    anomaly.anomaly_type.description()
                ));
            }
            if anomalies.len() > self.max_anomalies {
                s.push_str(&format!(
                    "  ... and {} more\n",
                    anomalies.len() - self.max_anomalies
                ));
            }
            s.push('\n');
        }

        s
    }

    fn text_recommendations(&self, recommendations: &[crate::analyzer::Recommendation]) -> String {
        let mut s = String::new();
        s.push_str("RECOMMENDATIONS\n");
        s.push_str(&"-".repeat(70));
        s.push_str("\n\n");

        for rec in recommendations {
            s.push_str(&format!("  [Priority {}] {}\n", rec.priority, rec.category));
            s.push_str(&format!("    Action: {}\n", rec.text));
            s.push_str(&format!("    Impact: {}\n\n", rec.expected_impact));
        }

        s
    }

    /// Generate JSON report
    fn generate_json(&self, report: &RootCauseAnalysisReport) -> String {
        serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate compact summary report
    fn generate_compact(&self, report: &RootCauseAnalysisReport) -> String {
        let mut s = String::new();

        s.push_str("RCA Summary\n");
        s.push_str(&"=".repeat(50));
        s.push_str("\n");

        s.push_str(&format!(
            "Health Score: {:.0}/100\n",
            report.summary.health_score
        ));
        s.push_str(&format!(
            "Orders: {} | Avg Cycle: {:.1}s\n",
            report.summary.orders_analyzed, report.summary.avg_cycle_time_s
        ));
        s.push_str(&format!(
            "Primary Issue: {}\n",
            report.summary.primary_delay_source
        ));

        if let Some(ref bottleneck) = report.summary.primary_bottleneck {
            s.push_str(&format!("Top Bottleneck: {}\n", bottleneck));
        }

        s.push_str(&format!(
            "Anomalies: {} | Slow Orders: {}\n",
            report.summary.anomaly_count, report.summary.slow_order_count
        ));

        s.push_str(&format!("\nFinding: {}\n", report.summary.key_finding));

        if !report.recommendations.is_empty() {
            s.push_str(&format!(
                "\nTop Recommendation: {}\n",
                report.recommendations[0].text
            ));
        }

        s
    }
}

impl Default for RCAReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Truncate string to max length
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::{RCASummary, Recommendation, RootCauseAnalysisReport};
    use crate::attribution::DelayAttributionSummary;
    use crate::bottleneck::{BottleneckAnalysis, BottleneckSummary};
    use crate::critical_path::{CriticalPathSummary, OrderPhase};

    fn create_test_report() -> RootCauseAnalysisReport {
        RootCauseAnalysisReport {
            summary: RCASummary {
                orders_analyzed: 100,
                avg_cycle_time_s: 45.0,
                primary_delay_source: "Travel Time".to_string(),
                primary_bottleneck: Some("Congested Node 42".to_string()),
                anomaly_count: 5,
                slow_order_count: 8,
                health_score: 72.0,
                key_finding: "System operating with moderate congestion".to_string(),
            },
            delay_attribution: DelayAttributionSummary::default(),
            bottleneck_analysis: BottleneckAnalysis {
                bottlenecks: Vec::new(),
                summary: BottleneckSummary {
                    total_count: 3,
                    congestion_count: 2,
                    station_count: 1,
                    robot_count: 0,
                    avg_severity: 45.0,
                    max_severity: 65.0,
                    primary_bottleneck: Some("Test bottleneck".to_string()),
                },
            },
            critical_paths: CriticalPathSummary {
                order_count: 100,
                avg_phase_times: std::collections::HashMap::new(),
                total_phase_times: std::collections::HashMap::new(),
                phase_percentages: std::collections::HashMap::new(),
                most_common_critical_phase: OrderPhase::TravelToStation,
                critical_phase_frequency: std::collections::HashMap::new(),
                avg_cycle_time_s: 45.0,
                avg_deviation_s: Some(5.0),
                slow_order_count: 8,
                phases_ranked: Vec::new(),
            },
            anomalies: Vec::new(),
            recommendations: vec![Recommendation {
                priority: 1,
                category: "Traffic".to_string(),
                text: "Add alternate routes".to_string(),
                expected_impact: "Reduce congestion by 20%".to_string(),
            }],
        }
    }

    #[test]
    fn test_text_generation() {
        let reporter = RCAReporter::new();
        let report = create_test_report();
        let output = reporter.generate(&report);

        assert!(output.contains("ROOT CAUSE ANALYSIS"));
        assert!(output.contains("Health Score"));
        assert!(output.contains("100")); // orders analyzed
    }

    #[test]
    fn test_json_generation() {
        let reporter = RCAReporter::new().with_format(ReportFormat::Json);
        let report = create_test_report();
        let output = reporter.generate(&report);

        // Should be valid JSON
        assert!(output.starts_with("{"));
        assert!(output.contains("\"summary\""));
    }

    #[test]
    fn test_compact_generation() {
        let reporter = RCAReporter::new().with_format(ReportFormat::Compact);
        let report = create_test_report();
        let output = reporter.generate(&report);

        assert!(output.contains("RCA Summary"));
        assert!(output.len() < 1000); // Should be short
    }

    #[test]
    fn test_format_parsing() {
        assert_eq!(ReportFormat::from_str("text"), Some(ReportFormat::Text));
        assert_eq!(ReportFormat::from_str("JSON"), Some(ReportFormat::Json));
        assert_eq!(
            ReportFormat::from_str("compact"),
            Some(ReportFormat::Compact)
        );
        assert_eq!(ReportFormat::from_str("invalid"), None);
    }
}
