//! Waremax Config - Configuration loading and validation

pub mod map_config;
pub mod scenario;
pub mod storage_config;
pub mod validation;

pub use map_config::MapConfig;
pub use scenario::*;
pub use storage_config::StorageConfig;
pub use validation::{
    validate_scenario, validate_scenario_only, FieldPath, ValidationError, ValidationErrorKind,
};
