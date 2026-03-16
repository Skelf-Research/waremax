//! Waremax Policies - Pluggable policies for task allocation, station assignment, etc.

pub mod traits;
pub mod allocation;
pub mod station;
pub mod batching;
pub mod priority;
pub mod destination;

pub use traits::*;
pub use allocation::{NearestRobotPolicy, RoundRobinPolicy, LeastBusyPolicy};
pub use station::{LeastQueuePolicy, NearestStationPolicy};
pub use batching::{NoBatchingPolicy, ZoneBatchingPolicy};
pub use priority::{StrictPriorityPolicy, FifoPolicy, DueTimePolicy};
pub use destination::{DestinationPolicy, DestinationContext, NearestEmptyBinPolicy};
