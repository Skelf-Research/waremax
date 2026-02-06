//! Configuration validation
//!
//! Provides comprehensive validation for scenario, map, and storage configs.
//! Catches errors early with clear, actionable error messages.

use super::map_config::MapConfig;
use super::scenario::ScenarioConfig;
use super::storage_config::StorageConfig;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Path to a configuration field (e.g., "stations[0].node")
#[derive(Debug, Clone)]
pub struct FieldPath(pub String);

impl FieldPath {
    pub fn new(path: &str) -> Self {
        Self(path.to_string())
    }

    pub fn root() -> Self {
        Self(String::new())
    }

    pub fn field(&self, name: &str) -> Self {
        if self.0.is_empty() {
            Self(name.to_string())
        } else {
            Self(format!("{}.{}", self.0, name))
        }
    }

    pub fn index(&self, idx: usize) -> Self {
        Self(format!("{}[{}]", self.0, idx))
    }
}

impl fmt::Display for FieldPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Detailed validation error with context
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub path: FieldPath,
    pub kind: ValidationErrorKind,
    pub suggestion: Option<String>,
}

impl ValidationError {
    pub fn new(path: FieldPath, kind: ValidationErrorKind) -> Self {
        Self {
            path,
            kind,
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.path, self.kind)?;
        if let Some(suggestion) = &self.suggestion {
            write!(f, " ({})", suggestion)?;
        }
        Ok(())
    }
}

/// Kinds of validation errors
#[derive(Debug, Clone)]
pub enum ValidationErrorKind {
    // Cross-reference errors
    NodeNotFound {
        node_id: String,
    },
    RackNotFound {
        rack_id: String,
    },

    // Value errors
    ValueMustBePositive {
        field: String,
        value: f64,
    },
    ValueMustBeNonNegative {
        field: String,
        value: f64,
    },
    IntValueMustBePositive {
        field: String,
        value: u32,
    },

    // Placement errors
    PlacementLevelOutOfBounds {
        level: u32,
        max_level: u32,
        rack_id: String,
    },
    PlacementBinOutOfBounds {
        bin: u32,
        max_bin: u32,
        rack_id: String,
    },

    // Type errors
    InvalidStationType {
        value: String,
        valid_types: Vec<String>,
    },

    // Consistency errors
    DuplicateId {
        id: String,
        entity_type: String,
    },
    EmptyCollection {
        collection: String,
    },
}

impl fmt::Display for ValidationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeNotFound { node_id } => {
                write!(f, "Node '{}' not found in map", node_id)
            }
            Self::RackNotFound { rack_id } => {
                write!(f, "Rack '{}' not found in storage config", rack_id)
            }
            Self::ValueMustBePositive { field, value } => {
                write!(f, "{} must be positive, got {}", field, value)
            }
            Self::ValueMustBeNonNegative { field, value } => {
                write!(f, "{} must be non-negative, got {}", field, value)
            }
            Self::IntValueMustBePositive { field, value } => {
                write!(f, "{} must be positive, got {}", field, value)
            }
            Self::PlacementLevelOutOfBounds {
                level,
                max_level,
                rack_id,
            } => {
                write!(
                    f,
                    "Level {} exceeds rack '{}' max level {} (0-indexed)",
                    level,
                    rack_id,
                    max_level - 1
                )
            }
            Self::PlacementBinOutOfBounds {
                bin,
                max_bin,
                rack_id,
            } => {
                write!(
                    f,
                    "Bin {} exceeds rack '{}' max bin {} (0-indexed)",
                    bin,
                    rack_id,
                    max_bin - 1
                )
            }
            Self::InvalidStationType { value, valid_types } => {
                write!(
                    f,
                    "Invalid station type '{}', valid types: {}",
                    value,
                    valid_types.join(", ")
                )
            }
            Self::DuplicateId { id, entity_type } => {
                write!(f, "Duplicate {} ID: '{}'", entity_type, id)
            }
            Self::EmptyCollection { collection } => {
                write!(f, "{} cannot be empty", collection)
            }
        }
    }
}

