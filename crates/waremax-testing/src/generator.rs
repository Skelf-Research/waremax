//! Scenario generation utilities
//!
//! Provides ScenarioBuilder for creating scenarios programmatically and
//! SweepGenerator for parameter sweeps.

use waremax_config::{
    ArrivalProcess, BatchingConfig, BatteryConfig, ChargingStationConfig, ConsumptionConfig,
    DueTimeConfig, FailureConfig, LinesConfig, MaintenanceStationConfig, MapRef, MetricsConfig,
    OrderConfig, PolicyConfig, PriorityConfig, RobotConfig, RobotMaintenanceConfig, RoutingConfig,
    ScenarioConfig, ServiceTimeConfig, SimulationParams, SkuPopularity, StationAssignmentConfig,
    StationConfig, StorageRef, TaskAllocationConfig, TrafficConfig,
};

/// Builder for creating ScenarioConfig programmatically
#[derive(Clone)]
pub struct ScenarioBuilder {
    seed: u64,
    duration_minutes: f64,
    warmup_minutes: f64,

    // Grid configuration (for inline map generation)
    grid_rows: u32,
    grid_cols: u32,
    grid_spacing_m: f64,

    // Robot configuration
    robot_count: u32,
    robot_speed_mps: f64,
    robot_payload_kg: f64,
    battery_enabled: bool,
    battery_capacity_wh: f64,
    battery_min_soc: f64,
    maintenance_enabled: bool,
    maintenance_interval_hours: f64,
    failure_enabled: bool,
    failure_mtbf_hours: f64,

    // Station configuration
    pick_station_count: u32,
    station_concurrency: u32,
    station_queue_capacity: Option<u32>,
    service_time_base: f64,
    service_time_per_item: f64,
    service_time_distribution: String,
    service_time_stddev: f64,

    // Order configuration
    order_rate_per_hour: f64,
    items_per_order_mean: f64,
    sku_count: u32,
    due_time_minutes: Option<f64>,

    // Charging infrastructure
    charging_station_count: u32,
    charging_bays_per_station: u32,
    charging_rate_w: f64,

    // Maintenance infrastructure
    maintenance_station_count: u32,
    maintenance_bays_per_station: u32,
    maintenance_duration_s: f64,

    // Policy configuration
    task_allocation_policy: String,
    station_assignment_policy: String,
    batching_policy: String,
    priority_policy: String,
    traffic_policy: String,

    // Routing configuration
    routing_algorithm: String,
    congestion_aware: bool,

    // Metrics configuration
    metrics_sample_interval_s: f64,
    trace_enabled: bool,
}

impl Default for ScenarioBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ScenarioBuilder {
    /// Create a new ScenarioBuilder with default values
    pub fn new() -> Self {
        Self {
            seed: 42,
            duration_minutes: 60.0,
            warmup_minutes: 5.0,

            grid_rows: 10,
            grid_cols: 10,
            grid_spacing_m: 3.0,

            robot_count: 10,
            robot_speed_mps: 2.0,
            robot_payload_kg: 25.0,
            battery_enabled: false,
            battery_capacity_wh: 400.0,
            battery_min_soc: 0.15,
            maintenance_enabled: false,
            maintenance_interval_hours: 8.0,
            failure_enabled: false,
            failure_mtbf_hours: 100.0,

            pick_station_count: 4,
            station_concurrency: 1,
            station_queue_capacity: None,
            service_time_base: 8.0,
            service_time_per_item: 2.0,
            service_time_distribution: "constant".to_string(),
            service_time_stddev: 0.0,

            order_rate_per_hour: 60.0,
            items_per_order_mean: 3.0,
            sku_count: 100,
            due_time_minutes: None,

            charging_station_count: 0,
            charging_bays_per_station: 2,
            charging_rate_w: 200.0,

            maintenance_station_count: 0,
            maintenance_bays_per_station: 2,
            maintenance_duration_s: 300.0,

            task_allocation_policy: "nearest_robot".to_string(),
            station_assignment_policy: "least_queue".to_string(),
            batching_policy: "none".to_string(),
            priority_policy: "strict_priority".to_string(),
            traffic_policy: "wait_at_node".to_string(),

            routing_algorithm: "dijkstra".to_string(),
            congestion_aware: false,

            metrics_sample_interval_s: 60.0,
            trace_enabled: false,
        }
    }

