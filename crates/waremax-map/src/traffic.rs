//! Traffic management for edge and node capacity

use crate::deadlock::{WaitForGraph, WaitingFor};
use crate::position::ContinuousEdgeState;
use std::collections::{HashMap, HashSet};
use waremax_core::{EdgeId, NodeId, RobotId};

/// Manages traffic flow and capacity constraints in the warehouse
pub struct TrafficManager {
    edge_occupancy: HashMap<EdgeId, HashSet<RobotId>>,
    node_occupancy: HashMap<NodeId, HashSet<RobotId>>,
    edge_capacity: HashMap<EdgeId, u32>,
    node_capacity: HashMap<NodeId, u32>,
    default_edge_capacity: u32,
    default_node_capacity: u32,
    /// v2: Wait-for graph for deadlock detection
    pub wait_graph: WaitForGraph,
    /// v2: Whether deadlock detection is enabled
    pub deadlock_detection_enabled: bool,
    /// v4: Continuous position tracking per edge (only populated when continuous policy is active)
    continuous_states: HashMap<EdgeId, Vec<ContinuousEdgeState>>,
    /// v4: Current travel direction per edge (from, to). None if edge is empty.
    edge_directions: HashMap<EdgeId, Option<(NodeId, NodeId)>>,
    /// v4: Edge lengths for progress calculations
    edge_lengths: HashMap<EdgeId, f64>,
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
            wait_graph: WaitForGraph::new(),
            deadlock_detection_enabled: false,
            continuous_states: HashMap::new(),
            edge_directions: HashMap::new(),
            edge_lengths: HashMap::new(),
        }
    }

    /// Enable or disable deadlock detection
    pub fn set_deadlock_detection(&mut self, enabled: bool) {
        self.deadlock_detection_enabled = enabled;
    }

    pub fn set_edge_capacity(&mut self, edge: EdgeId, capacity: u32) {
        self.edge_capacity.insert(edge, capacity);
    }

    pub fn set_node_capacity(&mut self, node: NodeId, capacity: u32) {
        self.node_capacity.insert(node, capacity);
    }

    pub fn can_enter_edge(&self, edge: EdgeId, robot: RobotId) -> bool {
        let capacity = self
            .edge_capacity
            .get(&edge)
            .copied()
            .unwrap_or(self.default_edge_capacity);
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
        let capacity = self
            .node_capacity
            .get(&node)
            .copied()
            .unwrap_or(self.default_node_capacity);
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
        self.edge_occupancy
            .get(&edge)
            .into_iter()
            .flat_map(|s| s.iter().copied())
    }

    pub fn robots_at_node(&self, node: NodeId) -> impl Iterator<Item = RobotId> + '_ {
        self.node_occupancy
            .get(&node)
            .into_iter()
            .flat_map(|s| s.iter().copied())
    }

    // === v2: Deadlock Detection Methods ===

    /// Record that a robot is waiting for an edge
    ///
    /// Automatically determines which robots are blocking the edge.
    pub fn record_edge_wait(&mut self, robot: RobotId, edge: EdgeId) {
        if !self.deadlock_detection_enabled {
            return;
        }

        let blockers: Vec<RobotId> = self.robots_on_edge(edge).filter(|&r| r != robot).collect();

        self.wait_graph.add_wait(
            robot,
            WaitingFor::Edge {
                edge_id: edge,
                blocked_by: blockers,
            },
        );
    }

    /// Record that a robot is waiting for a node
    ///
    /// Automatically determines which robots are blocking the node.
    pub fn record_node_wait(&mut self, robot: RobotId, node: NodeId) {
        if !self.deadlock_detection_enabled {
            return;
        }

        let blockers: Vec<RobotId> = self.robots_at_node(node).filter(|&r| r != robot).collect();

        self.wait_graph.add_wait(
            robot,
            WaitingFor::Node {
                node_id: node,
                blocked_by: blockers,
            },
        );
    }

    /// Clear a robot's wait status (e.g., when it acquires the resource)
    pub fn clear_wait(&mut self, robot: RobotId) {
        self.wait_graph.remove_wait(robot);
    }

    /// Check if a robot is currently waiting
    pub fn is_waiting(&self, robot: RobotId) -> bool {
        self.wait_graph.is_waiting(robot)
    }

    /// Check for deadlocks in the current wait-for graph
    ///
    /// Returns Some(cycle) if a deadlock is detected, None otherwise.
    pub fn check_deadlock(&self) -> Option<Vec<RobotId>> {
        if !self.deadlock_detection_enabled {
            return None;
        }
        self.wait_graph.detect_cycle()
    }

    /// Get all robots currently waiting
    pub fn waiting_count(&self) -> usize {
        self.wait_graph.waiting_count()
    }

    // === v4: Continuous Position Tracking Methods ===

    /// Register edge length for progress calculations.
    pub fn register_edge_length(&mut self, edge: EdgeId, length_m: f64) {
        self.edge_lengths.insert(edge, length_m);
    }

    /// Get the registered length of an edge.
    pub fn edge_length(&self, edge: EdgeId) -> f64 {
        self.edge_lengths.get(&edge).copied().unwrap_or(1.0)
    }

    /// Get the current travel direction of an edge, if any.
    pub fn edge_direction(&self, edge: EdgeId) -> Option<Option<(NodeId, NodeId)>> {
        self.edge_directions.get(&edge).copied()
    }

    /// Set the travel direction for an edge.
    pub fn set_edge_direction(&mut self, edge: EdgeId, direction: Option<(NodeId, NodeId)>) {
        self.edge_directions.insert(edge, direction);
    }

    /// Get the continuous edge states for a given edge.
    pub fn continuous_states(&self, edge: EdgeId) -> Option<&Vec<ContinuousEdgeState>> {
        self.continuous_states.get(&edge)
    }

    /// Get a mutable reference to continuous edge states for a given edge.
    pub fn continuous_states_mut(&mut self, edge: EdgeId) -> &mut Vec<ContinuousEdgeState> {
        self.continuous_states.entry(edge).or_default()
    }

    /// Add a continuous edge state for a robot entering an edge.
    pub fn add_continuous_state(&mut self, edge: EdgeId, state: ContinuousEdgeState) {
        self.continuous_states.entry(edge).or_default().push(state);
    }

    /// Update the progress of a robot on an edge.
    pub fn update_continuous_progress(
        &mut self,
        edge: EdgeId,
        robot: RobotId,
        progress: f64,
    ) {
        if let Some(states) = self.continuous_states.get_mut(&edge) {
            for state in states.iter_mut() {
                if state.robot_id == robot {
                    state.progress = progress.clamp(0.0, 1.0);
                    break;
                }
            }
        }
    }

    /// Remove a robot's continuous state from an edge.
    pub fn remove_continuous_state(&mut self,
        edge: EdgeId,
        robot: RobotId,
    ) {
        if let Some(states) = self.continuous_states.get_mut(&edge) {
            states.retain(|s| s.robot_id != robot);
            if states.is_empty() {
                self.continuous_states.remove(&edge);
                self.edge_directions.remove(&edge);
            }
        }
    }

    /// Count occupants on an edge (backward-compatible alias).
    pub fn edge_occupancy_count(&self, edge: EdgeId) -> usize {
        self.get_edge_occupancy(edge)
    }

    /// Get the capacity of an edge.
    pub fn edge_capacity(&self, edge: EdgeId) -> u32 {
        self.edge_capacity.get(&edge).copied().unwrap_or(self.default_edge_capacity)
    }
}
