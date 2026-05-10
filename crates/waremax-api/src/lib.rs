//! Waremax API - Axum-based API library for simulation control
//!
//! Provides a reusable Axum router builder for the Waremax simulation
//! API. Consumers can embed this router in their own server or use the
//! `waremax-api-server` binary crate.

pub mod handlers;
pub mod server;
pub mod session;
pub mod simulation;
pub mod state;
pub mod types;

pub use server::{create_router, ApiConfig};
pub use types::SessionConfig;
