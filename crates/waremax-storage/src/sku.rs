//! SKU definitions and catalog

use rkyv::{Archive, Deserialize, Serialize};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use std::collections::HashMap;
use waremax_core::SkuId;

/// SKU (Stock Keeping Unit) definition
#[derive(Archive, Deserialize, Serialize, SerdeDeserialize, SerdeSerialize, Clone, Debug)]
pub struct Sku {
    pub id: SkuId,
    pub string_id: String,
    pub unit_pick_time_s: f64,
    pub weight_kg: Option<f64>,
}

impl Sku {
    pub fn new(id: SkuId, string_id: String, unit_pick_time_s: f64) -> Self {
        Self {
            id,
            string_id,
            unit_pick_time_s,
            weight_kg: None,
        }
    }
}

/// Catalog of all SKUs
#[derive(Clone, Default)]
pub struct SkuCatalog {
    skus: HashMap<SkuId, Sku>,
    string_to_id: HashMap<String, SkuId>,
}

impl SkuCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, sku: Sku) {
        let id = sku.id;
        let string_id = sku.string_id.clone();
        self.skus.insert(id, sku);
        self.string_to_id.insert(string_id, id);
    }

    pub fn get(&self, id: SkuId) -> Option<&Sku> {
        self.skus.get(&id)
    }

    pub fn by_string(&self, s: &str) -> Option<SkuId> {
        self.string_to_id.get(s).copied()
    }

    pub fn count(&self) -> usize {
        self.skus.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Sku> {
        self.skus.values()
    }

    pub fn ids(&self) -> impl Iterator<Item = SkuId> + '_ {
        self.skus.keys().copied()
    }
}
