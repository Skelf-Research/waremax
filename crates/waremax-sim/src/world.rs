//! World state container

use std::collections::HashMap;
use waremax_analysis::AttributionCollector;
use waremax_core::{
    ChargingStationId, IdGenerator, MaintenanceStationId, NodeId, OrderId, RackId, RobotId, SimRng,
    SimTime, SkuId, StationId, TaskId,
};
use waremax_entities::{ChargingStation, MaintenanceStation, Order, Robot, Station, Task};
use waremax_map::{NodeType, ReservationManager, Router, TrafficManager, WarehouseMap};
use waremax_metrics::{EventTraceCollector, TimeSeriesCollector};
use waremax_policies::{
    BatchingPolicy, DeadlockResolver, LeastQueuePolicy, NearestRobotPolicy, NoBatchingPolicy,
    PolicyContext, PriorityPolicy, StationAssignmentPolicy, StrictPriorityPolicy,
    TaskAllocationPolicy, TrafficPolicy, WaitAtNodePolicy, YoungestRobotBacksUp,
};
use waremax_storage::{BinAddress, Inventory, Rack, Sku, SkuCatalog};

use crate::distributions::DistributionSet;

/// Container for all active policies
pub struct PolicySet {
    pub task_allocation: Box<dyn TaskAllocationPolicy>,
    pub station_assignment: Box<dyn StationAssignmentPolicy>,
    pub batching: Box<dyn BatchingPolicy>,
    pub priority: Box<dyn PriorityPolicy>,
    /// v1: Traffic policy for handling congestion
    pub traffic: Box<dyn TrafficPolicy>,
}

impl PolicySet {
    /// Get policy names for logging
    pub fn names(&self) -> (String, String, String, String) {
        (
            self.task_allocation.name().to_string(),
            self.station_assignment.name().to_string(),
            self.batching.name().to_string(),
            self.priority.name().to_string(),
        )
    }

    /// Get all policy names including traffic (v1)
    pub fn all_names(&self) -> (String, String, String, String, String) {
        (
            self.task_allocation.name().to_string(),
            self.station_assignment.name().to_string(),
            self.batching.name().to_string(),
            self.priority.name().to_string(),
            self.traffic.name().to_string(),
        )
    }
}

impl Default for PolicySet {
    fn default() -> Self {
        Self {
            task_allocation: Box::new(NearestRobotPolicy::new()),
            station_assignment: Box::new(LeastQueuePolicy::default()),
            batching: Box::new(NoBatchingPolicy::new()),
            priority: Box::new(StrictPriorityPolicy::new()),
            traffic: Box::new(WaitAtNodePolicy::new()),
        }
    }
}

/// Container for all simulation state
pub struct World {
    // Random number generator
    pub rng: SimRng,

    // Map and routing
    pub map: WarehouseMap,
    pub router: Router,
    pub traffic: TrafficManager,

    // Storage
    pub racks: HashMap<waremax_core::RackId, Rack>,
    pub inventory: Inventory,
    pub skus: SkuCatalog,

    // Entities
    pub robots: HashMap<RobotId, Robot>,
    pub stations: HashMap<StationId, Station>,
    pub orders: HashMap<OrderId, Order>,
    pub tasks: HashMap<TaskId, Task>,

    // v1: Charging infrastructure
    pub charging_stations: HashMap<ChargingStationId, ChargingStation>,

    // v3: Maintenance infrastructure
    pub maintenance_stations: HashMap<MaintenanceStationId, MaintenanceStation>,

    // Pending work queues
    pub pending_tasks: Vec<TaskId>,

    // ID generators
    pub order_id_gen: IdGenerator<OrderId>,
    pub task_id_gen: IdGenerator<TaskId>,
    pub charging_id_gen: IdGenerator<ChargingStationId>,
    pub maintenance_id_gen: IdGenerator<MaintenanceStationId>,

    // Policies
    pub policies: PolicySet,

    // Distributions
    pub distributions: DistributionSet,

    // v1: Time-series metrics
    pub time_series: TimeSeriesCollector,

    // Configuration
    pub due_time_offset_min: Option<f64>,
    /// v1: Metrics sample interval in seconds
    pub metrics_sample_interval_s: f64,

    /// v2: Deadlock resolution policy
    pub deadlock_resolver: Box<dyn DeadlockResolver>,

    /// v2: Reservation-based traffic control
    pub reservation_manager: ReservationManager,