/// Collection of validation errors and warnings
#[derive(Debug, Default)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: ValidationError) {
        self.warnings.push(warning);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Convert to Result: Ok(warnings) if no errors, Err(errors) if there are errors
    pub fn into_result(self) -> Result<Vec<ValidationError>, Vec<ValidationError>> {
        if self.errors.is_empty() {
            Ok(self.warnings)
        } else {
            Err(self.errors)
        }
    }
}

// ============================================================================
// Main Entry Points
// ============================================================================

/// Validate all configuration files for consistency and correctness.
///
/// Returns Ok with warnings if valid, Err with errors if invalid.
/// Warnings are non-fatal issues that may indicate problems but don't prevent simulation.
pub fn validate_scenario(
    scenario: &ScenarioConfig,
    map: Option<&MapConfig>,
    storage: Option<&StorageConfig>,
) -> Result<Vec<ValidationError>, Vec<ValidationError>> {
    let mut ctx = ValidationErrors::new();

    // Phase 1: Validate scenario in isolation (always runs)
    validate_scenario_standalone(scenario, &mut ctx);

    // Phase 2: Validate map in isolation (if provided)
    if let Some(map) = map {
        validate_map_standalone(map, &mut ctx);
    }

    // Phase 3: Validate storage in isolation (if provided)
    if let Some(storage) = storage {
        validate_storage_standalone(storage, &mut ctx);
    }

    // Phase 4: Cross-reference validations (if all configs provided)
    if let (Some(map), Some(storage)) = (map, storage) {
        validate_cross_references(scenario, map, storage, &mut ctx);
    }

    ctx.into_result()
}

/// Validate scenario without external map/storage files (for demo mode)
pub fn validate_scenario_only(
    scenario: &ScenarioConfig,
) -> Result<Vec<ValidationError>, Vec<ValidationError>> {
    validate_scenario(scenario, None, None)
}

// ============================================================================
// Standalone Validators
// ============================================================================

