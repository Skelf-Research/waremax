//! Deadlock resolution policies
//!
//! Provides strategies for resolving deadlocks when detected in the wait-for graph.

use waremax_core::{RobotId, NodeId, SimTime};

/// Result of a deadlock resolution decision
#[derive(Clone, Debug)]
pub enum DeadlockResolution {
    /// Force one robot to back up to a specific node
    BackUp {
        /// Robot that should back up
        robot: RobotId,
        /// Node to back up to
        to_node: NodeId,
    },
    /// Abort the lowest priority robot's current task and requeue it
    AbortTask {
        /// Robot whose task should be aborted
        robot: RobotId,
    },
    /// Wait and retry (deadlock may resolve naturally, e.g., a robot completes its task)
    WaitAndRetry {
        /// How long to wait before checking again
        duration: SimTime,
    },
}

/// Context provided to the deadlock resolver
#[derive(Clone, Debug)]
pub struct DeadlockContext {
    /// Robot IDs involved in the deadlock cycle
    pub cycle: Vec<RobotId>,
    /// Current positions of robots in the cycle
    pub positions: Vec<(RobotId, NodeId)>,
    /// Previous nodes of robots (for backing up)
    pub previous_nodes: Vec<(RobotId, Option<NodeId>)>,
    /// Task priorities of robots (lower = higher priority)
    pub priorities: Vec<(RobotId, u32)>,
}

impl DeadlockContext {
    /// Create a new deadlock context
    pub fn new(cycle: Vec<RobotId>) -> Self {
        Self {
            cycle,
            positions: Vec::new(),
            previous_nodes: Vec::new(),
            priorities: Vec::new(),
        }
    }

    /// Add position information for a robot
    pub fn with_position(mut self, robot: RobotId, node: NodeId) -> Self {
        self.positions.push((robot, node));
        self
    }

    /// Add previous node information for a robot
    pub fn with_previous(mut self, robot: RobotId, prev: Option<NodeId>) -> Self {
        self.previous_nodes.push((robot, prev));
        self
    }

    /// Add priority information for a robot
    pub fn with_priority(mut self, robot: RobotId, priority: u32) -> Self {
        self.priorities.push((robot, priority));
        self
    }

    /// Get the robot with the highest ID (youngest)
    pub fn youngest_robot(&self) -> Option<RobotId> {
        self.cycle.iter().max_by_key(|r| r.as_u32()).copied()
    }

    /// Get the robot with the lowest priority (highest priority number)
    pub fn lowest_priority_robot(&self) -> Option<RobotId> {
        self.priorities
            .iter()
            .max_by_key(|(_, p)| p)
            .map(|(r, _)| *r)
    }

    /// Get the previous node for a specific robot
    pub fn get_previous_node(&self, robot: RobotId) -> Option<NodeId> {
        self.previous_nodes
            .iter()
            .find(|(r, _)| *r == robot)
            .and_then(|(_, node)| *node)
    }
}

/// Trait for deadlock resolution strategies
pub trait DeadlockResolver: Send + Sync {
    /// Resolve a deadlock given the context
    fn resolve(&self, ctx: &DeadlockContext) -> DeadlockResolution;

    /// Get the name of this resolver for logging/debugging
    fn name(&self) -> &'static str;
}

/// Resolver that makes the youngest robot (highest ID) back up
///
/// This is a simple deterministic strategy that ensures consistent
/// behavior across runs.
#[derive(Clone, Debug, Default)]
pub struct YoungestRobotBacksUp;

impl YoungestRobotBacksUp {
    pub fn new() -> Self {
        Self
    }
}

impl DeadlockResolver for YoungestRobotBacksUp {
    fn resolve(&self, ctx: &DeadlockContext) -> DeadlockResolution {
        let robot = ctx.youngest_robot().expect("deadlock cycle cannot be empty");

        // Try to find a previous node to back up to
        if let Some(prev_node) = ctx.get_previous_node(robot) {
            DeadlockResolution::BackUp {
                robot,
                to_node: prev_node,
            }
        } else {
            // If no previous node available, abort the task instead
            DeadlockResolution::AbortTask { robot }
        }
    }