    /// v3: Event trace collector for debugging
    pub trace_collector: EventTraceCollector,

    /// v5: Attribution collector for RCA
    pub attribution_collector: AttributionCollector,
}

impl World {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: SimRng::new(seed),
            map: WarehouseMap::new(),
            router: Router::new(true),
            traffic: TrafficManager::new(1, 1),
            racks: HashMap::new(),
            inventory: Inventory::new(),
            skus: SkuCatalog::new(),
            robots: HashMap::new(),
            stations: HashMap::new(),
            orders: HashMap::new(),
            tasks: HashMap::new(),
            charging_stations: HashMap::new(),
            maintenance_stations: HashMap::new(),
            pending_tasks: Vec::new(),
            order_id_gen: IdGenerator::new(),
            task_id_gen: IdGenerator::new(),
            charging_id_gen: IdGenerator::new(),
            maintenance_id_gen: IdGenerator::new(),
            policies: PolicySet::default(),
            distributions: DistributionSet::default(),
            time_series: TimeSeriesCollector::new(60.0), // Default 60s sample interval
            due_time_offset_min: Some(60.0),
            metrics_sample_interval_s: 60.0,
            deadlock_resolver: Box::new(YoungestRobotBacksUp::new()),
            reservation_manager: ReservationManager::new(),
            trace_collector: EventTraceCollector::default(),
            attribution_collector: AttributionCollector::new(),
        }
    }

    /// Get a charging station by ID
    pub fn get_charging_station(&self, id: ChargingStationId) -> Option<&ChargingStation> {
        self.charging_stations.get(&id)
    }

    /// Get a mutable charging station by ID
    pub fn get_charging_station_mut(
        &mut self,
        id: ChargingStationId,
    ) -> Option<&mut ChargingStation> {
        self.charging_stations.get_mut(&id)
    }

    /// Find the nearest charging station with available capacity
    pub fn find_nearest_charging_station(
        &mut self,
        from_node: NodeId,
    ) -> Option<ChargingStationId> {
        let mut best: Option<(ChargingStationId, f64)> = None;

        for (id, station) in &self.charging_stations {
            // Check if station has capacity (queue not full or can charge)
            if !station.can_accept() {
                continue;
            }

            // Calculate distance
            if let Some(route) = self.router.find_route(&self.map, from_node, station.node) {
                let dist = route.total_distance;
                if best.is_none() || dist < best.unwrap().1 {
                    best = Some((*id, dist));
                }
            }
        }

        best.map(|(id, _)| id)
    }

    /// Generate the next charging station ID
    pub fn next_charging_id(&mut self) -> ChargingStationId {
        self.charging_id_gen.next_id()
    }

    // === v3: Maintenance station helpers ===

    /// Get a maintenance station by ID
    pub fn get_maintenance_station(&self, id: MaintenanceStationId) -> Option<&MaintenanceStation> {
        self.maintenance_stations.get(&id)
    }

    /// Get a mutable maintenance station by ID
    pub fn get_maintenance_station_mut(
        &mut self,
        id: MaintenanceStationId,
    ) -> Option<&mut MaintenanceStation> {
        self.maintenance_stations.get_mut(&id)
    }

    /// Find the nearest maintenance station with available capacity
    pub fn find_nearest_maintenance_station(
        &mut self,
        from_node: NodeId,
    ) -> Option<MaintenanceStationId> {
        let mut best: Option<(MaintenanceStationId, f64)> = None;

        for (id, station) in &self.maintenance_stations {
            // Check if station has capacity (queue not full or can service)
            if !station.can_accept() {
                continue;
            }

            // Calculate distance
            if let Some(route) = self.router.find_route(&self.map, from_node, station.node) {
                let dist = route.total_distance;
                if best.is_none() || dist < best.unwrap().1 {
                    best = Some((*id, dist));
                }
            }
        }

        best.map(|(id, _)| id)
    }

    /// Generate the next maintenance station ID
    pub fn next_maintenance_id(&mut self) -> MaintenanceStationId {
        self.maintenance_id_gen.next_id()
    }

    /// Create a PolicyContext from current world state for policy decisions
    pub fn policy_context(&self, current_time: SimTime) -> PolicyContext<'_> {
        PolicyContext {
            current_time,
            map: &self.map,
            robots: &self.robots,
            tasks: &self.tasks,
            stations: &self.stations,
            orders: &self.orders,
        }
    }

    pub fn next_order_id(&mut self) -> OrderId {
        self.order_id_gen.next_id()
    }

    pub fn next_task_id(&mut self) -> TaskId {
        self.task_id_gen.next_id()
    }

    pub fn get_robot(&self, id: RobotId) -> Option<&Robot> {
        self.robots.get(&id)
    }

    pub fn get_robot_mut(&mut self, id: RobotId) -> Option<&mut Robot> {
        self.robots.get_mut(&id)
    }

    pub fn get_station(&self, id: StationId) -> Option<&Station> {
        self.stations.get(&id)
    }

    pub fn get_station_mut(&mut self, id: StationId) -> Option<&mut Station> {
        self.stations.get_mut(&id)
    }

    pub fn get_task(&self, id: TaskId) -> Option<&Task> {
        self.tasks.get(&id)
    }

    pub fn get_task_mut(&mut self, id: TaskId) -> Option<&mut Task> {
        self.tasks.get_mut(&id)
    }

    pub fn get_order(&self, id: OrderId) -> Option<&Order> {
        self.orders.get(&id)
    }

    pub fn get_order_mut(&mut self, id: OrderId) -> Option<&mut Order> {
        self.orders.get_mut(&id)
    }

    pub fn idle_robots(&self) -> impl Iterator<Item = &Robot> {
        self.robots.values().filter(|r| r.is_available())
    }

    pub fn pick_stations(&self) -> impl Iterator<Item = &Station> {
        self.stations
            .values()
            .filter(|s| s.station_type == waremax_entities::StationType::Pick)
    }

    /// Initialize demo inventory with SKUs and stock placements
    /// This creates SKUs and places inventory at rack/storage nodes
    pub fn init_demo_inventory(&mut self, num_skus: u32) {
        // Create SKUs
        for i in 0..num_skus {
            let sku = Sku::new(SkuId(i), format!("SKU-{:04}", i), 2.0);
            self.skus.add(sku);
        }

        // Find rack/storage nodes in the map and create racks
        let rack_nodes: Vec<(NodeId, String)> = self
            .map
            .nodes
            .iter()
            .filter(|(_, node)| matches!(node.node_type, NodeType::Rack | NodeType::Staging))
            .map(|(id, node)| (*id, node.string_id.clone()))
            .collect();

        // If no rack nodes found, use interior aisle nodes as storage
        let storage_nodes: Vec<(NodeId, String)> = if rack_nodes.is_empty() {
            self.map
                .nodes
                .iter()
                .filter(|(_, node)| matches!(node.node_type, NodeType::Aisle))
                .take(10) // Use up to 10 aisle nodes as storage
                .map(|(id, node)| (*id, node.string_id.clone()))
                .collect()
        } else {
            rack_nodes
        };

        // Create racks and place inventory
        for (idx, (node_id, node_string_id)) in storage_nodes.iter().enumerate() {
            let rack_id = RackId(idx as u32);
            let rack = Rack::new(
                rack_id,
                format!("RACK-{}", node_string_id),
                *node_id,
                3, // 3 levels
                4, // 4 bins per level
            );
            self.racks.insert(rack_id, rack);

            // Place some SKUs in this rack's bins
            for level in 0..3 {
                for bin in 0..4 {
                    // Assign a SKU to this bin (distribute SKUs across bins)
                    let sku_idx = ((idx * 12) + (level * 4) + bin) as u32 % num_skus;
                    let bin_addr = BinAddress::new(rack_id, level as u32, bin as u32);
                    let quantity = 10 + self.rng.gen_range(1..=20u32); // 10-30 units
                    self.inventory
                        .add_placement(bin_addr, SkuId(sku_idx), quantity);
                }
            }
        }
    }

    /// Get the first pick station (for assigning tasks)
    pub fn first_pick_station(&self) -> Option<StationId> {
        self.pick_stations().next().map(|s| s.id)
    }

    /// Find a bin location for a SKU with available stock
    pub fn find_sku_location(&self, sku_id: SkuId, quantity: u32) -> Option<(BinAddress, NodeId)> {
        if let Some(bin_addr) = self.inventory.find_sku_with_stock(sku_id, quantity) {
            // Find the rack to get its access node
            if let Some(rack) = self.racks.get(&bin_addr.rack_id) {
                return Some((bin_addr.clone(), rack.access_node));
            }
        }
        None
    }
}
