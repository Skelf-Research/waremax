//! Destination policies for bin selection in putaway/replenishment tasks

use std::collections::HashMap;
use waremax_core::{NodeId, RackId, SkuId};
use waremax_map::WarehouseMap;
use waremax_storage::inventory::Inventory;
use waremax_storage::rack::BinAddress;

/// Context for destination bin selection
pub struct DestinationContext<'a> {
    pub map: &'a WarehouseMap,
    pub inventory: &'a Inventory,
    pub rack_access_nodes: &'a HashMap<RackId, NodeId>,
    pub robot_location: NodeId,
}

/// Policy for selecting destination bins for putaway tasks
pub trait DestinationPolicy: Send + Sync {
    /// Select a bin for storing the given SKU
    fn select_bin(
        &self,
        ctx: &DestinationContext,
        sku_id: SkuId,
        quantity: u32,
    ) -> Option<BinAddress>;

    /// Policy name for logging
    fn name(&self) -> &'static str;
}

/// Select the nearest empty bin for putaway
pub struct NearestEmptyBinPolicy;

impl NearestEmptyBinPolicy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NearestEmptyBinPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl DestinationPolicy for NearestEmptyBinPolicy {
    fn select_bin(
        &self,
        ctx: &DestinationContext,
        _sku_id: SkuId,
        _quantity: u32,
    ) -> Option<BinAddress> {
        // Find all empty bins and select the nearest one
        let empty_bins = ctx.inventory.get_empty_bins();

        if empty_bins.is_empty() {
            return None;
        }

        // Find nearest bin by access node distance
        let mut best_bin = None;
        let mut best_distance = f64::MAX;

        for bin_addr in empty_bins {
            if let Some(&access_node) = ctx.rack_access_nodes.get(&bin_addr.rack_id) {
                let distance = ctx.map.euclidean_distance(ctx.robot_location, access_node);
                if distance < best_distance {
                    best_distance = distance;
                    best_bin = Some(bin_addr.clone());
                }
            }
        }

        best_bin
    }

    fn name(&self) -> &'static str {
        "nearest_empty_bin"
    }
}

/// Select a bin that already contains the same SKU (for consolidation)
pub struct ConsolidateBinPolicy {
    /// Maximum fill ratio before skipping a bin
    max_fill_ratio: f64,
    /// Bin capacities indexed by rack
    bin_capacity: u32,
}

impl ConsolidateBinPolicy {
    pub fn new(max_fill_ratio: f64, bin_capacity: u32) -> Self {
        Self {
            max_fill_ratio,
            bin_capacity,
        }
    }
}

impl Default for ConsolidateBinPolicy {
    fn default() -> Self {
        Self::new(0.9, 100)
    }
}

impl DestinationPolicy for ConsolidateBinPolicy {
    fn select_bin(
        &self,
        ctx: &DestinationContext,
        sku_id: SkuId,
        quantity: u32,
    ) -> Option<BinAddress> {
        // First try to find a bin with the same SKU that has space
        for bin_addr in ctx.inventory.find_sku(sku_id) {
            if let Some(current_qty) = ctx.inventory.get_quantity(bin_addr) {
                let fill_ratio = current_qty as f64 / self.bin_capacity as f64;

                if fill_ratio < self.max_fill_ratio && (current_qty + quantity) <= self.bin_capacity
                {
                    return Some(bin_addr.clone());
                }
            }
        }

        // Fallback to nearest empty bin
        NearestEmptyBinPolicy.select_bin(ctx, sku_id, quantity)
    }

    fn name(&self) -> &'static str {
        "consolidate_bin"
    }
}
