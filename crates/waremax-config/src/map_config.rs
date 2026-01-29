//! Map configuration parsing

use serde::{Deserialize, Serialize};
use super::scenario::ConfigError;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapConfig {
    pub nodes: Vec<NodeConfig>,
    pub edges: Vec<EdgeConfig>,
    #[serde(default)]
    pub constraints: ConstraintsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeConfig {
    pub id: String,
    pub x: f64,
    pub y: f64,
    #[serde(rename = "type")]
    pub node_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EdgeConfig {
    pub from: String,
    pub to: String,
    pub length_m: f64,
    #[serde(default = "default_true")]
    pub bidirectional: bool,
    #[serde(default = "default_capacity")]
    pub capacity: u32,
}

fn default_true() -> bool {
    true
}

fn default_capacity() -> u32 {
    1
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ConstraintsConfig {
    #[serde(default)]
    pub blocked_nodes: Vec<String>,
    #[serde(default)]
    pub blocked_edges: Vec<BlockedEdge>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockedEdge {
    pub from: String,
    pub to: String,
}

impl MapConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }
}
