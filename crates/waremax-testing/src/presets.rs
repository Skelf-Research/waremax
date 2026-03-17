//! Predefined scenario presets for common testing patterns
//!
//! Provides ready-to-use scenario configurations for quick testing,
//! benchmarking, and stress testing.

use crate::generator::ScenarioBuilder;
use waremax_config::ScenarioConfig;

/// Built-in scenario presets for common testing patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioPreset {
    /// Minimal configuration for unit tests and quick debugging
    /// - 3x3 grid, 1 robot, 1 station
    /// - 10 orders/hr, 1 min duration
    Minimal,

    /// Quick test configuration for fast iteration
    /// - 5x5 grid, 3 robots, 2 stations
    /// - 30 orders/hr, 5 min duration
    Quick,

    /// Standard test configuration for general testing
    /// - 10x10 grid, 10 robots, 4 stations
    /// - 60 orders/hr, 30 min duration
    Standard,

    /// Baseline configuration for reproducible comparisons
    /// - 10x10 grid, 10 robots, 4 stations
    /// - 60 orders/hr, 60 min warmup + 60 min
    Baseline,

    /// High load configuration for stress testing
    /// - 20x20 grid, 50 robots, 10 stations
    /// - 300 orders/hr, 60 min duration
    HighLoad,

    /// Peak hours simulation with high order rate
    /// - 15x15 grid, 20 robots, 6 stations
    /// - 200 orders/hr, 120 min duration
    PeakHours,

    /// Stress test configuration that pushes limits
    /// - 30x30 grid, 100 robots, 20 stations
    /// - 500 orders/hr, 120 min duration
    StressTest,

    /// Battery-focused scenario for charging tests
    /// - 10x10 grid, 15 robots with battery, 4 stations
    /// - 60 orders/hr, 4 charging stations
    BatteryTest,

    /// Maintenance-focused scenario for reliability tests
    /// - 10x10 grid, 20 robots with maintenance/failures
    /// - 60 orders/hr, 2 maintenance stations
    MaintenanceTest,
}

impl ScenarioPreset {
    /// Get a ScenarioBuilder configured for this preset
    pub fn builder(&self) -> ScenarioBuilder {
        match self {
            ScenarioPreset::Minimal => {
                ScenarioBuilder::new()
                    .grid(3, 3)
                    .robots(1)
                    .pick_stations(1)
                    .order_rate(10.0)
                    .duration(1.0)
                    .warmup(0.0)
            }

            ScenarioPreset::Quick => {
                ScenarioBuilder::new()
                    .grid(5, 5)
                    .robots(3)
                    .pick_stations(2)
                    .order_rate(30.0)
                    .duration(5.0)
                    .warmup(1.0)
            }

            ScenarioPreset::Standard => {
                ScenarioBuilder::new()
                    .grid(10, 10)
                    .robots(10)
                    .pick_stations(4)
                    .order_rate(60.0)
                    .duration(30.0)
                    .warmup(5.0)
            }

            ScenarioPreset::Baseline => {
                ScenarioBuilder::new()
                    .grid(10, 10)
                    .robots(10)
                    .pick_stations(4)
                    .order_rate(60.0)
                    .duration(60.0)
                    .warmup(60.0)
                    .seed(12345) // Fixed seed for reproducibility
            }

            ScenarioPreset::HighLoad => {
                ScenarioBuilder::new()
                    .grid(20, 20)
                    .robots(50)
                    .pick_stations(10)
                    .station_concurrency(2)
                    .order_rate(300.0)
                    .items_per_order(4.0)
                    .duration(60.0)
                    .warmup(10.0)
            }

            ScenarioPreset::PeakHours => {
                ScenarioBuilder::new()
                    .grid(15, 15)
                    .robots(20)
                    .pick_stations(6)
                    .station_concurrency(2)
                    .order_rate(200.0)
                    .items_per_order(3.5)
                    .duration(120.0)
                    .warmup(15.0)
                    .service_time_lognormal(8.0, 2.0, 2.0)
            }

            ScenarioPreset::StressTest => {
                ScenarioBuilder::new()
                    .grid(30, 30)
                    .robots(100)
                    .pick_stations(20)
                    .station_concurrency(3)
                    .order_rate(500.0)
                    .items_per_order(5.0)
                    .sku_count(500)
                    .duration(120.0)
                    .warmup(20.0)
                    .traffic_policy("reroute_on_wait")
                    .routing_algorithm("astar")
                    .congestion_aware(true)
            }

            ScenarioPreset::BatteryTest => {
                ScenarioBuilder::new()
                    .grid(10, 10)
                    .robots(15)
                    .pick_stations(4)
                    .order_rate(60.0)
                    .duration(120.0)
                    .warmup(10.0)
                    .robot_battery(400.0, 0.15)
                    .charging_stations(4, 2, 200.0)
            }

            ScenarioPreset::MaintenanceTest => {
                ScenarioBuilder::new()
                    .grid(10, 10)
                    .robots(20)
                    .pick_stations(4)
                    .order_rate(60.0)
                    .duration(480.0) // 8 hours to see maintenance cycles
                    .warmup(30.0)
                    .enable_maintenance(8.0) // Maintenance every 8 hours
                    .enable_failures(50.0) // MTBF of 50 hours
                    .maintenance_stations(2, 2)
            }
        }
    }

