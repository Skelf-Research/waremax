//! Deadlock detection using Wait-For Graph
//!
//! Tracks which robots are waiting for which resources and detects
//! circular wait conditions that would cause deadlocks.

use std::collections::{HashMap, HashSet};
use waremax_core::{RobotId, NodeId, EdgeId};

/// Represents what a robot is waiting for
#[derive(Clone, Debug)]
pub enum WaitingFor {
    /// Waiting for an edge, blocked by specific robots
    Edge {
        edge_id: EdgeId,
        blocked_by: Vec<RobotId>,
    },
    /// Waiting for a node, blocked by specific robots
    Node {
        node_id: NodeId,
        blocked_by: Vec<RobotId>,
    },
}

impl WaitingFor {
    /// Get the robots that are blocking this wait
    pub fn blockers(&self) -> &[RobotId] {
        match self {
            WaitingFor::Edge { blocked_by, .. } => blocked_by,
            WaitingFor::Node { blocked_by, .. } => blocked_by,
        }
    }
}

/// Wait-For Graph for deadlock detection
///
/// Maintains a graph where:
/// - Nodes represent robots
/// - Edges represent "waits-for" relationships
///
/// A cycle in this graph indicates a deadlock.
#[derive(Clone, Debug, Default)]
pub struct WaitForGraph {
    /// Maps robot_id -> what it's waiting for (including which robots block it)
    waiting_for: HashMap<RobotId, WaitingFor>,
}

impl WaitForGraph {
    /// Create a new empty wait-for graph
    pub fn new() -> Self {
        Self {
            waiting_for: HashMap::new(),
        }
    }

    /// Record that a robot is waiting for a resource
    pub fn add_wait(&mut self, robot: RobotId, waiting_for: WaitingFor) {
        self.waiting_for.insert(robot, waiting_for);
    }

    /// Remove a robot's wait (e.g., when it acquires the resource)
    pub fn remove_wait(&mut self, robot: RobotId) {
        self.waiting_for.remove(&robot);
    }

    /// Check if a robot is currently waiting
    pub fn is_waiting(&self, robot: RobotId) -> bool {
        self.waiting_for.contains_key(&robot)
    }

    /// Get what a robot is waiting for, if anything
    pub fn get_wait(&self, robot: RobotId) -> Option<&WaitingFor> {
        self.waiting_for.get(&robot)
    }

    /// Get the number of robots currently waiting
    pub fn waiting_count(&self) -> usize {
        self.waiting_for.len()
    }

    /// Detect if there's a cycle (deadlock) in the wait-for graph
    ///
    /// Returns Some(cycle) with the robot IDs forming the cycle,
    /// or None if no deadlock exists.
    ///
    /// Uses DFS-based cycle detection.
    pub fn detect_cycle(&self) -> Option<Vec<RobotId>> {
        // For each waiting robot, try to find a cycle starting from it
        for &start_robot in self.waiting_for.keys() {
            if let Some(cycle) = self.find_cycle_from(start_robot) {
                return Some(cycle);
            }
        }
        None
    }

    /// Try to find a cycle starting from a specific robot using DFS
    fn find_cycle_from(&self, start: RobotId) -> Option<Vec<RobotId>> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        let mut path_set = HashSet::new();

