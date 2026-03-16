//! Waremax Core - Core types and DES kernel for warehouse robot simulation
//!
//! This crate provides the fundamental types and discrete-event simulation kernel
//! used throughout the Waremax simulation system.

pub mod time;
pub mod id;
pub mod event;
pub mod kernel;
pub mod rng;
pub mod error;

pub use time::SimTime;
pub use id::*;
pub use event::{SimEvent, ScheduledEvent};
pub use kernel::Kernel;
pub use rng::SimRng;
pub use error::SimError;
