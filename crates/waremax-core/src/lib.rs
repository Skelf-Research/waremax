//! Waremax Core - Core types and DES kernel for warehouse robot simulation
//!
//! This crate provides the fundamental types and discrete-event simulation kernel
//! used throughout the Waremax simulation system.

pub mod error;
pub mod event;
pub mod id;
pub mod kernel;
pub mod rng;
pub mod time;

pub use error::SimError;
pub use event::{ScheduledEvent, SimEvent};
pub use id::*;
pub use kernel::Kernel;
pub use rng::SimRng;
pub use time::SimTime;