    // === Simulation parameters ===

    /// Set the random seed
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Set simulation duration in minutes
    pub fn duration(mut self, minutes: f64) -> Self {
        self.duration_minutes = minutes;
        self
    }

    /// Set warmup duration in minutes
    pub fn warmup(mut self, minutes: f64) -> Self {
        self.warmup_minutes = minutes;
        self
    }

    // === Grid configuration ===

    /// Set grid dimensions (rows x cols with default spacing)
    pub fn grid(mut self, rows: u32, cols: u32) -> Self {
        self.grid_rows = rows;
        self.grid_cols = cols;
        self
    }

    /// Set grid dimensions with custom spacing
    pub fn grid_with_spacing(mut self, rows: u32, cols: u32, spacing_m: f64) -> Self {
        self.grid_rows = rows;
        self.grid_cols = cols;
        self.grid_spacing_m = spacing_m;
        self
    }

    // === Robot configuration ===

    /// Set number of robots
    pub fn robots(mut self, count: u32) -> Self {
        self.robot_count = count;
        self
    }

    /// Set robot speed in m/s
    pub fn robot_speed(mut self, mps: f64) -> Self {
        self.robot_speed_mps = mps;
        self
    }

    /// Set robot payload capacity in kg
    pub fn robot_payload(mut self, kg: f64) -> Self {
        self.robot_payload_kg = kg;
        self
    }

    /// Enable robot battery with specified capacity and min SOC
    pub fn robot_battery(mut self, capacity_wh: f64, min_soc: f64) -> Self {
        self.battery_enabled = true;
        self.battery_capacity_wh = capacity_wh;
        self.battery_min_soc = min_soc;
        self
    }

    /// Enable scheduled maintenance with interval in hours
    pub fn enable_maintenance(mut self, interval_hours: f64) -> Self {
        self.maintenance_enabled = true;
        self.maintenance_interval_hours = interval_hours;
        self
    }

    /// Enable random failures with MTBF in hours
    pub fn enable_failures(mut self, mtbf_hours: f64) -> Self {
        self.failure_enabled = true;
        self.failure_mtbf_hours = mtbf_hours;
        self
    }

    // === Station configuration ===

    /// Set number of pick stations
    pub fn pick_stations(mut self, count: u32) -> Self {
        self.pick_station_count = count;
        self
    }

    /// Set station concurrency (workers per station)
    pub fn station_concurrency(mut self, n: u32) -> Self {
        self.station_concurrency = n;
        self
    }

    /// Set station queue capacity
    pub fn station_queue_capacity(mut self, capacity: u32) -> Self {
        self.station_queue_capacity = Some(capacity);
        self
    }

    /// Set constant service time model
    pub fn service_time_constant(mut self, base_s: f64, per_item_s: f64) -> Self {
        self.service_time_distribution = "constant".to_string();
        self.service_time_base = base_s;
        self.service_time_per_item = per_item_s;
        self
    }

    /// Set lognormal service time model
    pub fn service_time_lognormal(mut self, mean_s: f64, stddev: f64, per_item_s: f64) -> Self {
        self.service_time_distribution = "lognormal".to_string();
        self.service_time_base = mean_s;
        self.service_time_stddev = stddev;
        self.service_time_per_item = per_item_s;
        self
    }

    // === Order configuration ===

    /// Set order arrival rate (orders per hour)
    pub fn order_rate(mut self, orders_per_hour: f64) -> Self {
        self.order_rate_per_hour = orders_per_hour;
        self
    }

    /// Set average items per order
    pub fn items_per_order(mut self, mean: f64) -> Self {
        self.items_per_order_mean = mean;
        self
    }

    /// Set number of SKUs in inventory
    pub fn sku_count(mut self, count: u32) -> Self {
        self.sku_count = count;
        self
    }

    /// Set due time offset in minutes from order arrival
    pub fn due_time(mut self, minutes: f64) -> Self {
        self.due_time_minutes = Some(minutes);
        self
    }

