//! Waremax Policies - Pluggable policies for task allocation, station assignment, etc.

pub mod traits;
pub mod allocation;
pub mod station;
pub mod batching;
pub mod priority;
pub mod destination;
pub mod traffic;
pub mod deadlock;

pub use traits::*;

// Task allocation policies
pub use allocation::{NearestRobotPolicy, RoundRobinPolicy, LeastBusyPolicy};
pub use allocation::{AuctionPolicy, WorkloadBalancedPolicy}; // v1

// Station assignment policies
pub use station::{LeastQueuePolicy, NearestStationPolicy};
pub use station::{FastestServicePolicy, DueTimePriorityStationPolicy}; // v1

// Batching policies
pub use batching::{NoBatchingPolicy, ZoneBatchingPolicy};
pub use batching::StationBatchPolicy; // v1

// Priority policies
pub use priority::{StrictPriorityPolicy, FifoPolicy, DueTimePolicy};
pub use priority::WeightedFairPolicy; // v1

// Destination policies (v1)
pub use destination::{DestinationPolicy, DestinationContext, NearestEmptyBinPolicy};

// Traffic policies (v1)
pub use traffic::{TrafficPolicy, TrafficPolicyContext, TrafficAction};
pub use traffic::{WaitAtNodePolicy, RerouteOnWaitPolicy, AdaptiveTrafficPolicy};

// Deadlock resolution policies (v2)
pub use deadlock::{DeadlockResolver, DeadlockResolution, DeadlockContext};
pub use deadlock::{YoungestRobotBacksUp, LowestPriorityAborts, WaitAndRetryResolver, TieredResolver};
pub use deadlock::create_deadlock_resolver;
