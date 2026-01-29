//! Rack and bin storage structures

use rkyv::{Archive, Deserialize, Serialize};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use waremax_core::{RackId, NodeId};

/// A storage rack in the warehouse
#[derive(Archive, Deserialize, Serialize, Clone, Debug)]
pub struct Rack {
    pub id: RackId,
    pub string_id: String,
    pub access_node: NodeId,
    pub levels: u32,
    pub bins_per_level: u32,
    pub zone: Option<String>,
}

impl Rack {
    pub fn new(id: RackId, string_id: String, access_node: NodeId, levels: u32, bins_per_level: u32) -> Self {
        Self {
            id,
            string_id,
            access_node,
            levels,
            bins_per_level,
            zone: None,
        }
    }

    pub fn total_bins(&self) -> u32 {
        self.levels * self.bins_per_level
    }

    pub fn bin_exists(&self, level: u32, bin: u32) -> bool {
        level < self.levels && bin < self.bins_per_level
    }
}

/// Address of a specific bin within a rack
#[derive(Archive, Deserialize, Serialize, SerdeDeserialize, SerdeSerialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BinAddress {
    pub rack_id: RackId,
    pub level: u32,
    pub bin: u32,
}

impl BinAddress {
    pub fn new(rack_id: RackId, level: u32, bin: u32) -> Self {
        Self { rack_id, level, bin }
    }
}

impl std::fmt::Display for BinAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "R{}-L{}-B{}", self.rack_id.0, self.level, self.bin)
    }
}