    // === Charging infrastructure ===

    /// Set charging station configuration
    pub fn charging_stations(mut self, count: u32, bays_each: u32, rate_w: f64) -> Self {
        self.charging_station_count = count;
        self.charging_bays_per_station = bays_each;
        self.charging_rate_w = rate_w;
        self
    }

    // === Maintenance infrastructure ===

    /// Set maintenance station configuration
    pub fn maintenance_stations(mut self, count: u32, bays_each: u32) -> Self {
        self.maintenance_station_count = count;
        self.maintenance_bays_per_station = bays_each;
        self
    }

    // === Policy configuration ===

    /// Set task allocation policy
    pub fn task_allocation(mut self, policy: &str) -> Self {
        self.task_allocation_policy = policy.to_string();
        self
    }

    /// Set station assignment policy
    pub fn station_assignment(mut self, policy: &str) -> Self {
        self.station_assignment_policy = policy.to_string();
        self
    }

    /// Set batching policy
    pub fn batching(mut self, policy: &str) -> Self {
        self.batching_policy = policy.to_string();
        self
    }

    /// Set priority policy
    pub fn priority(mut self, policy: &str) -> Self {
        self.priority_policy = policy.to_string();
        self
    }

    /// Set traffic policy
    pub fn traffic_policy(mut self, policy: &str) -> Self {
        self.traffic_policy = policy.to_string();
        self
    }

    // === Routing configuration ===

    /// Set routing algorithm
    pub fn routing_algorithm(mut self, algorithm: &str) -> Self {
        self.routing_algorithm = algorithm.to_string();
        self
    }

    /// Enable congestion-aware routing
    pub fn congestion_aware(mut self, enabled: bool) -> Self {
        self.congestion_aware = enabled;
        self
    }

    // === Metrics configuration ===

    /// Set metrics sample interval
    pub fn metrics_sample_interval(mut self, seconds: f64) -> Self {
        self.metrics_sample_interval_s = seconds;
        self
    }

    /// Enable event tracing
    pub fn trace(mut self, enabled: bool) -> Self {
        self.trace_enabled = enabled;
        self
    }

