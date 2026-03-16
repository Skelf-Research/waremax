//! Error types for the simulation

use thiserror::Error;

/// Simulation error types
#[derive(Error, Debug)]
pub enum SimError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Entity not found
    #[error("Entity not found: {entity_type} with id {id}")]
    NotFound { entity_type: &'static str, id: u32 },

    /// Invalid state transition
    #[error("Invalid state transition: {0}")]
    InvalidState(String),

    /// Capacity exceeded
    #[error("Capacity exceeded: {0}")]
    CapacityExceeded(String),

    /// Inventory error
    #[error("Inventory error: {0}")]
    Inventory(String),

    /// Routing error
    #[error("Routing error: no path from {from} to {to}")]
    NoPath { from: u32, to: u32 },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl SimError {
    /// Create a not found error for a robot
    pub fn robot_not_found(id: u32) -> Self {
        Self::NotFound {
            entity_type: "Robot",
            id,
        }
    }

    /// Create a not found error for a node
    pub fn node_not_found(id: u32) -> Self {
        Self::NotFound {
            entity_type: "Node",
            id,
        }
    }

    /// Create a not found error for an edge
    pub fn edge_not_found(id: u32) -> Self {
        Self::NotFound {
            entity_type: "Edge",
            id,
        }
    }

    /// Create a not found error for a station
    pub fn station_not_found(id: u32) -> Self {
        Self::NotFound {
            entity_type: "Station",
            id,
        }
    }

    /// Create a not found error for a task
    pub fn task_not_found(id: u32) -> Self {
        Self::NotFound {
            entity_type: "Task",
            id,
        }
    }

    /// Create a not found error for an order
    pub fn order_not_found(id: u32) -> Self {
        Self::NotFound {
            entity_type: "Order",
            id,
        }
    }

    /// Create a not found error for a SKU
    pub fn sku_not_found(id: u32) -> Self {
        Self::NotFound {
            entity_type: "SKU",
            id,
        }
    }
}

/// Result type alias for simulation operations
pub type SimResult<T> = Result<T, SimError>;
