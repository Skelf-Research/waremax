//! World state container

use waremax_core::{RobotId, NodeId, StationId, OrderId, TaskId, SkuId, RackId, SimRng, SimTime, IdGenerator};
use waremax_map::{WarehouseMap, Router, TrafficManager, NodeType};
use waremax_storage::{Inventory, SkuCatalog, Rack, Sku, BinAddress};
use waremax_entities::{Robot, Station, Order, Task};
use waremax_policies::{
    TaskAllocationPolicy, StationAssignmentPolicy, BatchingPolicy, PriorityPolicy,
    PolicyContext, NearestRobotPolicy, LeastQueuePolicy, NoBatchingPolicy, StrictPriorityPolicy,
};
use std::collections::HashMap;

use crate::distributions::DistributionSet;

/// Container for all active policies
pub struct PolicySet {
    pub task_allocation: Box<dyn TaskAllocationPolicy>,
    pub station_assignment: Box<dyn StationAssignmentPolicy>,
    pub batching: Box<dyn BatchingPolicy>,
    pub priority: Box<dyn PriorityPolicy>,
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
}

impl Default for PolicySet {
    fn default() -> Self {
        Self {
            task_allocation: Box::new(NearestRobotPolicy::new()),
            station_assignment: Box::new(LeastQueuePolicy::default()),
            batching: Box::new(NoBatchingPolicy::new()),
            priority: Box::new(StrictPriorityPolicy::new()),
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

    // Pending work queues
    pub pending_tasks: Vec<TaskId>,

    // ID generators
    pub order_id_gen: IdGenerator<OrderId>,
    pub task_id_gen: IdGenerator<TaskId>,

    // Policies
    pub policies: PolicySet,

    // Distributions
    pub distributions: DistributionSet,

    // Configuration
    pub due_time_offset_min: Option<f64>,
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
            pending_tasks: Vec::new(),
            order_id_gen: IdGenerator::new(),
            task_id_gen: IdGenerator::new(),
            policies: PolicySet::default(),
            distributions: DistributionSet::default(),
            due_time_offset_min: Some(60.0),
        }
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
        self.order_id_gen.next()
    }

    pub fn next_task_id(&mut self) -> TaskId {
        self.task_id_gen.next()
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
        self.stations.values().filter(|s| s.station_type == waremax_entities::StationType::Pick)
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
        let rack_nodes: Vec<(NodeId, String)> = self.map.nodes.iter()
            .filter(|(_, node)| matches!(node.node_type, NodeType::Rack | NodeType::Staging))
            .map(|(id, node)| (*id, node.string_id.clone()))
            .collect();

        // If no rack nodes found, use interior aisle nodes as storage
        let storage_nodes: Vec<(NodeId, String)> = if rack_nodes.is_empty() {
            self.map.nodes.iter()
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
                3,  // 3 levels
                4,  // 4 bins per level
            );
            self.racks.insert(rack_id, rack);

            // Place some SKUs in this rack's bins
            for level in 0..3 {
                for bin in 0..4 {
                    // Assign a SKU to this bin (distribute SKUs across bins)
                    let sku_idx = ((idx * 12) + (level * 4) + bin) as u32 % num_skus;
                    let bin_addr = BinAddress::new(rack_id, level as u32, bin as u32);
                    let quantity = 10 + self.rng.gen_range(1..=20u32); // 10-30 units
                    self.inventory.add_placement(bin_addr, SkuId(sku_idx), quantity);
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