    fn name(&self) -> &'static str {
        "youngest_backs_up"
    }
}

/// Resolver that aborts the task of the lowest-priority robot
///
/// This strategy preserves high-priority work at the cost of
/// potentially requeuing lower-priority tasks.
#[derive(Clone, Debug, Default)]
pub struct LowestPriorityAborts;

impl LowestPriorityAborts {
    pub fn new() -> Self {
        Self
    }
}

impl DeadlockResolver for LowestPriorityAborts {
    fn resolve(&self, ctx: &DeadlockContext) -> DeadlockResolution {
        // Use lowest priority robot if priority info available
        let robot = ctx
            .lowest_priority_robot()
            // Fall back to youngest robot if no priority info
            .or_else(|| ctx.youngest_robot())
            .expect("deadlock cycle cannot be empty");

        DeadlockResolution::AbortTask { robot }
    }

    fn name(&self) -> &'static str {
        "lowest_priority_aborts"
    }
}

/// Resolver that waits for a short duration before checking again
///
/// This strategy is useful when deadlocks may resolve naturally
/// (e.g., a robot is about to complete its current task).
#[derive(Clone, Debug)]
pub struct WaitAndRetryResolver {
    /// How long to wait before retrying
    pub wait_duration: SimTime,
}

impl WaitAndRetryResolver {
    pub fn new(wait_seconds: f64) -> Self {
        Self {
            wait_duration: SimTime::from_seconds(wait_seconds),
        }
    }
}

impl Default for WaitAndRetryResolver {
    fn default() -> Self {
        Self::new(1.0) // Default 1 second wait
    }
}

impl DeadlockResolver for WaitAndRetryResolver {
    fn resolve(&self, _ctx: &DeadlockContext) -> DeadlockResolution {
        DeadlockResolution::WaitAndRetry {
            duration: self.wait_duration,
        }
    }

    fn name(&self) -> &'static str {
        "wait_and_retry"
    }
}

/// Resolver that uses a tiered approach:
/// 1. First, try backing up the youngest robot
/// 2. If not possible, abort the lowest priority task
#[derive(Clone, Debug, Default)]
pub struct TieredResolver;

impl TieredResolver {
    pub fn new() -> Self {
        Self
    }
}

impl DeadlockResolver for TieredResolver {
    fn resolve(&self, ctx: &DeadlockContext) -> DeadlockResolution {
        // First, try to back up the youngest robot
        let youngest = ctx.youngest_robot().expect("deadlock cycle cannot be empty");

        if let Some(prev_node) = ctx.get_previous_node(youngest) {
            return DeadlockResolution::BackUp {
                robot: youngest,
                to_node: prev_node,
            };
        }

        // Try any robot in the cycle that can back up
        for &robot in &ctx.cycle {
            if let Some(prev_node) = ctx.get_previous_node(robot) {
                return DeadlockResolution::BackUp {
                    robot,
                    to_node: prev_node,
                };
            }
        }

        // No robot can back up, abort lowest priority task
        let victim = ctx
            .lowest_priority_robot()
            .or_else(|| ctx.youngest_robot())
            .expect("deadlock cycle cannot be empty");

        DeadlockResolution::AbortTask { robot: victim }
    }

    fn name(&self) -> &'static str {
        "tiered"
    }
}

