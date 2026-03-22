//! Policy factory - creates policy instances from configuration

use crate::world::PolicySet;
use waremax_config::{PolicyConfig, TrafficConfig};
use waremax_policies::{
    AdaptiveTrafficPolicy, BatchingPolicy, DueTimePolicy, FifoPolicy, LeastBusyPolicy,
    LeastQueuePolicy, NearestRobotPolicy, NearestStationPolicy, NoBatchingPolicy, PriorityPolicy,
    RerouteOnWaitPolicy, RoundRobinPolicy, StationAssignmentPolicy, StrictPriorityPolicy,
    TaskAllocationPolicy, TrafficPolicy, WaitAtNodePolicy, ZoneBatchingPolicy,
};

/// Create a PolicySet from scenario configuration
pub fn create_policies(config: &PolicyConfig) -> PolicySet {
    PolicySet {
        task_allocation: create_task_allocation(config),
        station_assignment: create_station_assignment(config),
        batching: create_batching(config),
        priority: create_priority(config),
        traffic: Box::new(WaitAtNodePolicy::new()), // Default traffic policy
    }
}

/// Create a PolicySet with traffic config (v1)
pub fn create_policies_with_traffic(
    config: &PolicyConfig,
    traffic_config: &TrafficConfig,
) -> PolicySet {
    PolicySet {
        task_allocation: create_task_allocation(config),
        station_assignment: create_station_assignment(config),
        batching: create_batching(config),
        priority: create_priority(config),
        traffic: create_traffic_policy(traffic_config),
    }
}

fn create_traffic_policy(config: &TrafficConfig) -> Box<dyn TrafficPolicy> {
    match config.policy.as_str() {
        "wait_at_node" => Box::new(WaitAtNodePolicy::new()),
        "reroute_on_wait" => Box::new(RerouteOnWaitPolicy::new(
            config.wait_threshold_s,
            config.max_reroute_attempts,
        )),
        "adaptive" => Box::new(AdaptiveTrafficPolicy::new(
            config.wait_threshold_s,
            config.node_capacity_default as usize,
        )),
        unknown => {
            eprintln!(
                "Warning: Unknown traffic policy '{}', using wait_at_node",
                unknown
            );
            Box::new(WaitAtNodePolicy::new())
        }
    }
}

fn create_task_allocation(config: &PolicyConfig) -> Box<dyn TaskAllocationPolicy> {
    match config.task_allocation.alloc_type.as_str() {
        "nearest_robot" => Box::new(NearestRobotPolicy::new()),
        "round_robin" => Box::new(RoundRobinPolicy::new()),
        "least_busy" => Box::new(LeastBusyPolicy::new()),
        unknown => {
            eprintln!(
                "Warning: Unknown task allocation policy '{}', using nearest_robot",
                unknown
            );
            Box::new(NearestRobotPolicy::new())
        }
    }
}

fn create_station_assignment(config: &PolicyConfig) -> Box<dyn StationAssignmentPolicy> {
    match config.station_assignment.assign_type.as_str() {
        "least_queue" => Box::new(LeastQueuePolicy::default()),
        "nearest_station" => Box::new(NearestStationPolicy::default()),
        unknown => {
            eprintln!(
                "Warning: Unknown station assignment policy '{}', using least_queue",
                unknown
            );
            Box::new(LeastQueuePolicy::default())
        }
    }
}

fn create_batching(config: &PolicyConfig) -> Box<dyn BatchingPolicy> {
    match config.batching.batch_type.as_str() {
        "none" => Box::new(NoBatchingPolicy::new()),
        "zone" => Box::new(ZoneBatchingPolicy::new(
            config.batching.max_items.unwrap_or(5),
            10.0, // Default zone radius
        )),
        unknown => {
            eprintln!("Warning: Unknown batching policy '{}', using none", unknown);
            Box::new(NoBatchingPolicy::new())
        }
    }
}

fn create_priority(config: &PolicyConfig) -> Box<dyn PriorityPolicy> {
    match config.priority.priority_type.as_str() {
        "strict_priority" => Box::new(StrictPriorityPolicy::new()),
        "fifo" => Box::new(FifoPolicy::new()),
        "due_time" => Box::new(DueTimePolicy::new()),
        unknown => {
            eprintln!(
                "Warning: Unknown priority policy '{}', using strict_priority",
                unknown
            );
            Box::new(StrictPriorityPolicy::new())
        }
    }
}