    /// Get a ScenarioConfig configured for this preset
    pub fn config(&self) -> ScenarioConfig {
        self.builder().build()
    }

    /// Get the preset name as a string
    pub fn name(&self) -> &'static str {
        match self {
            ScenarioPreset::Minimal => "minimal",
            ScenarioPreset::Quick => "quick",
            ScenarioPreset::Standard => "standard",
            ScenarioPreset::Baseline => "baseline",
            ScenarioPreset::HighLoad => "high_load",
            ScenarioPreset::PeakHours => "peak_hours",
            ScenarioPreset::StressTest => "stress_test",
            ScenarioPreset::BatteryTest => "battery_test",
            ScenarioPreset::MaintenanceTest => "maintenance_test",
        }
    }

    /// Get a preset by name
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "minimal" => Some(ScenarioPreset::Minimal),
            "quick" => Some(ScenarioPreset::Quick),
            "standard" => Some(ScenarioPreset::Standard),
            "baseline" => Some(ScenarioPreset::Baseline),
            "high_load" | "highload" => Some(ScenarioPreset::HighLoad),
            "peak_hours" | "peakhours" => Some(ScenarioPreset::PeakHours),
            "stress_test" | "stresstest" | "stress" => Some(ScenarioPreset::StressTest),
            "battery_test" | "batterytest" | "battery" => Some(ScenarioPreset::BatteryTest),
            "maintenance_test" | "maintenancetest" | "maintenance" => Some(ScenarioPreset::MaintenanceTest),
            _ => None,
        }
    }

    /// Get all available presets
    pub fn all() -> Vec<ScenarioPreset> {
        vec![
            ScenarioPreset::Minimal,
            ScenarioPreset::Quick,
            ScenarioPreset::Standard,
            ScenarioPreset::Baseline,
            ScenarioPreset::HighLoad,
            ScenarioPreset::PeakHours,
            ScenarioPreset::StressTest,
            ScenarioPreset::BatteryTest,
            ScenarioPreset::MaintenanceTest,
        ]
    }

    /// Get a description of this preset
    pub fn description(&self) -> &'static str {
        match self {
            ScenarioPreset::Minimal => "Minimal config for unit tests (3x3 grid, 1 robot, 1 station)",
            ScenarioPreset::Quick => "Quick iteration testing (5x5 grid, 3 robots, 5 min)",
            ScenarioPreset::Standard => "Standard testing (10x10 grid, 10 robots, 30 min)",
            ScenarioPreset::Baseline => "Reproducible baseline (10x10, fixed seed, 60 min warmup)",
            ScenarioPreset::HighLoad => "High load stress test (20x20, 50 robots, 300 orders/hr)",
            ScenarioPreset::PeakHours => "Peak hours simulation (15x15, 20 robots, 200 orders/hr)",
            ScenarioPreset::StressTest => "Maximum stress test (30x30, 100 robots, 500 orders/hr)",
            ScenarioPreset::BatteryTest => "Battery/charging focused (15 robots, 4 chargers)",
            ScenarioPreset::MaintenanceTest => "Maintenance/reliability focused (8 hour sim)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_presets_build_successfully() {
        for preset in ScenarioPreset::all() {
            let config = preset.config();
            assert!(config.robots.count > 0, "Preset {:?} should have robots", preset);
            assert!(!config.stations.is_empty(), "Preset {:?} should have stations", preset);
        }
    }

    #[test]
    fn test_preset_from_name() {
        assert_eq!(ScenarioPreset::from_name("quick"), Some(ScenarioPreset::Quick));
        assert_eq!(ScenarioPreset::from_name("QUICK"), Some(ScenarioPreset::Quick));
        assert_eq!(ScenarioPreset::from_name("stress_test"), Some(ScenarioPreset::StressTest));
        assert_eq!(ScenarioPreset::from_name("stresstest"), Some(ScenarioPreset::StressTest));
        assert_eq!(ScenarioPreset::from_name("invalid"), None);
    }

    #[test]
    fn test_preset_names_roundtrip() {
        for preset in ScenarioPreset::all() {
            let name = preset.name();
            let recovered = ScenarioPreset::from_name(name);
            assert_eq!(recovered, Some(preset), "Failed roundtrip for {:?}", preset);
        }
    }

    #[test]
    fn test_baseline_has_fixed_seed() {
        let config = ScenarioPreset::Baseline.config();
        assert_eq!(config.seed, 12345);
    }

    #[test]
    fn test_stress_test_has_many_robots() {
        let config = ScenarioPreset::StressTest.config();
        assert_eq!(config.robots.count, 100);
        assert!(config.stations.len() >= 20);
    }

    #[test]
    fn test_battery_test_has_charging() {
        let config = ScenarioPreset::BatteryTest.config();
        assert!(config.robots.battery.enabled);
        assert!(!config.charging_stations.is_empty());
    }

    #[test]
    fn test_maintenance_test_has_maintenance() {
        let config = ScenarioPreset::MaintenanceTest.config();
        assert!(config.robots.maintenance.enabled);
        assert!(config.robots.failure.enabled);
        assert!(!config.maintenance_stations.is_empty());
    }
}