fn validate_scenario_standalone(scenario: &ScenarioConfig, ctx: &mut ValidationErrors) {
    let root = FieldPath::new("scenario");

    // Simulation params
    let sim = root.field("simulation");
    if scenario.simulation.duration_minutes <= 0.0 {
        ctx.add_error(ValidationError::new(
            sim.field("duration_minutes"),
            ValidationErrorKind::ValueMustBePositive {
                field: "duration_minutes".to_string(),
                value: scenario.simulation.duration_minutes,
            },
        ));
    }
    if scenario.simulation.warmup_minutes < 0.0 {
        ctx.add_error(ValidationError::new(
            sim.field("warmup_minutes"),
            ValidationErrorKind::ValueMustBeNonNegative {
                field: "warmup_minutes".to_string(),
                value: scenario.simulation.warmup_minutes,
            },
        ));
    }

    // Robot config
    let robots = root.field("robots");
    if scenario.robots.count == 0 {
        ctx.add_error(ValidationError::new(
            robots.field("count"),
            ValidationErrorKind::IntValueMustBePositive {
                field: "count".to_string(),
                value: 0,
            },
        ));
    }
    if scenario.robots.max_speed_mps <= 0.0 {
        ctx.add_error(ValidationError::new(
            robots.field("max_speed_mps"),
            ValidationErrorKind::ValueMustBePositive {
                field: "max_speed_mps".to_string(),
                value: scenario.robots.max_speed_mps,
            },
        ));
    }
    if scenario.robots.max_payload_kg <= 0.0 {
        ctx.add_error(ValidationError::new(
            robots.field("max_payload_kg"),
            ValidationErrorKind::ValueMustBePositive {
                field: "max_payload_kg".to_string(),
                value: scenario.robots.max_payload_kg,
            },
        ));
    }

    // Stations
    let stations = root.field("stations");
    if scenario.stations.is_empty() {
        ctx.add_error(ValidationError::new(
            stations.clone(),
            ValidationErrorKind::EmptyCollection {
                collection: "stations".to_string(),
            },
        ));
    }

    // Check station uniqueness and validity
    let mut station_ids: HashSet<&str> = HashSet::new();
    let valid_station_types = vec!["pick", "drop", "inbound", "outbound"];

    for (i, station) in scenario.stations.iter().enumerate() {
        let station_path = stations.index(i);

        // Check unique ID
        if !station_ids.insert(&station.id) {
            ctx.add_error(ValidationError::new(
                station_path.field("id"),
                ValidationErrorKind::DuplicateId {
                    id: station.id.clone(),
                    entity_type: "station".to_string(),
                },
            ));
        }

        // Check valid station type
        if !valid_station_types.contains(&station.station_type.as_str()) {
            ctx.add_error(
                ValidationError::new(
                    station_path.field("type"),
                    ValidationErrorKind::InvalidStationType {
                        value: station.station_type.clone(),
                        valid_types: valid_station_types.iter().map(|s| s.to_string()).collect(),
                    },
                )
                .with_suggestion("use one of: pick, drop, inbound, outbound"),
            );
        }

        // Check concurrency > 0
        if station.concurrency == 0 {
            ctx.add_error(ValidationError::new(
                station_path.field("concurrency"),
                ValidationErrorKind::IntValueMustBePositive {
                    field: "concurrency".to_string(),
                    value: 0,
                },
            ));
        }

        // Check service time
        if station.service_time_s.base < 0.0 {
            ctx.add_error(ValidationError::new(
                station_path.field("service_time_s.base"),
                ValidationErrorKind::ValueMustBeNonNegative {
                    field: "base".to_string(),
                    value: station.service_time_s.base,
                },
            ));
        }
        if station.service_time_s.per_item < 0.0 {
            ctx.add_error(ValidationError::new(
                station_path.field("service_time_s.per_item"),
                ValidationErrorKind::ValueMustBeNonNegative {
                    field: "per_item".to_string(),
                    value: station.service_time_s.per_item,
                },
            ));
        }
    }

    // Orders config
    let orders = root.field("orders");
    if scenario.orders.arrival_process.rate_per_min <= 0.0 {
        ctx.add_error(ValidationError::new(
            orders.field("arrival_process.rate_per_min"),
            ValidationErrorKind::ValueMustBePositive {
                field: "rate_per_min".to_string(),
                value: scenario.orders.arrival_process.rate_per_min,
            },
        ));
    }
    if scenario.orders.lines_per_order.mean <= 0.0 {
        ctx.add_error(ValidationError::new(
            orders.field("lines_per_order.mean"),
            ValidationErrorKind::ValueMustBePositive {
                field: "mean".to_string(),
                value: scenario.orders.lines_per_order.mean,
            },
        ));
    }
    if scenario.orders.lines_per_order.dispersion <= 0.0 {
        ctx.add_error(ValidationError::new(
            orders.field("lines_per_order.dispersion"),
            ValidationErrorKind::ValueMustBePositive {
                field: "dispersion".to_string(),
                value: scenario.orders.lines_per_order.dispersion,
            },
        ));
    }
    if scenario.orders.sku_popularity.alpha <= 0.0 {
        ctx.add_error(ValidationError::new(
            orders.field("sku_popularity.alpha"),
            ValidationErrorKind::ValueMustBePositive {
                field: "alpha".to_string(),
                value: scenario.orders.sku_popularity.alpha,
            },
        ));
    }

    // Traffic config
    let traffic = root.field("traffic");
    if scenario.traffic.edge_capacity_default == 0 {
        ctx.add_error(ValidationError::new(
            traffic.field("edge_capacity_default"),
            ValidationErrorKind::IntValueMustBePositive {
                field: "edge_capacity_default".to_string(),
                value: 0,
            },
        ));
    }
    if scenario.traffic.node_capacity_default == 0 {
        ctx.add_error(ValidationError::new(
            traffic.field("node_capacity_default"),
            ValidationErrorKind::IntValueMustBePositive {
                field: "node_capacity_default".to_string(),
                value: 0,
            },
        ));
    }
}

