//! Edge traffic control policies for managing robot entry and traversal.
//!
//! These policies decide whether a robot may enter an edge/node and
//! handle lifecycle callbacks (enter, leave, position update).
//! They are orthogonal to the congestion policies in `traffic.rs`.

use waremax_core::{EdgeId, NodeId, RobotId, SimTime};
use waremax_map::TrafficManager;

/// Policy for controlling edge/node entry and traversal.
///
/// Implementations can range from simple capacity checks to continuous
/// sub-edge position tracking with collision avoidance.
pub trait EdgeTrafficPolicy: Send + Sync {
    /// Human-readable policy name.
    fn name(&self) -> &str;

    /// Can `robot` enter `edge` traveling from `from` to `to`?
    fn can_enter_edge(
        &self,
        traffic: &TrafficManager,
        edge: EdgeId,
        robot: RobotId,
        from: NodeId,
        to: NodeId,
    ) -> bool;

    /// Can `robot` enter `node`?
    fn can_enter_node(
        &self,
        traffic: &TrafficManager,
        node: NodeId,
        robot: RobotId,
    ) -> bool;

    /// Called when `robot` successfully enters `edge` from `from` to `to`.
    fn on_enter_edge(
        &mut self,
        traffic: &mut TrafficManager,
        edge: EdgeId,
        robot: RobotId,
        from: NodeId,
        to: NodeId,
    );

    /// Called when `robot` leaves `edge`.
    fn on_leave_edge(
        &mut self,
        traffic: &mut TrafficManager,
        edge: EdgeId,
        robot: RobotId,
    );

    /// Called when `robot` enters `node`.
    fn on_enter_node(
        &mut self,
        traffic: &mut TrafficManager,
        node: NodeId,
        robot: RobotId,
    );

    /// Called when `robot` leaves `node`.
    fn on_leave_node(
        &mut self,
        traffic: &mut TrafficManager,
        node: NodeId,
        robot: RobotId,
    );

    /// Called during continuous tracking with a position update.
    ///
    /// Default no-op for policies that don't support continuous tracking.
    fn on_position_update(
        &mut self,
        _traffic: &mut TrafficManager,
        _edge: EdgeId,
        _robot: RobotId,
        _progress: f64,
    ) {
    }

    /// Periodic tick (e.g. for cleanup or revalidation).
    ///
    /// Default no-op.
    fn tick(
        &mut self,
        _traffic: &mut TrafficManager,
        _current_time: SimTime,
    ) {
    }
}

/// Capacity-based coarse policy that matches the original v0 behavior.
///
/// Robots occupy entire edges/nodes. No direction locking, no continuous
/// position tracking. Multiple robots may share an edge up to capacity.
pub struct CoarseTrafficPolicy;

impl CoarseTrafficPolicy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CoarseTrafficPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgeTrafficPolicy for CoarseTrafficPolicy {
    fn name(&self) -> &str {
        "coarse"
    }

    fn can_enter_edge(
        &self,
        traffic: &TrafficManager,
        edge: EdgeId,
        robot: RobotId,
        _from: NodeId,
        _to: NodeId,
    ) -> bool {
        traffic.can_enter_edge(edge, robot)
    }

    fn can_enter_node(
        &self,
        traffic: &TrafficManager,
        node: NodeId,
        robot: RobotId,
    ) -> bool {
        traffic.can_enter_node(node, robot)
    }

    fn on_enter_edge(
        &mut self,
        traffic: &mut TrafficManager,
        edge: EdgeId,
        robot: RobotId,
        _from: NodeId,
        _to: NodeId,
    ) {
        traffic.enter_edge(edge, robot);
    }

    fn on_leave_edge(
        &mut self,
        traffic: &mut TrafficManager,
        edge: EdgeId,
        robot: RobotId,
    ) {
        traffic.leave_edge(edge, robot);
    }

    fn on_enter_node(
        &mut self,
        traffic: &mut TrafficManager,
        node: NodeId,
        robot: RobotId,
    ) {
        traffic.enter_node(node, robot);
    }

    fn on_leave_node(
        &mut self,
        traffic: &mut TrafficManager,
        node: NodeId,
        robot: RobotId,
    ) {
        traffic.leave_node(node, robot);
    }
}

/// Continuous position tracking policy with direction locking and
/// following-distance enforcement.
///
/// Robots track progress along edges (0.0 to 1.0). An edge may only
/// be traversed in one direction at a time. Robots must maintain a
/// minimum following distance from the robot ahead of them.
pub struct ContinuousTrafficPolicy {
    /// Minimum distance (meters) between robots on the same edge.
    pub safety_distance_m: f64,
    /// How often (simulation seconds) to emit position update events.
    pub position_update_interval_s: f64,
}

impl ContinuousTrafficPolicy {
    pub fn new(safety_distance_m: f64, position_update_interval_s: f64) -> Self {
        Self {
            safety_distance_m,
            position_update_interval_s,
        }
    }

    pub fn default_config() -> Self {
        Self::new(1.0, 0.1)
    }
}

impl Default for ContinuousTrafficPolicy {
    fn default() -> Self {
        Self::default_config()
    }
}

impl EdgeTrafficPolicy for ContinuousTrafficPolicy {
    fn name(&self) -> &str {
        "continuous"
    }

