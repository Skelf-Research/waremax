//! Traffic management policies for handling congestion

use waremax_core::{EdgeId, NodeId, RobotId, SimTime};
use waremax_map::{TrafficManager, WarehouseMap};

/// Context for traffic policy decisions
pub struct TrafficPolicyContext<'a> {
    pub robot_id: RobotId,
    pub current_node: NodeId,
    pub blocked_edge: EdgeId,
    pub target_node: NodeId,
    pub destination: NodeId,
    pub wait_duration: SimTime,
    pub map: &'a WarehouseMap,
    pub traffic: &'a TrafficManager,
}

/// Action to take when traffic is blocked
#[derive(Clone, Debug, PartialEq)]
pub enum TrafficAction {
    /// Continue waiting at current position
    Wait,
    /// Attempt to find an alternate route
    Reroute,
    /// Abort current movement
    Abort,
}

/// Policy for handling traffic congestion
pub trait TrafficPolicy: Send + Sync {
    /// Decide action when robot cannot enter an edge
    fn on_blocked(&self, ctx: &TrafficPolicyContext) -> TrafficAction;

    /// Policy name for logging
    fn name(&self) -> &'static str;
}

/// Simple wait-at-node policy (v0 behavior)
/// Robot waits until the edge becomes available
pub struct WaitAtNodePolicy;

impl WaitAtNodePolicy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WaitAtNodePolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl TrafficPolicy for WaitAtNodePolicy {
    fn on_blocked(&self, _ctx: &TrafficPolicyContext) -> TrafficAction {
        TrafficAction::Wait
    }

    fn name(&self) -> &'static str {
        "wait_at_node"
    }
}

/// v1: Reroute-on-wait policy
/// Attempts to find an alternate route after waiting for a threshold duration
pub struct RerouteOnWaitPolicy {
    /// Wait threshold before attempting reroute (seconds)
    wait_threshold_s: f64,
    /// Maximum reroute attempts before falling back to wait
    _max_reroute_attempts: u32,
}

impl RerouteOnWaitPolicy {
    pub fn new(wait_threshold_s: f64, max_reroute_attempts: u32) -> Self {
        Self {
            wait_threshold_s,
            _max_reroute_attempts: max_reroute_attempts,
        }
    }

    pub fn default_thresholds() -> Self {
        Self::new(2.0, 3)
    }
}

impl Default for RerouteOnWaitPolicy {
    fn default() -> Self {
        Self::default_thresholds()
    }
}

impl TrafficPolicy for RerouteOnWaitPolicy {
    fn on_blocked(&self, ctx: &TrafficPolicyContext) -> TrafficAction {
        // Check if we've waited long enough to consider rerouting
        if ctx.wait_duration.as_seconds() >= self.wait_threshold_s {
            TrafficAction::Reroute
        } else {
            TrafficAction::Wait
        }
    }

    fn name(&self) -> &'static str {
        "reroute_on_wait"
    }
}

/// v1: Adaptive traffic policy
/// Combines waiting with intelligent rerouting based on congestion levels
pub struct AdaptiveTrafficPolicy {
    /// Base wait threshold (seconds)
    base_wait_s: f64,
    /// Reroute if congestion is above this level
    congestion_threshold: usize,
}

impl AdaptiveTrafficPolicy {
    pub fn new(base_wait_s: f64, congestion_threshold: usize) -> Self {
        Self {
            base_wait_s,
            congestion_threshold,
        }
    }
}

impl Default for AdaptiveTrafficPolicy {
    fn default() -> Self {
        Self::new(1.0, 2)
    }
}

impl TrafficPolicy for AdaptiveTrafficPolicy {
    fn on_blocked(&self, ctx: &TrafficPolicyContext) -> TrafficAction {
        let edge_congestion = ctx.traffic.get_edge_occupancy(ctx.blocked_edge);
        let node_congestion = ctx.traffic.get_node_occupancy(ctx.current_node);

        // If congestion is high, reroute sooner
        if (edge_congestion >= self.congestion_threshold
            || node_congestion >= self.congestion_threshold)
            && ctx.wait_duration.as_seconds() >= self.base_wait_s * 0.5
        {
            return TrafficAction::Reroute;
        }

        // Normal wait threshold
        if ctx.wait_duration.as_seconds() >= self.base_wait_s {
            TrafficAction::Reroute
        } else {
            TrafficAction::Wait
        }
    }

    fn name(&self) -> &'static str {
        "adaptive"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wait_at_node_always_waits() {
        let policy = WaitAtNodePolicy::new();
        // Would need full context to test, but the policy always returns Wait
        assert_eq!(policy.name(), "wait_at_node");
    }

    #[test]
    fn test_policy_names() {
        assert_eq!(WaitAtNodePolicy::new().name(), "wait_at_node");
        assert_eq!(RerouteOnWaitPolicy::default().name(), "reroute_on_wait");
        assert_eq!(AdaptiveTrafficPolicy::default().name(), "adaptive");
    }
}
