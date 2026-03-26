//! Inventory tracking

use crate::rack::BinAddress;
use std::collections::HashMap;
use thiserror::Error;
use waremax_core::SkuId;

/// Error types for inventory operations
#[derive(Error, Debug)]
pub enum InventoryError {
    #[error("Bin not found: {0}")]
    BinNotFound(BinAddress),

    #[error("Insufficient stock at {bin}: requested {requested}, available {available}")]
    InsufficientStock {
        bin: BinAddress,
        requested: u32,
        available: u32,
    },
}

/// Inventory slot containing SKU and quantity
#[derive(Clone, Debug)]
pub struct InventorySlot {
    pub sku_id: SkuId,
    pub quantity: u32,
}

/// Inventory manager tracking stock in bins
#[derive(Clone, Default)]
pub struct Inventory {
    bins: HashMap<BinAddress, InventorySlot>,
    sku_locations: HashMap<SkuId, Vec<BinAddress>>,
    /// All known bin addresses (including empty ones)
    all_bins: Vec<BinAddress>,
    /// Replenishment thresholds by SKU
    replen_thresholds: HashMap<SkuId, u32>,
}

impl Inventory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a bin address as available for inventory
    pub fn register_bin(&mut self, address: BinAddress) {
        if !self.all_bins.contains(&address) {
            self.all_bins.push(address);
        }
    }

    /// Set replenishment threshold for a SKU
    pub fn set_replen_threshold(&mut self, sku_id: SkuId, threshold: u32) {
        self.replen_thresholds.insert(sku_id, threshold);
    }

    /// Get replenishment threshold for a SKU
    pub fn get_replen_threshold(&self, sku_id: SkuId) -> Option<u32> {
        self.replen_thresholds.get(&sku_id).copied()
    }

    pub fn add_placement(&mut self, address: BinAddress, sku_id: SkuId, quantity: u32) {
        self.bins
            .insert(address.clone(), InventorySlot { sku_id, quantity });
        self.sku_locations
            .entry(sku_id)
            .or_default()
            .push(address.clone());
        self.register_bin(address);
    }

    pub fn get_slot(&self, address: &BinAddress) -> Option<&InventorySlot> {
        self.bins.get(address)
    }

    pub fn get_quantity(&self, address: &BinAddress) -> Option<u32> {
        self.bins.get(address).map(|s| s.quantity)
    }

    pub fn find_sku(&self, sku_id: SkuId) -> impl Iterator<Item = &BinAddress> {
        self.sku_locations
            .get(&sku_id)
            .into_iter()
            .flat_map(|v| v.iter())
    }

    pub fn find_sku_with_stock(&self, sku_id: SkuId, min_qty: u32) -> Option<&BinAddress> {
        self.sku_locations
            .get(&sku_id)?
            .iter()
            .find(|addr| self.bins.get(*addr).is_some_and(|s| s.quantity >= min_qty))
    }

    pub fn decrement(&mut self, address: &BinAddress, qty: u32) -> Result<(), InventoryError> {
        let slot = self
            .bins
            .get_mut(address)
            .ok_or_else(|| InventoryError::BinNotFound(address.clone()))?;

        if slot.quantity < qty {
            return Err(InventoryError::InsufficientStock {
                bin: address.clone(),
                requested: qty,
                available: slot.quantity,
            });
        }

        slot.quantity -= qty;
        Ok(())
    }

    pub fn increment(&mut self, address: &BinAddress, qty: u32) -> Result<(), InventoryError> {
        let slot = self
            .bins
            .get_mut(address)
            .ok_or_else(|| InventoryError::BinNotFound(address.clone()))?;
        slot.quantity += qty;
        Ok(())
    }

    pub fn total_quantity(&self, sku_id: SkuId) -> u32 {
        self.sku_locations
            .get(&sku_id)
            .map(|addrs| {
                addrs
                    .iter()
                    .filter_map(|addr| self.bins.get(addr))
                    .map(|slot| slot.quantity)
                    .sum()
            })
            .unwrap_or(0)
    }

    /// Get all bins that are empty (no inventory or zero quantity)
    pub fn get_empty_bins(&self) -> Vec<&BinAddress> {
        self.all_bins
            .iter()
            .filter(|addr| self.bins.get(*addr).is_none_or(|slot| slot.quantity == 0))
            .collect()
    }

    /// Get all registered bins
    pub fn all_bins(&self) -> &[BinAddress] {
        &self.all_bins
    }

    /// Check if a SKU needs replenishment (below threshold)
    pub fn needs_replenishment(&self, sku_id: SkuId) -> Option<(BinAddress, u32, u32)> {
        let threshold = self.replen_thresholds.get(&sku_id)?;

        // Find the first bin with this SKU that's below threshold
        for addr in self.sku_locations.get(&sku_id)? {
            if let Some(slot) = self.bins.get(addr) {
                if slot.quantity < *threshold {
                    return Some((addr.clone(), slot.quantity, *threshold));
                }
            }
        }
        None
    }

    /// Get all SKUs that need replenishment
    pub fn get_replenishment_needed(&self) -> Vec<(SkuId, BinAddress, u32, u32)> {
        let mut results = Vec::new();
        for (&sku_id, &threshold) in &self.replen_thresholds {
            if let Some(locations) = self.sku_locations.get(&sku_id) {
                for addr in locations {
                    if let Some(slot) = self.bins.get(addr) {
                        if slot.quantity < threshold {
                            results.push((sku_id, addr.clone(), slot.quantity, threshold));
                        }
                    }
                }
            }
        }
        results
    }

    /// Create a new empty bin slot (for putaway destination)
    pub fn create_empty_slot(&mut self, address: BinAddress, sku_id: SkuId) {
        self.bins.insert(
            address.clone(),
            InventorySlot {
                sku_id,
                quantity: 0,
            },
        );
        self.sku_locations
            .entry(sku_id)
            .or_default()
            .push(address.clone());
        self.register_bin(address);
    }
}
