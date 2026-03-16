//! Graph-based warehouse map representation

use rkyv::{Archive, Deserialize, Serialize};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use waremax_core::{NodeId, EdgeId};
use std::collections::HashMap;

/// Node type in the warehouse map
#[derive(Archive, Deserialize, Serialize, SerdeDeserialize, SerdeSerialize, Clone, Debug, PartialEq)]
pub enum NodeType {
    Aisle,
    StationPick,
    StationDrop,
    StationInbound,
    StationOutbound,
    Charging,
    Staging,
    Rack,
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::Aisle
    }
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
}

impl Edge {
    pub fn new(id: EdgeId, from: NodeId, to: NodeId, length_m: f64) -> Self {
        Self {
            id,
            from,
            to,
            length_m,
            capacity: 1,
        }
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

    pub fn add_edge(&mut self, edge: Edge, bidirectional: bool) {
        let from = edge.from;
        let to = edge.to;
        let length = edge.length_m;
        let edge_id = edge.id;

        self.adjacency.entry(from).or_default().push((to, edge_id, length));

        if bidirectional {
            let reverse_id = EdgeId(edge_id.0 + 100000);
            self.adjacency.entry(to).or_default().push((from, reverse_id, length));
            self.edges.insert(
                reverse_id,
                Edge {
                    id: reverse_id,
                    from: to,
                    to: from,
                    length_m: length,
                    capacity: edge.capacity,
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