fn validate_map_standalone(map: &MapConfig, ctx: &mut ValidationErrors) {
    let root = FieldPath::new("map");

    // Nodes must not be empty
    let nodes_path = root.field("nodes");
    if map.nodes.is_empty() {
        ctx.add_error(ValidationError::new(
            nodes_path.clone(),
            ValidationErrorKind::EmptyCollection {
                collection: "nodes".to_string(),
            },
        ));
    }

    // Check node uniqueness
    let mut node_ids: HashSet<&str> = HashSet::new();
    for (i, node) in map.nodes.iter().enumerate() {
        if !node_ids.insert(&node.id) {
            ctx.add_error(ValidationError::new(
                nodes_path.index(i).field("id"),
                ValidationErrorKind::DuplicateId {
                    id: node.id.clone(),
                    entity_type: "node".to_string(),
                },
            ));
        }
    }

    // Validate edges
    let edges_path = root.field("edges");
    for (i, edge) in map.edges.iter().enumerate() {
        let edge_path = edges_path.index(i);

        // Check edge length > 0
        if edge.length_m <= 0.0 {
            ctx.add_error(ValidationError::new(
                edge_path.field("length_m"),
                ValidationErrorKind::ValueMustBePositive {
                    field: "length_m".to_string(),
                    value: edge.length_m,
                },
            ));
        }

        // Check capacity > 0
        if edge.capacity == 0 {
            ctx.add_error(ValidationError::new(
                edge_path.field("capacity"),
                ValidationErrorKind::IntValueMustBePositive {
                    field: "capacity".to_string(),
                    value: 0,
                },
            ));
        }

        // Check from/to nodes exist
        if !node_ids.contains(edge.from.as_str()) {
            ctx.add_error(
                ValidationError::new(
                    edge_path.field("from"),
                    ValidationErrorKind::NodeNotFound {
                        node_id: edge.from.clone(),
                    },
                )
                .with_suggestion(format!("available nodes: {:?}", node_ids)),
            );
        }
        if !node_ids.contains(edge.to.as_str()) {
            ctx.add_error(
                ValidationError::new(
                    edge_path.field("to"),
                    ValidationErrorKind::NodeNotFound {
                        node_id: edge.to.clone(),
                    },
                )
                .with_suggestion(format!("available nodes: {:?}", node_ids)),
            );
        }
    }
}

fn validate_storage_standalone(storage: &StorageConfig, ctx: &mut ValidationErrors) {
    let root = FieldPath::new("storage");

    // Check rack uniqueness and validity
    let racks_path = root.field("racks");
    let mut rack_ids: HashSet<&str> = HashSet::new();

    for (i, rack) in storage.racks.iter().enumerate() {
        let rack_path = racks_path.index(i);

        // Check unique ID
        if !rack_ids.insert(&rack.id) {
            ctx.add_error(ValidationError::new(
                rack_path.field("id"),
                ValidationErrorKind::DuplicateId {
                    id: rack.id.clone(),
                    entity_type: "rack".to_string(),
                },
            ));
        }

        // Check levels > 0
        if rack.levels == 0 {
            ctx.add_error(ValidationError::new(
                rack_path.field("levels"),
                ValidationErrorKind::IntValueMustBePositive {
                    field: "levels".to_string(),
                    value: 0,
                },
            ));
        }

        // Check bins_per_level > 0
        if rack.bins_per_level == 0 {
            ctx.add_error(ValidationError::new(
                rack_path.field("bins_per_level"),
                ValidationErrorKind::IntValueMustBePositive {
                    field: "bins_per_level".to_string(),
                    value: 0,
                },
            ));
        }
    }
}

