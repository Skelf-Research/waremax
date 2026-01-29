//! Task allocation policies

use crate::traits::{TaskAllocationPolicy, PolicyContext};
use waremax_core::{RobotId, TaskId};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Allocate tasks to the nearest idle robot
pub struct NearestRobotPolicy;

impl NearestRobotPolicy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NearestRobotPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskAllocationPolicy for NearestRobotPolicy {
    fn allocate(&self, ctx: &PolicyContext, task_id: TaskId) -> Option<RobotId> {
        let task = ctx.tasks.get(&task_id)?;
        let pickup_node = task.source.access_node;

        // Find idle robots and calculate distance to pickup
        let mut candidates: Vec<(RobotId, f64)> = ctx
            .robots
            .values()
            .filter(|r| r.is_available())
            .map(|r| {
                let dist = ctx.map.euclidean_distance(r.current_node, pickup_node);
                (r.id, dist)
            })
            .collect();

        // Sort by distance (nearest first)
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        candidates.first().map(|(id, _)| *id)
    }

    fn name(&self) -> &'static str {
        "nearest_robot"
    }
}

/// Round-robin task allocation - cycles through available robots
pub struct RoundRobinPolicy {
    last_index: AtomicUsize,
}

impl RoundRobinPolicy {
    pub fn new() -> Self {
        Self {
            last_index: AtomicUsize::new(0),
        }
    }
}

impl Default for RoundRobinPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskAllocationPolicy for RoundRobinPolicy {
    fn allocate(&self, ctx: &PolicyContext, _task_id: TaskId) -> Option<RobotId> {
        let mut available: Vec<_> = ctx.robots.values()
            .filter(|r| r.is_available())
            .collect();

        if available.is_empty() {
            return None;
        }

        // Sort by robot ID for deterministic ordering
        available.sort_by_key(|r| r.id.0);

        let idx = self.last_index.fetch_add(1, Ordering::Relaxed) % available.len();

        Some(available[idx].id)
    }

    fn name(&self) -> &'static str {
        "round_robin"
    }
}

/// Allocate tasks to the robot with the smallest task queue
pub struct LeastBusyPolicy;

impl LeastBusyPolicy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LeastBusyPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskAllocationPolicy for LeastBusyPolicy {
    fn allocate(&self, ctx: &PolicyContext, _task_id: TaskId) -> Option<RobotId> {
        ctx.robots
            .values()
            .filter(|r| r.is_available())
            .min_by_key(|r| r.task_queue.len())
            .map(|r| r.id)
    }

    fn name(&self) -> &'static str {
        "least_busy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use waremax_core::{NodeId, OrderId, StationId, SimTime};
    use waremax_entities::{Robot, Task, BinLocation, Station, Order};
    use waremax_map::WarehouseMap;
    use waremax_storage::BinAddress;
    use std::collections::HashMap;

    fn test_context<'a>(
        map: &'a WarehouseMap,
        robots: &'a HashMap<RobotId, Robot>,
        tasks: &'a HashMap<TaskId, Task>,
        stations: &'a HashMap<StationId, Station>,
        orders: &'a HashMap<OrderId, Order>,
    ) -> PolicyContext<'a> {
        PolicyContext {
            current_time: SimTime::ZERO,
            map,
            robots,
            tasks,
            stations,
            orders,
        }
    }

    fn make_robot(id: u32, node: u32) -> Robot {
        Robot::new(RobotId(id), NodeId(node), 1.5, 25.0)
    }

    fn make_task(id: u32, node: u32) -> Task {
        Task::new_pick(
            TaskId(id),
            OrderId(0),
            waremax_core::SkuId(0),
            1,
            BinLocation::new(BinAddress::new(waremax_core::RackId(0), 0, 0), NodeId(node)),
            StationId(0),
            SimTime::ZERO,
        )
    }

    #[test]
    fn test_round_robin_cycles() {
        let map = WarehouseMap::new();
        let mut robots = HashMap::new();
        robots.insert(RobotId(0), make_robot(0, 0));
        robots.insert(RobotId(1), make_robot(1, 1));
        robots.insert(RobotId(2), make_robot(2, 2));

        let mut tasks = HashMap::new();
        tasks.insert(TaskId(0), make_task(0, 5));

        let stations = HashMap::new();
        let orders = HashMap::new();

        let policy = RoundRobinPolicy::new();
        let ctx = test_context(&map, &robots, &tasks, &stations, &orders);

        // Should cycle through robots 0, 1, 2, 0, 1, 2...
        let r1 = policy.allocate(&ctx, TaskId(0));
        let r2 = policy.allocate(&ctx, TaskId(0));
        let r3 = policy.allocate(&ctx, TaskId(0));
        let r4 = policy.allocate(&ctx, TaskId(0));

        assert_eq!(r1, Some(RobotId(0)));
        assert_eq!(r2, Some(RobotId(1)));
        assert_eq!(r3, Some(RobotId(2)));
        assert_eq!(r4, Some(RobotId(0))); // Wraps around
    }

    #[test]
    fn test_round_robin_no_available() {
        let map = WarehouseMap::new();
        let mut robots = HashMap::new();
        let mut robot = make_robot(0, 0);
        robot.start_task(TaskId(99)); // Make busy
        robots.insert(RobotId(0), robot);

        let tasks = HashMap::new();
        let stations = HashMap::new();
        let orders = HashMap::new();

        let policy = RoundRobinPolicy::new();
        let ctx = test_context(&map, &robots, &tasks, &stations, &orders);

        assert_eq!(policy.allocate(&ctx, TaskId(0)), None);
    }

    #[test]
    fn test_least_busy_selects_smallest_queue() {
        let map = WarehouseMap::new();
        let mut robots = HashMap::new();

        let mut robot0 = make_robot(0, 0);
        robot0.assign_task(TaskId(10));
        robot0.assign_task(TaskId(11));
        robots.insert(RobotId(0), robot0);

        let mut robot1 = make_robot(1, 1);
        robot1.assign_task(TaskId(12));
        robots.insert(RobotId(1), robot1);

        let robot2 = make_robot(2, 2); // Empty queue
        robots.insert(RobotId(2), robot2);

        let tasks = HashMap::new();
        let stations = HashMap::new();
        let orders = HashMap::new();

        let policy = LeastBusyPolicy::new();
        let ctx = test_context(&map, &robots, &tasks, &stations, &orders);

        // Should select robot 2 with empty queue
        assert_eq!(policy.allocate(&ctx, TaskId(0)), Some(RobotId(2)));
    }

    #[test]
    fn test_policy_names() {
        assert_eq!(NearestRobotPolicy::new().name(), "nearest_robot");
        assert_eq!(RoundRobinPolicy::new().name(), "round_robin");
        assert_eq!(LeastBusyPolicy::new().name(), "least_busy");
    }
}
