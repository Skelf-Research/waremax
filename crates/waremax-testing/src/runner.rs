//! Batch simulation runner with parallel execution
//!
//! Provides BatchRunner for running multiple simulations in parallel
//! using rayon.

use rayon::prelude::*;
use std::time::Instant;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use waremax_config::ScenarioConfig;
use waremax_metrics::SimulationReport;
use waremax_sim::{SimulationRunner, World};
use waremax_core::{NodeId, EdgeId, RobotId, StationId};
use waremax_entities::{Robot, Station, StationType, ServiceTimeModel, ChargingStation, BatteryConsumptionModel, MaintenanceStation};
use waremax_map::{WarehouseMap, Node, Edge, NodeType, Router, TrafficManager};
use waremax_metrics::TimeSeriesCollector;

/// Result of a single simulation run
#[derive(Clone)]
pub struct RunResult {
    /// Label identifying this run
    pub label: String,
    /// Hash of the config for comparison
    pub config_hash: u64,
    /// Random seed used
    pub seed: u64,
    /// Simulation report
    pub report: SimulationReport,
    /// Wall-clock time to run in milliseconds
    pub duration_ms: u64,
}

impl RunResult {
    /// Get throughput (orders completed per hour)
    pub fn throughput(&self) -> f64 {
        let duration_hours = self.report.duration_s / 3600.0;
        if duration_hours > 0.0 {
            self.report.orders_completed as f64 / duration_hours
        } else {
            0.0
        }
    }

    /// Get P95 cycle time
    pub fn p95_cycle_time(&self) -> f64 {
        self.report.p95_cycle_time_s
    }

    /// Get robot utilization
    pub fn robot_utilization(&self) -> f64 {
        self.report.robot_utilization
    }

    /// Get station utilization
    pub fn station_utilization(&self) -> f64 {
        self.report.station_utilization
    }
}

/// Runs multiple simulations in parallel
pub struct BatchRunner {
    scenarios: Vec<(String, ScenarioConfig)>,
    parallelism: Option<usize>,
}

impl BatchRunner {
    /// Create a new BatchRunner with the given scenarios
    pub fn new(scenarios: Vec<(String, ScenarioConfig)>) -> Self {
        Self {
            scenarios,
            parallelism: None,
        }
    }

    /// Set the number of parallel threads to use
    pub fn parallelism(mut self, n: usize) -> Self {
        self.parallelism = Some(n);
        self
    }

    /// Run all scenarios in parallel
    pub fn run(&self) -> Vec<RunResult> {
        // Configure thread pool if custom parallelism specified
        if let Some(n) = self.parallelism {
            rayon::ThreadPoolBuilder::new()
                .num_threads(n)
                .build_global()
                .ok(); // Ignore error if pool already initialized
        }

        self.scenarios
            .par_iter()
            .map(|(label, config)| {
                let start = Instant::now();
                let report = run_simulation_from_config(config);
                let duration_ms = start.elapsed().as_millis() as u64;

                let config_hash = hash_config(config);

                RunResult {
                    label: label.clone(),
                    config_hash,
                    seed: config.seed,
                    report,
                    duration_ms,
                }
            })
            .collect()
    }

    /// Run scenarios with multiple replications (different seeds)
    pub fn run_with_replications(&self, seeds: &[u64]) -> Vec<RunResult> {
        // Create all scenario/seed combinations
        let mut all_scenarios: Vec<(String, ScenarioConfig)> = Vec::new();

        for (label, config) in &self.scenarios {
            for &seed in seeds {
                let mut config_with_seed = config.clone();
                config_with_seed.seed = seed;
                let new_label = format!("{}_seed={}", label, seed);
                all_scenarios.push((new_label, config_with_seed));
            }
        }

        // Run in parallel
        all_scenarios
            .par_iter()
            .map(|(label, config)| {
                let start = Instant::now();
                let report = run_simulation_from_config(config);
                let duration_ms = start.elapsed().as_millis() as u64;

                let config_hash = hash_config(config);

                RunResult {
                    label: label.clone(),
                    config_hash,
                    seed: config.seed,
                    report,
                    duration_ms,
                }
            })
            .collect()
    }

    /// Get the number of scenarios
    pub fn scenario_count(&self) -> usize {
        self.scenarios.len()
    }
}

/// Run a simulation from a ScenarioConfig and return the report
pub fn run_simulation_from_config(config: &ScenarioConfig) -> SimulationReport {
    let world = build_world_from_config(config);
    let mut runner = SimulationRunner::new(
        world,
        config.simulation.duration_minutes,
        config.simulation.warmup_minutes,
    );
    runner.run()
}

