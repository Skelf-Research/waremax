//! Map configuration parsing

use super::scenario::ConfigError;
use serde::{Deserialize, Serialize};

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
    /// Legacy field for backward compatibility (defaults to true)
    #[serde(default = "default_true")]
    pub bidirectional: bool,
    /// v2: Explicit direction ("one_way" | "bidirectional"), takes precedence over bidirectional
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default = "default_capacity")]
    pub capacity: u32,
    /// v2: Speed multiplier for routing cost (default: 1.0)
    /// Values < 1.0 = express/faster lane, > 1.0 = slower/restricted
    #[serde(default = "default_speed_multiplier")]
    pub speed_multiplier: Option<f64>,
}

impl EdgeConfig {
    /// Check if this edge should be treated as one-way
    ///
    /// Returns true for one-way edges, false for bidirectional.
    /// If `direction` field is set, it takes precedence.
    /// Otherwise, uses the legacy `bidirectional` field.
    pub fn is_one_way(&self) -> bool {
        if let Some(dir) = &self.direction {
            dir == "one_way"
        } else {
            !self.bidirectional
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_capacity() -> u32 {
    1
}

fn default_speed_multiplier() -> Option<f64> {
    None // None means use default 1.0
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