    /// Build the ScenarioConfig
    pub fn build(self) -> ScenarioConfig {
        // Generate station configs
        let mut stations = Vec::new();
        let nodes_count = self.grid_rows * self.grid_cols;

        for i in 0..self.pick_station_count {
            // Place stations at different nodes
            let node_id = i % nodes_count;
            stations.push(StationConfig {
                id: format!("S{}", i),
                node: node_id.to_string(),
                station_type: "pick".to_string(),
                concurrency: self.station_concurrency,
                queue_capacity: self.station_queue_capacity,
                service_time_s: if self.service_time_distribution == "lognormal" {
                    ServiceTimeConfig::lognormal(
                        self.service_time_base,
                        self.service_time_stddev,
                        self.service_time_per_item,
                        0.0,
                    )
                } else {
                    ServiceTimeConfig::constant(self.service_time_base, self.service_time_per_item)
                },
            });
        }

        // Generate charging station configs
        let mut charging_stations = Vec::new();
        for i in 0..self.charging_station_count {
            let node_id = (nodes_count - 1 - i) % nodes_count;
            charging_stations.push(ChargingStationConfig {
                id: format!("CHARGE_{}", i),
                node: node_id.to_string(),
                bays: self.charging_bays_per_station,
                charge_rate_w: self.charging_rate_w,
                queue_capacity: Some(10),
            });
        }

        // Generate maintenance station configs
        let mut maintenance_stations = Vec::new();
        for i in 0..self.maintenance_station_count {
            let node_id = (nodes_count / 2 + i) % nodes_count;
            maintenance_stations.push(MaintenanceStationConfig {
                id: format!("MAINT_{}", i),
                node: node_id.to_string(),
                bays: self.maintenance_bays_per_station,
                maintenance_duration_s: self.maintenance_duration_s,
                repair_time: ServiceTimeConfig::lognormal(600.0, 0.3, 0.0, 0.0),
                queue_capacity: Some(5),
            });
        }

        ScenarioConfig {
            seed: self.seed,
            simulation: SimulationParams {
                duration_minutes: self.duration_minutes,
                warmup_minutes: self.warmup_minutes,
                time_unit: "seconds".to_string(),
            },
            map: MapRef {
                file: "inline".to_string(), // Special marker for inline generation
            },
            storage: StorageRef {
                file: "inline".to_string(),
            },
            robots: RobotConfig {
                count: self.robot_count,
                max_speed_mps: self.robot_speed_mps,
                max_payload_kg: self.robot_payload_kg,
                battery: BatteryConfig {
                    enabled: self.battery_enabled,
                    capacity_wh: self.battery_capacity_wh,
                    min_soc: self.battery_min_soc,
                    consumption: ConsumptionConfig::default(),
                },
                maintenance: RobotMaintenanceConfig {
                    enabled: self.maintenance_enabled,
                    interval_hours: self.maintenance_interval_hours,
                },
                failure: FailureConfig {
                    enabled: self.failure_enabled,
                    mtbf_hours: self.failure_mtbf_hours,
                },
            },
            stations,
            orders: OrderConfig {
                arrival_process: ArrivalProcess {
                    process_type: "poisson".to_string(),
                    rate_per_min: self.order_rate_per_hour / 60.0,
                },
                lines_per_order: LinesConfig {
                    dist_type: "poisson".to_string(),
                    mean: self.items_per_order_mean,
                    dispersion: 1.0,
                },
                sku_popularity: SkuPopularity {
                    dist_type: "zipf".to_string(),
                    alpha: 1.0,
                },
                due_times: self.due_time_minutes.map(|m| DueTimeConfig {
                    due_type: "fixed_offset".to_string(),
                    minutes: m,
                }),
            },
            policies: PolicyConfig {
                task_allocation: TaskAllocationConfig {
                    alloc_type: self.task_allocation_policy,
                    travel_weight: 1.0,
                    queue_weight: 0.5,
                },
                station_assignment: StationAssignmentConfig {
                    assign_type: self.station_assignment_policy,
                },
                batching: BatchingConfig {
                    batch_type: self.batching_policy,
                    max_items: None,
                    max_weight_kg: None,
                },
                priority: PriorityConfig {
                    priority_type: self.priority_policy,
                    pick_weight: 1,
                    putaway_weight: 1,
                    replen_weight: 1,
                },
            },
            traffic: TrafficConfig {
                policy: self.traffic_policy,
                ..TrafficConfig::default()
            },
            routing: RoutingConfig {
                algorithm: self.routing_algorithm,
                congestion_aware: self.congestion_aware,
                ..RoutingConfig::default()
            },
            inbound: None,
            replenishment: None,
            charging_stations,
            metrics: MetricsConfig {
                sample_interval_s: self.metrics_sample_interval_s,
                trace: waremax_config::TraceConfig {
                    enabled: self.trace_enabled,
                    max_entries: 10000,
                    sample_rate: 1.0,
                },
                ..MetricsConfig::default()
            },
            maintenance_stations,
        }
    }

    /// Get the grid dimensions for this builder
    pub fn get_grid_dimensions(&self) -> (u32, u32, f64) {
        (self.grid_rows, self.grid_cols, self.grid_spacing_m)
    }

    /// Get the SKU count
    pub fn get_sku_count(&self) -> u32 {
        self.sku_count
    }
}

/// Parameter sweep dimension
#[derive(Clone)]
pub enum SweepDimension {
    /// Sweep robot count
    RobotCount(Vec<u32>),
    /// Sweep order rate (per hour)
    OrderRate(Vec<f64>),
    /// Sweep station count
    StationCount(Vec<u32>),
    /// Sweep grid size (rows, assuming square grid)
    GridSize(Vec<u32>),
    /// Sweep task allocation policy
    TaskAllocationPolicy(Vec<String>),
    /// Sweep random seeds
    Seed(Vec<u64>),
    /// Custom sweep with name and values
    Custom {
        name: String,
        #[allow(clippy::type_complexity)]
        applicator: std::sync::Arc<dyn Fn(&mut ScenarioBuilder, &serde_json::Value) + Send + Sync>,
    },
}

