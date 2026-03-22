//! Rack and bin storage structures

use rkyv::{Archive, Deserialize, Serialize};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use waremax_core::{NodeId, RackId};

/// A storage rack in the warehouse
#[derive(Archive, Deserialize, Serialize, Clone, Debug)]
pub struct Rack {
    pub id: RackId,
    pub string_id: String,
    pub access_node: NodeId,
    pub levels: u32,
    pub bins_per_level: u32,
    pub zone: Option<String>,
    /// Base time to access level 0 (seconds)
    pub base_access_time_s: f64,
    /// Additional time per level above 0 (seconds)
    pub per_level_time_s: f64,
}

impl Rack {
    pub fn new(
        id: RackId,
        string_id: String,
        access_node: NodeId,
        levels: u32,
        bins_per_level: u32,
    ) -> Self {
        Self {
            id,
            string_id,
            access_node,
            levels,
            bins_per_level,
            zone: None,
            base_access_time_s: 0.0,
            per_level_time_s: 0.0,
        }
    }

    /// Create a rack with level-specific access times
    pub fn with_access_times(mut self, base_access_time_s: f64, per_level_time_s: f64) -> Self {
        self.base_access_time_s = base_access_time_s;
        self.per_level_time_s = per_level_time_s;
        self
    }

    pub fn total_bins(&self) -> u32 {
        self.levels * self.bins_per_level
    }

    pub fn bin_exists(&self, level: u32, bin: u32) -> bool {
        level < self.levels && bin < self.bins_per_level
    }

    /// Calculate the access time for a specific level
    ///
    /// Higher levels take longer to access (e.g., reaching higher shelves)
    pub fn access_time(&self, level: u32) -> f64 {
        self.base_access_time_s + (level as f64 * self.per_level_time_s)
    }
}

/// Address of a specific bin within a rack
#[derive(
    Archive,
    Deserialize,
    Serialize,
    SerdeDeserialize,
    SerdeSerialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
)]
pub struct BinAddress {
    pub rack_id: RackId,
    pub level: u32,
    pub bin: u32,
}

impl BinAddress {
    pub fn new(rack_id: RackId, level: u32, bin: u32) -> Self {
        Self {
            rack_id,
            level,
            bin,
        }
    }
}

impl std::fmt::Display for BinAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "R{}-L{}-B{}", self.rack_id.0, self.level, self.bin)
    }
}