fn validate_cross_references(
    scenario: &ScenarioConfig,
    map: &MapConfig,
    storage: &StorageConfig,
    ctx: &mut ValidationErrors,
) {
    // Build lookup sets
    let node_ids: HashSet<&str> = map.nodes.iter().map(|n| n.id.as_str()).collect();
    let rack_map: HashMap<&str, &super::storage_config::RackConfig> =
        storage.racks.iter().map(|r| (r.id.as_str(), r)).collect();

    // Validate station nodes exist in map
    for (i, station) in scenario.stations.iter().enumerate() {
        if !node_ids.contains(station.node.as_str()) {
            ctx.add_error(
                ValidationError::new(
                    FieldPath::new(&format!("scenario.stations[{}].node", i)),
                    ValidationErrorKind::NodeNotFound {
                        node_id: station.node.clone(),
                    },
                )
                .with_suggestion(format!(
                    "available nodes: {}",
                    node_ids
                        .iter()
                        .take(10)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            );
        }
    }

    // Validate rack access_nodes exist in map
    for (i, rack) in storage.racks.iter().enumerate() {
        if !node_ids.contains(rack.access_node.as_str()) {
            ctx.add_error(
                ValidationError::new(
                    FieldPath::new(&format!("storage.racks[{}].access_node", i)),
                    ValidationErrorKind::NodeNotFound {
                        node_id: rack.access_node.clone(),
                    },
                )
                .with_suggestion(format!(
                    "available nodes: {}",
                    node_ids
                        .iter()
                        .take(10)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            );
        }
    }

    // Validate placements reference valid racks with valid level/bin
    for (sku, placements) in &storage.placements {
        for (j, placement) in placements.iter().enumerate() {
            let path = FieldPath::new(&format!("storage.placements.{}[{}]", sku, j));

            if let Some(rack) = rack_map.get(placement.rack.as_str()) {
                // Check level bounds
                if placement.level >= rack.levels {
                    ctx.add_error(ValidationError::new(
                        path.field("level"),
                        ValidationErrorKind::PlacementLevelOutOfBounds {
                            level: placement.level,
                            max_level: rack.levels,
                            rack_id: rack.id.clone(),
                        },
                    ));
                }

                // Check bin bounds
                if placement.bin >= rack.bins_per_level {
                    ctx.add_error(ValidationError::new(
                        path.field("bin"),
                        ValidationErrorKind::PlacementBinOutOfBounds {
                            bin: placement.bin,
                            max_bin: rack.bins_per_level,
                            rack_id: rack.id.clone(),
                        },
                    ));
                }
            } else {
                ctx.add_error(
                    ValidationError::new(
                        path.field("rack"),
                        ValidationErrorKind::RackNotFound {
                            rack_id: placement.rack.clone(),
                        },
                    )
                    .with_suggestion(format!(
                        "available racks: {}",
                        rack_map.keys().cloned().collect::<Vec<_>>().join(", ")
                    )),
                );
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::*;

    fn minimal_scenario() -> ScenarioConfig {
        ScenarioConfig {
            seed: 42,
            simulation: SimulationParams {
                duration_minutes: 60.0,
                warmup_minutes: 5.0,
                time_unit: "seconds".to_string(),
            },
            map: MapRef {
                file: "map.json".to_string(),
            },
            storage: StorageRef {
                file: "storage.yaml".to_string(),
            },
            robots: RobotConfig {
                count: 5,
                max_speed_mps: 1.5,
                max_payload_kg: 25.0,
                battery: BatteryConfig::default(),
                maintenance: RobotMaintenanceConfig::default(),
                failure: FailureConfig::default(),
            },
            stations: vec![StationConfig {
                id: "S1".to_string(),
                node: "0".to_string(),
                station_type: "pick".to_string(),
                concurrency: 2,
                queue_capacity: None,
                service_time_s: ServiceTimeConfig::constant(5.0, 2.0),
            }],
            orders: OrderConfig {
                arrival_process: ArrivalProcess {
                    process_type: "poisson".to_string(),
                    rate_per_min: 4.0,
                },
                lines_per_order: LinesConfig {
                    dist_type: "negbin".to_string(),
                    mean: 2.0,
                    dispersion: 1.0,
                },
                sku_popularity: SkuPopularity {
                    dist_type: "zipf".to_string(),
                    alpha: 1.0,
                },
                due_times: Some(DueTimeConfig {
                    due_type: "fixed".to_string(),
                    minutes: 60.0,
                }),
            },
            policies: PolicyConfig::default(),
            traffic: TrafficConfig::default(),
            routing: RoutingConfig::default(),
            inbound: None,
            replenishment: None,
            charging_stations: vec![],
            metrics: MetricsConfig::default(),
            maintenance_stations: vec![],
        }
    }

    #[test]
    fn test_valid_scenario() {
        let scenario = minimal_scenario();
        let result = validate_scenario_only(&scenario);
        assert!(result.is_ok(), "Expected valid scenario: {:?}", result);
    }

    #[test]
    fn test_negative_duration_rejected() {
        let mut scenario = minimal_scenario();
        scenario.simulation.duration_minutes = -10.0;
        let result = validate_scenario_only(&scenario);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            &e.kind,
            ValidationErrorKind::ValueMustBePositive { field, .. } if field == "duration_minutes"
        )));
    }

    #[test]
    fn test_zero_robot_count_rejected() {
        let mut scenario = minimal_scenario();
        scenario.robots.count = 0;
        let result = validate_scenario_only(&scenario);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            &e.kind,
            ValidationErrorKind::IntValueMustBePositive { field, .. } if field == "count"
        )));
    }

    #[test]
    fn test_invalid_station_type_rejected() {
        let mut scenario = minimal_scenario();
        scenario.stations[0].station_type = "invalid".to_string();
        let result = validate_scenario_only(&scenario);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(&e.kind, ValidationErrorKind::InvalidStationType { .. })));
    }

    #[test]
    fn test_duplicate_station_id_rejected() {
        let mut scenario = minimal_scenario();
        scenario.stations.push(StationConfig {
            id: "S1".to_string(), // Duplicate
            node: "1".to_string(),
            station_type: "drop".to_string(),
            concurrency: 1,
            queue_capacity: None,
            service_time_s: ServiceTimeConfig::constant(3.0, 1.0),
        });
        let result = validate_scenario_only(&scenario);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(&e.kind, ValidationErrorKind::DuplicateId { .. })));
    }

    #[test]
    fn test_empty_stations_rejected() {
        let mut scenario = minimal_scenario();
        scenario.stations.clear();
        let result = validate_scenario_only(&scenario);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(&e.kind, ValidationErrorKind::EmptyCollection { .. })));
    }

    #[test]
    fn test_negative_service_time_rejected() {
        let mut scenario = minimal_scenario();
        scenario.stations[0].service_time_s.base = -5.0;
        let result = validate_scenario_only(&scenario);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            &e.kind,
            ValidationErrorKind::ValueMustBeNonNegative { field, .. } if field == "base"
        )));
    }

    #[test]
    fn test_map_edge_references_invalid_node() {
        use crate::map_config::*;

        let map = MapConfig {
            nodes: vec![NodeConfig {
                id: "N1".to_string(),
                x: 0.0,
                y: 0.0,
                node_type: "aisle".to_string(),
            }],
            edges: vec![EdgeConfig {
                from: "N1".to_string(),
                to: "N2".to_string(), // N2 doesn't exist
                length_m: 3.0,
                bidirectional: true,
                direction: None,
                capacity: 1,
                speed_multiplier: None,
            }],
            constraints: ConstraintsConfig::default(),
        };

        let mut ctx = ValidationErrors::new();
        validate_map_standalone(&map, &mut ctx);

        assert!(ctx.has_errors());
        assert!(ctx.errors.iter().any(
            |e| matches!(&e.kind, ValidationErrorKind::NodeNotFound { node_id } if node_id == "N2")
        ));
    }

    #[test]
    fn test_placement_out_of_bounds() {
        use crate::storage_config::*;

        let storage = StorageConfig {
            racks: vec![RackConfig {
                id: "R1".to_string(),
                access_node: "N1".to_string(),
                levels: 3,
                bins_per_level: 4,
                zone: None,
                base_access_time_s: None,
                per_level_time_s: None,
            }],
            placements: [(
                "SKU001".to_string(),
                vec![PlacementConfig {
                    rack: "R1".to_string(),
                    level: 5, // Out of bounds (max is 2)
                    bin: 0,
                    qty: 10,
                }],
            )]
            .into_iter()
            .collect(),
            skus: vec![],
        };

        let scenario = minimal_scenario();
        let map = MapConfig {
            nodes: vec![crate::map_config::NodeConfig {
                id: "N1".to_string(),
                x: 0.0,
                y: 0.0,
                node_type: "rack".to_string(),
            }],
            edges: vec![],
            constraints: crate::map_config::ConstraintsConfig::default(),
        };

        let result = validate_scenario(&scenario, Some(&map), Some(&storage));
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            &e.kind,
            ValidationErrorKind::PlacementLevelOutOfBounds { .. }
        )));
    }
}
