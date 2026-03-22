//! Waremax Policies - Pluggable policies for task allocation, station assignment, etc.

pub mod allocation;
pub mod batching;
pub mod deadlock;
pub mod destination;
pub mod priority;
pub mod station;
pub mod traffic;
pub mod traits;

pub use traits::*;

// Task allocation policies
pub use allocation::{AuctionPolicy, WorkloadBalancedPolicy};
pub use allocation::{LeastBusyPolicy, NearestRobotPolicy, RoundRobinPolicy}; // v1

// Station assignment policies
pub use station::{DueTimePriorityStationPolicy, FastestServicePolicy};
pub use station::{LeastQueuePolicy, NearestStationPolicy}; // v1

// Batching policies
pub use batching::StationBatchPolicy;
pub use batching::{NoBatchingPolicy, ZoneBatchingPolicy}; // v1

// Priority policies
pub use priority::WeightedFairPolicy;
pub use priority::{DueTimePolicy, FifoPolicy, StrictPriorityPolicy}; // v1

// Destination policies (v1)
pub use destination::{DestinationContext, DestinationPolicy, NearestEmptyBinPolicy};

// Traffic policies (v1)
pub use traffic::{AdaptiveTrafficPolicy, RerouteOnWaitPolicy, WaitAtNodePolicy};
pub use traffic::{TrafficAction, TrafficPolicy, TrafficPolicyContext};

// Deadlock resolution policies (v2)
pub use deadlock::create_deadlock_resolver;
pub use deadlock::{DeadlockContext, DeadlockResolution, DeadlockResolver};
pub use deadlock::{
    LowestPriorityAborts, TieredResolver, WaitAndRetryResolver, YoungestRobotBacksUp,
};