    fn can_enter_edge(
        &self,
        traffic: &TrafficManager,
        edge: EdgeId,
        _robot: RobotId,
        from: NodeId,
        to: NodeId,
    ) -> bool {
        // 1. Capacity check
        let capacity = traffic.edge_capacity(edge);
        if traffic.edge_occupancy_count(edge) >= capacity as usize {
            return false;
        }

        // 2. Direction check (head-on collision prevention)
        if let Some(Some((existing_from, existing_to))) = traffic.edge_direction(edge) {
            if (existing_from, existing_to) != (from, to) {
                return false; // Opposite direction in use
            }
        }

        // 3. Following distance check
        if let Some(states) = traffic.continuous_states(edge) {
            let edge_length = traffic.edge_length(edge);
            if edge_length > 0.0 {
                let min_gap = self.safety_distance_m / edge_length;
                for state in states {
                    if state.from == from && state.to == to {
                        if state.progress < min_gap {
                            return false; // Too close to robot ahead
                        }
                    }
                }
            }
        }

        true
    }

    fn can_enter_node(
        &self,
        traffic: &TrafficManager,
        node: NodeId,
        robot: RobotId,
    ) -> bool {
        traffic.can_enter_node(node, robot)
    }

    fn on_enter_edge(
        &mut self,
        traffic: &mut TrafficManager,
        edge: EdgeId,
        robot: RobotId,
        from: NodeId,
        to: NodeId,
    ) {
        // Record coarse occupancy for deadlock detection compatibility
        traffic.enter_edge(edge, robot);

        // Set direction if this is the first robot on the edge
        if traffic.edge_direction(edge).is_none() {
            traffic.set_edge_direction(edge, Some((from, to)));
        }
    }

    fn on_leave_edge(
        &mut self,
        traffic: &mut TrafficManager,
        edge: EdgeId,
        robot: RobotId,
    ) {
        traffic.leave_edge(edge, robot);
        traffic.remove_continuous_state(edge, robot);
    }

    fn on_enter_node(
        &mut self,
        traffic: &mut TrafficManager,
        node: NodeId,
        robot: RobotId,
    ) {
        traffic.enter_node(node, robot);
    }

    fn on_leave_node(
        &mut self,
        traffic: &mut TrafficManager,
        node: NodeId,
        robot: RobotId,
    ) {
        traffic.leave_node(node, robot);
    }

    fn on_position_update(
        &mut self,
        traffic: &mut TrafficManager,
        edge: EdgeId,
        robot: RobotId,
        progress: f64,
    ) {
        traffic.update_continuous_progress(edge, robot, progress);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coarse_policy_allows_entry() {
        let mut traffic = TrafficManager::new(1, 1);
        let mut policy = CoarseTrafficPolicy::new();

        assert!(policy.can_enter_edge(&traffic, EdgeId(1), RobotId(1), NodeId(0), NodeId(1)));

        policy.on_enter_edge(&mut traffic, EdgeId(1), RobotId(1), NodeId(0), NodeId(1));

        // At capacity (default 1), second robot blocked
        assert!(!policy.can_enter_edge(&traffic, EdgeId(1), RobotId(2), NodeId(0), NodeId(1)));
    }

    #[test]
    fn test_continuous_direction_locking() {
        let mut traffic = TrafficManager::new(2, 2);
        let mut policy = ContinuousTrafficPolicy::default_config();

        // Robot 1 enters from 0 -> 1
        assert!(policy.can_enter_edge(&traffic, EdgeId(1), RobotId(1), NodeId(0), NodeId(1)));
        policy.on_enter_edge(&mut traffic, EdgeId(1), RobotId(1), NodeId(0), NodeId(1));

        // Robot 2 trying opposite direction is blocked
        assert!(!policy.can_enter_edge(&traffic, EdgeId(1), RobotId(2), NodeId(1), NodeId(0)));

        // Robot 3 same direction is allowed (capacity 2)
        assert!(policy.can_enter_edge(&traffic, EdgeId(1), RobotId(3), NodeId(0), NodeId(1)));
    }

    #[test]
    fn test_continuous_following_distance() {
        let mut traffic = TrafficManager::new(2, 2);
        traffic.register_edge_length(EdgeId(1), 10.0);

        let mut policy = ContinuousTrafficPolicy::new(3.0, 0.1);

        // Robot 1 enters
        policy.on_enter_edge(&mut traffic, EdgeId(1), RobotId(1), NodeId(0), NodeId(1));
        // Add a continuous state with 0.5 progress (5m along 10m edge)
        use waremax_core::SimTime;
        use waremax_map::ContinuousEdgeState;
        traffic.add_continuous_state(
            EdgeId(1),
            ContinuousEdgeState::new(
                RobotId(1),
                NodeId(0),
                NodeId(1),
                SimTime::from_seconds(0.0),
                SimTime::from_seconds(10.0),
            ),
        );
        traffic.update_continuous_progress(EdgeId(1), RobotId(1), 0.5);

        // Robot 2 trying to enter: min_gap = 3.0/10.0 = 0.3
        // Robot 1 is at 0.5, which is > 0.3, so allowed
        assert!(policy.can_enter_edge(&traffic, EdgeId(1), RobotId(2), NodeId(0), NodeId(1)));

        // If robot 1 were at 0.1 (1m along), gap would be 0.1 < 0.3, blocked
        traffic.update_continuous_progress(EdgeId(1), RobotId(1), 0.1);
        assert!(!policy.can_enter_edge(&traffic, EdgeId(1), RobotId(2), NodeId(0), NodeId(1)));
    }
}
