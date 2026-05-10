//! Continuous position tracking for robots on edges
//!
//! Provides types for tracking robot positions along edges when using
//! the continuous traffic policy.

use waremax_core::{EdgeId, NodeId, RobotId, SimTime};

/// Physical position of a robot in the warehouse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RobotPosition {
    /// Robot is stationary at a node.
    AtNode {
        /// The node the robot is at.
        node: NodeId,
    },
    /// Robot is traversing an edge.
    OnEdge {
        /// The edge being traversed.
        edge: EdgeId,
        /// Source node.
        from: NodeId,
        /// Destination node.
        to: NodeId,
        /// Progress from 0.0 (at `from`) to 1.0 (at `to`).
        progress: f64,
    },
}

/// Continuous position tracking state for a single robot on an edge.
#[derive(Clone, Debug)]
pub struct ContinuousEdgeState {
    /// The robot being tracked.
    pub robot_id: RobotId,
    /// Source node.
    pub from: NodeId,
    /// Destination node.
    pub to: NodeId,
    /// Current progress along the edge (0.0 to 1.0).
    pub progress: f64,
    /// Simulation time when the robot entered the edge.
    pub entry_time: SimTime,
    /// Expected simulation time when the robot will exit the edge.
    pub expected_exit_time: SimTime,
}

impl ContinuousEdgeState {
    /// Create a new continuous edge state at progress 0.0.
    pub fn new(
        robot_id: RobotId,
        from: NodeId,
        to: NodeId,
        entry_time: SimTime,
        expected_exit_time: SimTime,
    ) -> Self {
        Self {
            robot_id,
            from,
            to,
            progress: 0.0,
            entry_time,
            expected_exit_time,
        }
    }

    /// Update progress based on elapsed simulation time.
    pub fn update_progress(&mut self, current_time: SimTime, edge_length_m: f64, speed_mps: f64) {
        let elapsed = (current_time - self.entry_time).as_seconds();
        let total_travel_time = edge_length_m / speed_mps;
        if total_travel_time > 0.0 {
            self.progress = (elapsed / total_travel_time).clamp(0.0, 1.0);
        } else {
            self.progress = 1.0;
        }
    }
}