/// Generator for parameter sweeps
pub struct SweepGenerator {
    base: ScenarioBuilder,
    dimensions: Vec<SweepDimension>,
}

impl SweepGenerator {
    /// Create a new SweepGenerator with a base scenario builder
    pub fn new(base: ScenarioBuilder) -> Self {
        Self {
            base,
            dimensions: Vec::new(),
        }
    }

    /// Create a SweepGenerator from an existing ScenarioConfig
    pub fn from_config(config: ScenarioConfig) -> Self {
        // Create a builder initialized from the config
        let builder = ScenarioBuilder::new()
            .robots(config.robots.count)
            .pick_stations(config.stations.len() as u32)
            .order_rate(config.orders.arrival_process.rate_per_min * 60.0)
            .duration(config.simulation.duration_minutes)
            .warmup(config.simulation.warmup_minutes)
            .seed(config.seed);

        Self {
            base: builder,
            dimensions: Vec::new(),
        }
    }

    /// Sweep over robot counts
    pub fn sweep_robot_count(mut self, values: &[u32]) -> Self {
        self.dimensions
            .push(SweepDimension::RobotCount(values.to_vec()));
        self
    }

    /// Sweep over order rates (per hour)
    pub fn sweep_order_rate(mut self, values: &[f64]) -> Self {
        self.dimensions
            .push(SweepDimension::OrderRate(values.to_vec()));
        self
    }

    /// Sweep over station counts
    pub fn sweep_station_count(mut self, values: &[u32]) -> Self {
        self.dimensions
            .push(SweepDimension::StationCount(values.to_vec()));
        self
    }

    /// Sweep over grid sizes (square grids)
    pub fn sweep_grid_size(mut self, values: &[u32]) -> Self {
        self.dimensions
            .push(SweepDimension::GridSize(values.to_vec()));
        self
    }

    /// Sweep over task allocation policies
    pub fn sweep_task_allocation(mut self, policies: &[&str]) -> Self {
        self.dimensions.push(SweepDimension::TaskAllocationPolicy(
            policies.iter().map(|s| s.to_string()).collect(),
        ));
        self
    }

    /// Sweep over random seeds
    pub fn sweep_seeds(mut self, count: u32) -> Self {
        let seeds: Vec<u64> = (0..count).map(|i| 42 + i as u64 * 1000).collect();
        self.dimensions.push(SweepDimension::Seed(seeds));
        self
    }

    /// Sweep over specific seeds
    pub fn sweep_seeds_explicit(mut self, seeds: &[u64]) -> Self {
        self.dimensions.push(SweepDimension::Seed(seeds.to_vec()));
        self
    }

    /// Generate all scenario combinations (full factorial)
    pub fn generate(&self) -> Vec<(String, ScenarioConfig)> {
        if self.dimensions.is_empty() {
            return vec![("base".to_string(), self.base.clone().build())];
        }

        // Start with base configuration
        let mut results = vec![(Vec::<(String, String)>::new(), self.base.clone())];

        // Apply each dimension
        for dim in &self.dimensions {
            let mut new_results = Vec::new();

            for (labels, builder) in results {
                match dim {
                    SweepDimension::RobotCount(values) => {
                        for &v in values {
                            let mut new_labels = labels.clone();
                            new_labels.push(("robots".to_string(), v.to_string()));
                            new_results.push((new_labels, builder.clone().robots(v)));
                        }
                    }
                    SweepDimension::OrderRate(values) => {
                        for &v in values {
                            let mut new_labels = labels.clone();
                            new_labels.push(("order_rate".to_string(), format!("{:.0}", v)));
                            new_results.push((new_labels, builder.clone().order_rate(v)));
                        }
                    }
                    SweepDimension::StationCount(values) => {
                        for &v in values {
                            let mut new_labels = labels.clone();
                            new_labels.push(("stations".to_string(), v.to_string()));
                            new_results.push((new_labels, builder.clone().pick_stations(v)));
                        }
                    }
                    SweepDimension::GridSize(values) => {
                        for &v in values {
                            let mut new_labels = labels.clone();
                            new_labels.push(("grid".to_string(), format!("{}x{}", v, v)));
                            new_results.push((new_labels, builder.clone().grid(v, v)));
                        }
                    }
                    SweepDimension::TaskAllocationPolicy(values) => {
                        for v in values {
                            let mut new_labels = labels.clone();
                            new_labels.push(("alloc".to_string(), v.clone()));
                            new_results.push((new_labels, builder.clone().task_allocation(v)));
                        }
                    }
                    SweepDimension::Seed(values) => {
                        for &v in values {
                            let mut new_labels = labels.clone();
                            new_labels.push(("seed".to_string(), v.to_string()));
                            new_results.push((new_labels, builder.clone().seed(v)));
                        }
                    }
                    SweepDimension::Custom {
                        name,
                        applicator: _,
                    } => {
                        // For custom sweeps, we'd need the values stored differently
                        // For now, skip custom dimensions in factorial generation
                        let mut new_labels = labels.clone();
                        new_labels.push((name.clone(), "custom".to_string()));
                        new_results.push((new_labels, builder.clone()));
                    }
                }
            }

            results = new_results;
        }

        // Convert to final format with combined labels
        results
            .into_iter()
            .map(|(labels, builder)| {
                let label = labels
                    .into_iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("_");
                let label = if label.is_empty() {
                    "base".to_string()
                } else {
                    label
                };
                (label, builder.build())
            })
            .collect()
    }

