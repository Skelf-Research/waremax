//! Traffic management for edge and node capacity

use waremax_core::{NodeId, EdgeId, RobotId};
use std::collections::{HashMap, HashSet};

/// Manages traffic flow and capacity constraints in the warehouse
pub struct TrafficManager {
    edge_occupancy: HashMap<EdgeId, HashSet<RobotId>>,
    node_occupancy: HashMap<NodeId, HashSet<RobotId>>,
    edge_capacity: HashMap<EdgeId, u32>,
    node_capacity: HashMap<NodeId, u32>,
    default_edge_capacity: u32,
    default_node_capacity: u32,
}

impl TrafficManager {
    pub fn new(default_edge_capacity: u32, default_node_capacity: u32) -> Self {
        Self {
            edge_occupancy: HashMap::new(),
            node_occupancy: HashMap::new(),
            edge_capacity: HashMap::new(),
            node_capacity: HashMap::new(),
            default_edge_capacity,
            default_node_capacity,
        }
    }

    pub fn set_edge_capacity(&mut self, edge: EdgeId, capacity: u32) {
        self.edge_capacity.insert(edge, capacity);
    }

    pub fn set_node_capacity(&mut self, node: NodeId, capacity: u32) {
        self.node_capacity.insert(node, capacity);
    }

    pub fn can_enter_edge(&self, edge: EdgeId, robot: RobotId) -> bool {
        let capacity = self.edge_capacity.get(&edge).copied().unwrap_or(self.default_edge_capacity);
        let occupants = self.edge_occupancy.get(&edge);

        if let Some(set) = occupants {
            if set.contains(&robot) {
                return true;
            }
            (set.len() as u32) < capacity
        } else {
            capacity > 0
        }
    }

    pub fn can_enter_node(&self, node: NodeId, robot: RobotId) -> bool {
        let capacity = self.node_capacity.get(&node).copied().unwrap_or(self.default_node_capacity);
        let occupants = self.node_occupancy.get(&node);

        if let Some(set) = occupants {
            if set.contains(&robot) {
                return true;
            }
            (set.len() as u32) < capacity
        } else {
            capacity > 0
        }
    }

    pub fn enter_edge(&mut self, edge: EdgeId, robot: RobotId) {
        self.edge_occupancy.entry(edge).or_default().insert(robot);
    }

    pub fn leave_edge(&mut self, edge: EdgeId, robot: RobotId) {
        if let Some(set) = self.edge_occupancy.get_mut(&edge) {
            set.remove(&robot);
        }
    }

    pub fn enter_node(&mut self, node: NodeId, robot: RobotId) {
        self.node_occupancy.entry(node).or_default().insert(robot);
    }

    pub fn leave_node(&mut self, node: NodeId, robot: RobotId) {
        if let Some(set) = self.node_occupancy.get_mut(&node) {
            set.remove(&robot);
        }
    }

    pub fn get_edge_occupancy(&self, edge: EdgeId) -> usize {
        self.edge_occupancy.get(&edge).map_or(0, |s| s.len())
    }

    pub fn get_node_occupancy(&self, node: NodeId) -> usize {
        self.node_occupancy.get(&node).map_or(0, |s| s.len())
    }

    pub fn robots_on_edge(&self, edge: EdgeId) -> impl Iterator<Item = RobotId> + '_ {
        self.edge_occupancy.get(&edge).into_iter().flat_map(|s| s.iter().copied())
    }

    pub fn robots_at_node(&self, node: NodeId) -> impl Iterator<Item = RobotId> + '_ {
        self.node_occupancy.get(&node).into_iter().flat_map(|s| s.iter().copied())
    }
}