/// Create a deadlock resolver from a configuration string
pub fn create_deadlock_resolver(name: &str) -> Box<dyn DeadlockResolver> {
    match name {
        "youngest_backs_up" => Box::new(YoungestRobotBacksUp::new()),
        "lowest_priority_aborts" => Box::new(LowestPriorityAborts::new()),
        "wait_and_retry" => Box::new(WaitAndRetryResolver::default()),
        "tiered" => Box::new(TieredResolver::new()),
        _ => Box::new(YoungestRobotBacksUp::new()), // Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_youngest_robot_backs_up() {
        let resolver = YoungestRobotBacksUp::new();

        let ctx = DeadlockContext::new(vec![RobotId(1), RobotId(5), RobotId(3)])
            .with_previous(RobotId(5), Some(NodeId(100)));

        let resolution = resolver.resolve(&ctx);
        match resolution {
            DeadlockResolution::BackUp { robot, to_node } => {
                assert_eq!(robot, RobotId(5)); // Highest ID
                assert_eq!(to_node, NodeId(100));
            }
            _ => panic!("Expected BackUp resolution"),
        }
    }

    #[test]
    fn test_youngest_falls_back_to_abort() {
        let resolver = YoungestRobotBacksUp::new();

        // No previous nodes available
        let ctx = DeadlockContext::new(vec![RobotId(1), RobotId(5), RobotId(3)]);

        let resolution = resolver.resolve(&ctx);
        match resolution {
            DeadlockResolution::AbortTask { robot } => {
                assert_eq!(robot, RobotId(5)); // Highest ID
            }
            _ => panic!("Expected AbortTask resolution"),
        }
    }

    #[test]
    fn test_lowest_priority_aborts() {
        let resolver = LowestPriorityAborts::new();

        let ctx = DeadlockContext::new(vec![RobotId(1), RobotId(2), RobotId(3)])
            .with_priority(RobotId(1), 1)  // High priority
            .with_priority(RobotId(2), 10) // Low priority
            .with_priority(RobotId(3), 5); // Medium priority

        let resolution = resolver.resolve(&ctx);
        match resolution {
            DeadlockResolution::AbortTask { robot } => {
                assert_eq!(robot, RobotId(2)); // Lowest priority (highest number)
            }
            _ => panic!("Expected AbortTask resolution"),
        }
    }

    #[test]
    fn test_wait_and_retry() {
        let resolver = WaitAndRetryResolver::new(2.5);

        let ctx = DeadlockContext::new(vec![RobotId(1), RobotId(2)]);

        let resolution = resolver.resolve(&ctx);
        match resolution {
            DeadlockResolution::WaitAndRetry { duration } => {
                assert_eq!(duration.as_seconds(), 2.5);
            }
            _ => panic!("Expected WaitAndRetry resolution"),
        }
    }

    #[test]
    fn test_tiered_resolver() {
        let resolver = TieredResolver::new();

        // Case 1: Can back up
        let ctx1 = DeadlockContext::new(vec![RobotId(1), RobotId(3)])
            .with_previous(RobotId(3), Some(NodeId(50)));

        match resolver.resolve(&ctx1) {
            DeadlockResolution::BackUp { robot, .. } => {
                assert_eq!(robot, RobotId(3));
            }
            _ => panic!("Expected BackUp resolution"),
        }

        // Case 2: Cannot back up, falls back to abort
        let ctx2 = DeadlockContext::new(vec![RobotId(1), RobotId(3)])
            .with_priority(RobotId(1), 5)
            .with_priority(RobotId(3), 1);

        match resolver.resolve(&ctx2) {
            DeadlockResolution::AbortTask { robot } => {
                assert_eq!(robot, RobotId(1)); // Lower priority
            }
            _ => panic!("Expected AbortTask resolution"),
        }
    }

    #[test]
    fn test_create_resolver() {
        assert_eq!(create_deadlock_resolver("youngest_backs_up").name(), "youngest_backs_up");
        assert_eq!(create_deadlock_resolver("lowest_priority_aborts").name(), "lowest_priority_aborts");
        assert_eq!(create_deadlock_resolver("wait_and_retry").name(), "wait_and_retry");
        assert_eq!(create_deadlock_resolver("tiered").name(), "tiered");
        assert_eq!(create_deadlock_resolver("unknown").name(), "youngest_backs_up"); // Default
    }
}
