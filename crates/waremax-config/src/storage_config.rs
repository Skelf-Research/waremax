//! Storage configuration parsing

use super::scenario::ConfigError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    pub racks: Vec<RackConfig>,
    #[serde(default)]
    pub placements: HashMap<String, Vec<PlacementConfig>>,
    #[serde(default)]
    pub skus: Vec<SkuConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RackConfig {
    pub id: String,
    pub access_node: String,
    pub levels: u32,
    pub bins_per_level: u32,
    pub zone: Option<String>,
    /// Base time to access level 0 (seconds). Default: 0.0
    #[serde(default)]
    pub base_access_time_s: Option<f64>,
    /// Additional time per level above 0 (seconds). Default: 0.0
    #[serde(default)]
    pub per_level_time_s: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlacementConfig {
    pub rack: String,
    pub level: u32,
    pub bin: u32,
    pub qty: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkuConfig {
    pub id: String,
    #[serde(default = "default_pick_time")]
    pub unit_pick_time_s: f64,
    pub weight_kg: Option<f64>,
}

fn default_pick_time() -> f64 {
    3.0
}

impl StorageConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&content)?)
    }
}