/// Build a World from ScenarioConfig
pub fn build_world_from_config(scenario: &ScenarioConfig) -> World {
    let mut world = World::new(scenario.seed);

    // Determine grid size from builder hints or use default
    // The ScenarioBuilder puts "inline" in map.file when using inline generation
    let grid_size = if scenario.map.file == "inline" {
        // Try to infer from stations or use default
        let max_station_node: u32 = scenario.stations.iter()
            .filter_map(|s| s.node.parse::<u32>().ok())
            .max()
            .unwrap_or(24);
        ((max_station_node as f64).sqrt().ceil() as u32).max(5)
    } else {
        10 // Default grid size
    };
    let spacing = 3.0;

    // Build grid map
    let mut map = WarehouseMap::new();

    for row in 0..grid_size {
        for col in 0..grid_size {
            let id = row * grid_size + col;
            let x = col as f64 * spacing;
            let y = row as f64 * spacing;
            let node_type = if id == 0 { NodeType::StationPick } else { NodeType::Aisle };
            let node = Node::new(NodeId(id), format!("N{}", id), x, y, node_type);
            map.add_node(node);
        }
    }

    let mut edge_id = 0u32;
    for row in 0..grid_size {
        for col in 0..grid_size {
            let id = row * grid_size + col;
            if col < grid_size - 1 {
                let neighbor = id + 1;
                map.add_edge(
                    Edge::new(EdgeId(edge_id), NodeId(id), NodeId(neighbor), spacing),
                );
                edge_id += 1;
            }
            if row < grid_size - 1 {
                let neighbor = id + grid_size;
                map.add_edge(
                    Edge::new(EdgeId(edge_id), NodeId(id), NodeId(neighbor), spacing),
                );
                edge_id += 1;
            }
        }
    }

    world.map = map;
    world.router = Router::new(true);
    world.traffic = TrafficManager::new(
        scenario.traffic.node_capacity_default,
        scenario.traffic.edge_capacity_default,
    );

    let total_nodes = grid_size * grid_size;

    // Add robots
    for i in 0..scenario.robots.count {
        let start_node = i % total_nodes;
        let mut robot = if scenario.robots.battery.enabled {
            Robot::with_battery(
                RobotId(i),
                NodeId(start_node),
                scenario.robots.max_speed_mps,
                scenario.robots.max_payload_kg,
                scenario.robots.battery.capacity_wh,
                scenario.robots.battery.min_soc,
                BatteryConsumptionModel {
                    per_meter_wh: scenario.robots.battery.consumption.per_meter_wh,
                    per_kg_per_meter_wh: scenario.robots.battery.consumption.per_kg_per_meter_wh,
                    idle_power_w: scenario.robots.battery.consumption.idle_power_w,
                    service_power_w: scenario.robots.battery.consumption.service_power_w,
                },
            )
        } else {
            Robot::new(
                RobotId(i),
                NodeId(start_node),
                scenario.robots.max_speed_mps,
                scenario.robots.max_payload_kg,
            )
        };

        // Enable maintenance if configured
        if scenario.robots.maintenance.enabled {
            robot.enable_maintenance(scenario.robots.maintenance.interval_hours * 3600.0);
        }

        // Enable random failures if configured
        if scenario.robots.failure.enabled {
            robot.enable_failures(scenario.robots.failure.mtbf_hours * 3600.0);
        }

        world.robots.insert(RobotId(i), robot);
    }

    // Add stations from config
    for (idx, station_cfg) in scenario.stations.iter().enumerate() {
        let station_type = match station_cfg.station_type.as_str() {
            "pick" => StationType::Pick,
            "drop" => StationType::Drop,
            "inbound" => StationType::Inbound,
            "outbound" => StationType::Outbound,
            _ => StationType::Pick,
        };

        let service_time = match station_cfg.service_time_s.distribution.as_str() {
            "lognormal" => ServiceTimeModel::lognormal(
                station_cfg.service_time_s.base,
                station_cfg.service_time_s.base_stddev,
                station_cfg.service_time_s.per_item,
                station_cfg.service_time_s.per_item_stddev,
            ),
            "exponential" => ServiceTimeModel::exponential(
                station_cfg.service_time_s.base,
            ),
            "uniform" => ServiceTimeModel::uniform(
                station_cfg.service_time_s.min_s,
                station_cfg.service_time_s.max_s,
                station_cfg.service_time_s.per_item,
            ),
            _ => ServiceTimeModel::constant(
                station_cfg.service_time_s.base,
                station_cfg.service_time_s.per_item,
            ),
        };

        let node_id: u32 = station_cfg.node.parse().unwrap_or(idx as u32 % total_nodes);
        let station = Station::new(
            StationId(idx as u32),
            station_cfg.id.clone(),
            NodeId(node_id),
            station_type,
            station_cfg.concurrency,
            None, // queue_capacity
            service_time,
        );
        world.stations.insert(StationId(idx as u32), station);
    }

    // Add charging stations
    for (idx, cfg) in scenario.charging_stations.iter().enumerate() {
        let node_id: u32 = cfg.node.parse().unwrap_or((total_nodes - 1 - idx as u32) % total_nodes);
        let station_id = world.next_charging_id();
        let mut station = ChargingStation::new(
            station_id,
            cfg.id.clone(),
            NodeId(node_id),
            cfg.bays,
            cfg.charge_rate_w,
        );
        if let Some(capacity) = cfg.queue_capacity {
            station = station.with_queue_capacity(capacity);
        }
        world.charging_stations.insert(station_id, station);
    }

    // Add maintenance stations
    for (idx, cfg) in scenario.maintenance_stations.iter().enumerate() {
        let node_id: u32 = cfg.node.parse().unwrap_or((total_nodes / 2 + idx as u32) % total_nodes);
        let station_id = world.next_maintenance_id();

        let repair_time_model = match cfg.repair_time.distribution.as_str() {
            "lognormal" => ServiceTimeModel::lognormal(
                cfg.repair_time.base,
                cfg.repair_time.base_stddev,
                cfg.repair_time.per_item,
                cfg.repair_time.per_item_stddev,
            ),
            _ => ServiceTimeModel::constant(cfg.repair_time.base, 0.0),
        };

        let mut station = MaintenanceStation::new(
            station_id,
            cfg.id.clone(),
            NodeId(node_id),
            cfg.bays,
            cfg.maintenance_duration_s,
        )
        .with_repair_time_model(repair_time_model);
        if let Some(capacity) = cfg.queue_capacity {
            station = station.with_queue_capacity(capacity);
        }
        world.maintenance_stations.insert(station_id, station);
    }

    // Set up policies
    world.policies = waremax_sim::create_policies_with_traffic(
        &scenario.policies,
        &scenario.traffic,
    );

    // Set up distributions
    world.distributions = waremax_sim::create_distributions(&scenario.orders);

    // Initialize demo inventory based on SKU count from orders config
    // Use the SKU count from zipf distribution or default to 100
    world.init_demo_inventory(100);

    // Set metrics sample interval
    world.metrics_sample_interval_s = scenario.metrics.sample_interval_s;
    world.time_series = TimeSeriesCollector::new(scenario.metrics.sample_interval_s);

    // Set due time offset if configured
    if let Some(ref due_cfg) = scenario.orders.due_times {
        world.due_time_offset_min = Some(due_cfg.minutes);
    }

    world
}