    /// Get the number of scenarios that will be generated
    pub fn scenario_count(&self) -> usize {
        if self.dimensions.is_empty() {
            return 1;
        }

        self.dimensions
            .iter()
            .map(|dim| match dim {
                SweepDimension::RobotCount(v) => v.len(),
                SweepDimension::OrderRate(v) => v.len(),
                SweepDimension::StationCount(v) => v.len(),
                SweepDimension::GridSize(v) => v.len(),
                SweepDimension::TaskAllocationPolicy(v) => v.len(),
                SweepDimension::Seed(v) => v.len(),
                SweepDimension::Custom { .. } => 1,
            })
            .product()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_builder_defaults() {
        let config = ScenarioBuilder::new().build();

        assert_eq!(config.seed, 42);
        assert_eq!(config.robots.count, 10);
        assert_eq!(config.simulation.duration_minutes, 60.0);
    }

    #[test]
    fn test_scenario_builder_customization() {
        let config = ScenarioBuilder::new()
            .seed(123)
            .robots(20)
            .duration(120.0)
            .order_rate(100.0)
            .pick_stations(6)
            .task_allocation("auction")
            .build();

        assert_eq!(config.seed, 123);
        assert_eq!(config.robots.count, 20);
        assert_eq!(config.simulation.duration_minutes, 120.0);
        assert_eq!(config.stations.len(), 6);
        assert_eq!(config.policies.task_allocation.alloc_type, "auction");
    }

    #[test]
    fn test_sweep_generator_single_dimension() {
        let base = ScenarioBuilder::new();
        let sweep = SweepGenerator::new(base).sweep_robot_count(&[5, 10, 15]);

        let scenarios = sweep.generate();
        assert_eq!(scenarios.len(), 3);
        assert!(scenarios[0].0.contains("robots=5"));
        assert!(scenarios[1].0.contains("robots=10"));
        assert!(scenarios[2].0.contains("robots=15"));
    }

    #[test]
    fn test_sweep_generator_multiple_dimensions() {
        let base = ScenarioBuilder::new();
        let sweep = SweepGenerator::new(base)
            .sweep_robot_count(&[5, 10])
            .sweep_order_rate(&[60.0, 120.0]);

        let scenarios = sweep.generate();
        assert_eq!(scenarios.len(), 4); // 2 x 2 = 4 combinations
    }

    #[test]
    fn test_scenario_count() {
        let base = ScenarioBuilder::new();
        let sweep = SweepGenerator::new(base)
            .sweep_robot_count(&[5, 10, 15])
            .sweep_order_rate(&[60.0, 120.0])
            .sweep_seeds(3);

        assert_eq!(sweep.scenario_count(), 18); // 3 x 2 x 3 = 18
    }
}
