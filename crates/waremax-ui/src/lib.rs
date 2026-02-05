//! Waremax UI - Embedded web UI for simulation visualization
//!
//! Provides a real-time interactive web interface for running and
//! visualizing warehouse robot simulations.

pub mod embed;
pub mod handlers;
pub mod server;
pub mod session;
pub mod simulation;
pub mod types;

pub use server::{run_server, ServerConfig};
pub use types::SessionConfig;