        self.dfs_find_cycle(start, &mut visited, &mut path, &mut path_set, start)
    }

    /// DFS helper for cycle detection
    fn dfs_find_cycle(
        &self,
        current: RobotId,
        visited: &mut HashSet<RobotId>,
        path: &mut Vec<RobotId>,
        path_set: &mut HashSet<RobotId>,
        start: RobotId,
    ) -> Option<Vec<RobotId>> {
        // If we've already fully explored this robot, no cycle through it
        if visited.contains(&current) {
            return None;
        }

        // If current is in the current path, we found a cycle
        if path_set.contains(&current) {
            // Extract the cycle from the path
            let cycle_start_idx = path.iter().position(|&r| r == current)?;
            return Some(path[cycle_start_idx..].to_vec());
        }

        // Add current to path
        path.push(current);
        path_set.insert(current);

        // Get what this robot is waiting for
        if let Some(waiting) = self.waiting_for.get(&current) {
            // For each robot that's blocking us, continue DFS
            for &blocker in waiting.blockers() {
                // Check if blocker leads back to start (simple cycle check)
                if blocker == start && path.len() > 1 {
                    path.push(start);
                    return Some(path.clone());
                }

                // Continue DFS to see if blocker is also waiting
                if self.waiting_for.contains_key(&blocker) {
                    if let Some(cycle) = self.dfs_find_cycle(blocker, visited, path, path_set, start) {
                        return Some(cycle);
                    }
                }
            }
        }

        // Backtrack
        path.pop();
        path_set.remove(&current);
        visited.insert(current);

        None
    }

    /// Detect all cycles in the graph (for debugging/analysis)
    pub fn detect_all_cycles(&self) -> Vec<Vec<RobotId>> {
        let mut cycles = Vec::new();
        let mut found_robots = HashSet::new();

        for &start_robot in self.waiting_for.keys() {
            if found_robots.contains(&start_robot) {
                continue;
            }

            if let Some(cycle) = self.find_cycle_from(start_robot) {
                for &robot in &cycle {
                    found_robots.insert(robot);
                }
                cycles.push(cycle);
            }
        }

        cycles
    }

    /// Clear all waits (e.g., at end of simulation)
    pub fn clear(&mut self) {
        self.waiting_for.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph_no_cycle() {
        let graph = WaitForGraph::new();
        assert!(graph.detect_cycle().is_none());
    }

    #[test]
    fn test_single_wait_no_cycle() {
        let mut graph = WaitForGraph::new();
        graph.add_wait(
            RobotId(1),
            WaitingFor::Edge {
                edge_id: EdgeId(100),
                blocked_by: vec![RobotId(2)],
            },
        );
        // Robot 2 is not waiting, so no cycle
        assert!(graph.detect_cycle().is_none());
    }

    #[test]
    fn test_two_robot_cycle() {
        let mut graph = WaitForGraph::new();
        // Robot 1 waits for Robot 2
        graph.add_wait(
            RobotId(1),
            WaitingFor::Edge {
                edge_id: EdgeId(100),
                blocked_by: vec![RobotId(2)],
            },
        );
        // Robot 2 waits for Robot 1
        graph.add_wait(
            RobotId(2),
            WaitingFor::Node {
                node_id: NodeId(50),
                blocked_by: vec![RobotId(1)],
            },
        );

        let cycle = graph.detect_cycle();
        assert!(cycle.is_some());
        let cycle = cycle.unwrap();
        assert!(cycle.contains(&RobotId(1)));
        assert!(cycle.contains(&RobotId(2)));
    }

    #[test]
    fn test_three_robot_cycle() {
        let mut graph = WaitForGraph::new();
        // Robot 1 -> Robot 2 -> Robot 3 -> Robot 1
        graph.add_wait(
            RobotId(1),
            WaitingFor::Edge {
                edge_id: EdgeId(100),
                blocked_by: vec![RobotId(2)],
            },
        );
        graph.add_wait(
            RobotId(2),
            WaitingFor::Edge {
                edge_id: EdgeId(101),
                blocked_by: vec![RobotId(3)],
            },
        );
        graph.add_wait(
            RobotId(3),
            WaitingFor::Edge {
                edge_id: EdgeId(102),
                blocked_by: vec![RobotId(1)],
            },
        );

        let cycle = graph.detect_cycle();
        assert!(cycle.is_some());
        let cycle = cycle.unwrap();
        assert_eq!(cycle.len(), 4); // 1 -> 2 -> 3 -> 1
    }

    #[test]
    fn test_chain_no_cycle() {
        let mut graph = WaitForGraph::new();
        // Robot 1 -> Robot 2 -> Robot 3 (Robot 3 not waiting)
        graph.add_wait(
            RobotId(1),
            WaitingFor::Edge {
                edge_id: EdgeId(100),
                blocked_by: vec![RobotId(2)],
            },
        );
        graph.add_wait(
            RobotId(2),
            WaitingFor::Edge {
                edge_id: EdgeId(101),
                blocked_by: vec![RobotId(3)],
            },
        );
        // Robot 3 is not waiting for anything

        assert!(graph.detect_cycle().is_none());
    }

    #[test]
    fn test_remove_wait_breaks_cycle() {
        let mut graph = WaitForGraph::new();
        graph.add_wait(
            RobotId(1),
            WaitingFor::Edge {
                edge_id: EdgeId(100),
                blocked_by: vec![RobotId(2)],
            },
        );
        graph.add_wait(
            RobotId(2),
            WaitingFor::Node {
                node_id: NodeId(50),
                blocked_by: vec![RobotId(1)],
            },
        );

        // Cycle exists
        assert!(graph.detect_cycle().is_some());

        // Remove one wait
        graph.remove_wait(RobotId(1));

        // Cycle broken
        assert!(graph.detect_cycle().is_none());
    }

    #[test]
    fn test_multiple_blockers() {
        let mut graph = WaitForGraph::new();
        // Robot 1 is blocked by both Robot 2 and Robot 3
        graph.add_wait(
            RobotId(1),
            WaitingFor::Edge {
                edge_id: EdgeId(100),
                blocked_by: vec![RobotId(2), RobotId(3)],
            },
        );
        // Robot 2 waits for Robot 1 (creates cycle)
        graph.add_wait(
            RobotId(2),
            WaitingFor::Node {
                node_id: NodeId(50),
                blocked_by: vec![RobotId(1)],
            },
        );
        // Robot 3 is not waiting

        let cycle = graph.detect_cycle();
        assert!(cycle.is_some());
    }

    #[test]
    fn test_is_waiting() {
        let mut graph = WaitForGraph::new();
        assert!(!graph.is_waiting(RobotId(1)));

        graph.add_wait(
            RobotId(1),
            WaitingFor::Edge {
                edge_id: EdgeId(100),
                blocked_by: vec![RobotId(2)],
            },
        );

        assert!(graph.is_waiting(RobotId(1)));
        assert!(!graph.is_waiting(RobotId(2)));
    }

    #[test]
    fn test_waiting_count() {
        let mut graph = WaitForGraph::new();
        assert_eq!(graph.waiting_count(), 0);

        graph.add_wait(
            RobotId(1),
            WaitingFor::Edge {
                edge_id: EdgeId(100),
                blocked_by: vec![RobotId(2)],
            },
        );
        assert_eq!(graph.waiting_count(), 1);

        graph.add_wait(
            RobotId(2),
            WaitingFor::Node {
                node_id: NodeId(50),
                blocked_by: vec![RobotId(1)],
            },
        );
        assert_eq!(graph.waiting_count(), 2);

        graph.remove_wait(RobotId(1));
        assert_eq!(graph.waiting_count(), 1);
    }
}
