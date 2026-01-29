//! Scenario configuration parsing

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unsupported file format")]
    UnsupportedFormat,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScenarioConfig {
    pub seed: u64,
    pub simulation: SimulationParams,
    pub map: MapRef,
    pub storage: StorageRef,
    pub robots: RobotConfig,
    pub stations: Vec<StationConfig>,
    pub orders: OrderConfig,
    #[serde(default)]
    pub policies: PolicyConfig,
    #[serde(default)]
    pub traffic: TrafficConfig,
    #[serde(default)]
    pub routing: RoutingConfig,
    /// v1: Inbound/putaway configuration
    #[serde(default)]
    pub inbound: Option<InboundConfig>,
    /// v1: Replenishment configuration
    #[serde(default)]
    pub replenishment: Option<ReplenishmentConfig>,
    /// v1: Charging station configuration
    #[serde(default)]
    pub charging_stations: Vec<ChargingStationConfig>,
    /// v1: Metrics configuration
    #[serde(default)]
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimulationParams {
    pub duration_minutes: f64,
    #[serde(default)]
    pub warmup_minutes: f64,
    #[serde(default = "default_time_unit")]
    pub time_unit: String,
}

fn default_time_unit() -> String {
    "seconds".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapRef {
    pub file: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageRef {
    pub file: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RobotConfig {
    pub count: u32,
    pub max_speed_mps: f64,
    #[serde(default = "default_payload")]
    pub max_payload_kg: f64,
}

fn default_payload() -> f64 {
    25.0
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StationConfig {
    pub id: String,
    pub node: String,
    #[serde(rename = "type")]
    pub station_type: String,
    #[serde(default = "default_concurrency")]
    pub concurrency: u32,
    pub queue_capacity: Option<u32>,
    pub service_time_s: ServiceTimeConfig,
}

fn default_concurrency() -> u32 {
    1
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceTimeConfig {
    pub base: f64,
    #[serde(default)]
    pub per_item: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderConfig {
    pub arrival_process: ArrivalProcess,
    pub lines_per_order: LinesConfig,
    pub sku_popularity: SkuPopularity,
    #[serde(default)]
    pub due_times: Option<DueTimeConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArrivalProcess {
    #[serde(rename = "type")]
    pub process_type: String,
    pub rate_per_min: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinesConfig {
    #[serde(rename = "type")]
    pub dist_type: String,
    pub mean: f64,
    #[serde(default = "default_dispersion")]
    pub dispersion: f64,
}

fn default_dispersion() -> f64 {
    1.0
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkuPopularity {
    #[serde(rename = "type")]
    pub dist_type: String,
    #[serde(default = "default_alpha")]
    pub alpha: f64,
}

fn default_alpha() -> f64 {
    1.0
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DueTimeConfig {
    #[serde(rename = "type")]
    pub due_type: String,
    pub minutes: f64,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PolicyConfig {
    #[serde(default)]
    pub task_allocation: TaskAllocationConfig,
    #[serde(default)]
    pub station_assignment: StationAssignmentConfig,
    #[serde(default)]
    pub batching: BatchingConfig,
    #[serde(default)]
    pub priority: PriorityConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskAllocationConfig {
    #[serde(rename = "type", default = "default_allocation")]
    pub alloc_type: String,
}

fn default_allocation() -> String {
    "nearest_robot".to_string()
}

impl Default for TaskAllocationConfig {
    fn default() -> Self {
        Self {
            alloc_type: default_allocation(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StationAssignmentConfig {
    #[serde(rename = "type", default = "default_station_assign")]
    pub assign_type: String,
}

fn default_station_assign() -> String {
    "least_queue".to_string()
}

impl Default for StationAssignmentConfig {
    fn default() -> Self {
        Self {
            assign_type: default_station_assign(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchingConfig {
    #[serde(rename = "type", default = "default_batching")]
    pub batch_type: String,
    pub max_items: Option<u32>,
}

fn default_batching() -> String {
    "none".to_string()
}

impl Default for BatchingConfig {
    fn default() -> Self {
        Self {
            batch_type: default_batching(),
            max_items: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PriorityConfig {
    #[serde(rename = "type", default = "default_priority")]
    pub priority_type: String,
}

fn default_priority() -> String {
    "strict_priority".to_string()
}

impl Default for PriorityConfig {
    fn default() -> Self {
        Self {
            priority_type: default_priority(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TrafficConfig {
    #[serde(default = "default_traffic_policy")]
    pub policy: String,
    #[serde(default = "default_capacity")]
    pub edge_capacity_default: u32,
    #[serde(default = "default_capacity")]
    pub node_capacity_default: u32,
}

fn default_traffic_policy() -> String {
    "wait_at_node".to_string()
}

fn default_capacity() -> u32 {
    1
}

impl Default for TrafficConfig {
    fn default() -> Self {
        Self {
            policy: default_traffic_policy(),
            edge_capacity_default: default_capacity(),
            node_capacity_default: default_capacity(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoutingConfig {
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
    #[serde(default)]
    pub congestion_aware: bool,
    #[serde(default = "default_cache")]
    pub cache_routes: bool,
}

fn default_algorithm() -> String {
    "dijkstra".to_string()
}

fn default_cache() -> bool {
    true
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            algorithm: default_algorithm(),
            congestion_aware: false,
            cache_routes: default_cache(),
        }
    }
}

// === v1: Inbound/Putaway Configuration ===

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InboundConfig {
    /// Arrival process for inbound shipments
    pub arrival_process: ArrivalProcess,
    /// SKU distribution in shipments
    #[serde(default)]
    pub sku_distribution: Option<SkuPopularity>,
    /// Average items per shipment
    #[serde(default = "default_items_per_shipment")]
    pub items_per_shipment: f64,
}

fn default_items_per_shipment() -> f64 {
    50.0
}

// === v1: Replenishment Configuration ===

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReplenishmentConfig {
    /// Enable automatic replenishment triggers
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Default replenishment threshold (items)
    #[serde(default = "default_replen_threshold")]
    pub default_threshold: u32,
    /// Per-SKU thresholds (overrides default)
    #[serde(default)]
    pub sku_thresholds: std::collections::HashMap<String, u32>,
}

fn default_enabled() -> bool {
    true
}

fn default_replen_threshold() -> u32 {
    10
}

impl Default for ReplenishmentConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            default_threshold: default_replen_threshold(),
            sku_thresholds: std::collections::HashMap::new(),
        }
    }
}

// === v1: Charging Station Configuration ===

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChargingStationConfig {
    pub id: String,
    pub node: String,
    #[serde(default = "default_bays")]
    pub bays: u32,
    #[serde(default = "default_charge_rate")]
    pub charge_rate_w: f64,
    pub queue_capacity: Option<u32>,
}

fn default_bays() -> u32 {
    1
}

fn default_charge_rate() -> f64 {
    200.0
}

// === v1: Battery Configuration ===

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatteryConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_battery_capacity")]
    pub capacity_wh: f64,
    #[serde(default = "default_min_soc")]
    pub min_soc: f64,
    #[serde(default)]
    pub consumption: ConsumptionConfig,
}

fn default_battery_capacity() -> f64 {
    400.0
}

fn default_min_soc() -> f64 {
    0.15
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            capacity_wh: default_battery_capacity(),
            min_soc: default_min_soc(),
            consumption: ConsumptionConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConsumptionConfig {
    #[serde(default = "default_per_meter")]
    pub per_meter_wh: f64,
    #[serde(default = "default_per_kg_meter")]
    pub per_kg_per_meter_wh: f64,
    #[serde(default = "default_idle_power")]
    pub idle_power_w: f64,
    #[serde(default = "default_service_power")]
    pub service_power_w: f64,
}

fn default_per_meter() -> f64 {
    0.1
}

fn default_per_kg_meter() -> f64 {
    0.01
}

fn default_idle_power() -> f64 {
    5.0
}

fn default_service_power() -> f64 {
    20.0
}

impl Default for ConsumptionConfig {
    fn default() -> Self {
        Self {
            per_meter_wh: default_per_meter(),
            per_kg_per_meter_wh: default_per_kg_meter(),
            idle_power_w: default_idle_power(),
            service_power_w: default_service_power(),
        }
    }
}

// === v1: Metrics Configuration ===

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricsConfig {
    #[serde(default = "default_sample_interval")]
    pub sample_interval_s: f64,
    #[serde(default = "default_congestion_top_n")]
    pub congestion_top_n: usize,
    #[serde(default)]
    pub track_sla: bool,
}

fn default_sample_interval() -> f64 {
    60.0
}

fn default_congestion_top_n() -> usize {
    10
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            sample_interval_s: default_sample_interval(),
            congestion_top_n: default_congestion_top_n(),
            track_sla: false,
        }
    }
}

impl ScenarioConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;

        if path.ends_with(".yaml") || path.ends_with(".yml") {
            Ok(serde_yaml::from_str(&content)?)
        } else if path.ends_with(".json") {
            Ok(serde_json::from_str(&content)?)
        } else {
            Err(ConfigError::UnsupportedFormat)
        }
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, ConfigError> {
        Ok(serde_yaml::from_str(yaml)?)
    }

    pub fn from_json(json: &str) -> Result<Self, ConfigError> {
        Ok(serde_json::from_str(json)?)
    }
}