/// Hash a ScenarioConfig for comparison purposes
fn hash_config(config: &ScenarioConfig) -> u64 {
    let mut hasher = DefaultHasher::new();

    // Hash key configuration values
    config.robots.count.hash(&mut hasher);
    config.stations.len().hash(&mut hasher);
    config.simulation.duration_minutes.to_bits().hash(&mut hasher);
    config.policies.task_allocation.alloc_type.hash(&mut hasher);

    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets::ScenarioPreset;

    #[test]
    fn test_run_simulation_minimal() {
        let config = ScenarioPreset::Minimal.config();
        let report = run_simulation_from_config(&config);

        assert!(report.duration_s > 0.0);
    }

    #[test]
    fn test_batch_runner_single() {
        let config = ScenarioPreset::Minimal.config();
        let runner = BatchRunner::new(vec![("test".to_string(), config)]);

        let results = runner.run();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "test");
    }

    #[test]
    fn test_batch_runner_parallel() {
        let config1 = ScenarioPreset::Minimal.builder().seed(1).build();
        let config2 = ScenarioPreset::Minimal.builder().seed(2).build();

        let runner = BatchRunner::new(vec![
            ("seed1".to_string(), config1),
            ("seed2".to_string(), config2),
        ]).parallelism(2);

        let results = runner.run();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_run_result_metrics() {
        let config = ScenarioPreset::Quick.config();
        let report = run_simulation_from_config(&config);

        let result = RunResult {
            label: "test".to_string(),
            config_hash: 0,
            seed: 42,
            report,
            duration_ms: 100,
        };

        assert!(result.throughput() >= 0.0);
        assert!(result.robot_utilization() >= 0.0);
        assert!(result.robot_utilization() <= 1.0);
    }

    #[test]
    fn test_batch_runner_with_replications() {
        let config = ScenarioPreset::Minimal.config();
        let runner = BatchRunner::new(vec![("test".to_string(), config)]);

        let results = runner.run_with_replications(&[1, 2, 3]);
        assert_eq!(results.len(), 3);

        // Each result should have different seed
        let seeds: Vec<u64> = results.iter().map(|r| r.seed).collect();
        assert_eq!(seeds, vec![1, 2, 3]);
    }
}
