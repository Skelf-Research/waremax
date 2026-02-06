//! Graph-based warehouse map representation

use rkyv::{Archive, Deserialize, Serialize};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use std::collections::HashMap;
use waremax_core::{EdgeId, NodeId};

/// Direction of travel allowed on an edge (v2)
#[derive(
    Archive,
    Deserialize,
    Serialize,
    SerdeDeserialize,
    SerdeSerialize,
    Clone,
    Debug,
    PartialEq,
    Default,
)]
pub enum EdgeDirection {
    /// Traffic can flow in both directions
    #[default]
    Bidirectional,
    /// Traffic can only flow from->to (one-way aisle)
    OneWay,
}

/// Node type in the warehouse map
#[derive(
    Archive, Deserialize, Serialize, SerdeDeserialize, SerdeSerialize, Clone, Debug, PartialEq, Default,
)]
pub enum NodeType {
    #[default]
    Aisle,
    StationPick,
    StationDrop,
    StationInbound,
    StationOutbound,
    Charging,
    Staging,
    Rack,
}

/// A node in the warehouse map
#[derive(Archive, Deserialize, Serialize, Clone, Debug)]
pub struct Node {
    pub id: NodeId,
    pub string_id: String,
    pub x: f64,
    pub y: f64,
    pub node_type: NodeType,
    pub capacity: u32,
}

impl Node {
    pub fn new(id: NodeId, string_id: String, x: f64, y: f64, node_type: NodeType) -> Self {
        Self {
            id,
            string_id,
            x,
            y,
            node_type,
            capacity: 1,
        }
    }
}

/// An edge in the warehouse map
#[derive(Archive, Deserialize, Serialize, Clone, Debug)]
pub struct Edge {
    pub id: EdgeId,
    pub from: NodeId,
    pub to: NodeId,
    pub length_m: f64,
    pub capacity: u32,
    /// v2: Direction of travel allowed on this edge
    pub direction: EdgeDirection,
    /// v2: Speed multiplier for routing cost calculation
    /// 1.0 = normal, <1.0 = express/faster, >1.0 = slower/restricted
    pub speed_multiplier: f64,
}

impl Edge {
    pub fn new(id: EdgeId, from: NodeId, to: NodeId, length_m: f64) -> Self {
        Self {
            id,
            from,
            to,
            length_m,
            capacity: 1,
            direction: EdgeDirection::Bidirectional,
            speed_multiplier: 1.0,
        }
    }

    /// Set the direction for this edge (builder pattern)
    pub fn with_direction(mut self, direction: EdgeDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set the capacity for this edge (builder pattern)
    pub fn with_capacity(mut self, capacity: u32) -> Self {
        self.capacity = capacity;
        self
    }

    /// Set the speed multiplier for this edge (builder pattern)
    /// Values < 1.0 make the edge faster (express lane)
    /// Values > 1.0 make the edge slower (restricted)
    pub fn with_speed_multiplier(mut self, multiplier: f64) -> Self {
        self.speed_multiplier = multiplier;
        self
    }
}

/// The warehouse map graph
#[derive(Clone, Default)]
pub struct WarehouseMap {
    pub nodes: HashMap<NodeId, Node>,
    pub edges: HashMap<EdgeId, Edge>,
    pub adjacency: HashMap<NodeId, Vec<(NodeId, EdgeId, f64)>>,
    pub string_to_node: HashMap<String, NodeId>,
    pub blocked_nodes: Vec<NodeId>,
    pub blocked_edges: Vec<EdgeId>,
}

impl WarehouseMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: Node) {
        let id = node.id;
        let string_id = node.string_id.clone();
        self.adjacency.entry(id).or_default();
        self.string_to_node.insert(string_id, id);
        self.nodes.insert(id, node);
    }

    /// Add an edge to the map
    ///
    /// If the edge direction is Bidirectional, a reverse edge is automatically created.
    /// If OneWay, only the forward direction is added.
    pub fn add_edge(&mut self, edge: Edge) {
        let from = edge.from;
        let to = edge.to;
        let length = edge.length_m;
        let edge_id = edge.id;
        let direction = edge.direction.clone();

        self.adjacency
            .entry(from)
            .or_default()
            .push((to, edge_id, length));

        if direction == EdgeDirection::Bidirectional {
            let reverse_id = EdgeId(edge_id.0 + 100000);
            self.adjacency
                .entry(to)
                .or_default()
                .push((from, reverse_id, length));
            self.edges.insert(
                reverse_id,
                Edge {
                    id: reverse_id,
                    from: to,
                    to: from,
                    length_m: length,
                    capacity: edge.capacity,
                    direction: EdgeDirection::OneWay, // Reverse edge is one-way
                    speed_multiplier: edge.speed_multiplier, // Copy speed multiplier
                },
            );
        }

        self.edges.insert(edge_id, edge);
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn get_node_by_string(&self, s: &str) -> Option<&Node> {
        self.string_to_node.get(s).and_then(|id| self.nodes.get(id))
    }

    pub fn get_edge(&self, id: EdgeId) -> Option<&Edge> {
        self.edges.get(&id)
    }

    pub fn neighbors(&self, node: NodeId) -> impl Iterator<Item = (NodeId, EdgeId, f64)> + '_ {
        self.adjacency
            .get(&node)
            .into_iter()
            .flat_map(|v| v.iter().copied())
            .filter(move |(neighbor, edge_id, _)| {
                !self.blocked_nodes.contains(neighbor) && !self.blocked_edges.contains(edge_id)
            })
    }

    pub fn euclidean_distance(&self, from: NodeId, to: NodeId) -> f64 {
        let n1 = self.nodes.get(&from);
        let n2 = self.nodes.get(&to);
        match (n1, n2) {
            (Some(a), Some(b)) => ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt(),
            _ => f64::INFINITY,
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}
