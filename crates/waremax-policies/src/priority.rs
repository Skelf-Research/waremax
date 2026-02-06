//! Priority arbitration policies

use crate::traits::{PolicyContext, PriorityPolicy};
use waremax_core::{SimTime, TaskId};
use waremax_entities::TaskType;

/// Strict priority: pick > replen > putaway
pub struct StrictPriorityPolicy;

impl StrictPriorityPolicy {
    pub fn new() -> Self {
        Self
    }

    fn task_priority(task_type: &TaskType) -> u32 {
        match task_type {
            TaskType::Pick => 0, // Highest
            TaskType::Replenishment => 1,
            TaskType::Putaway => 2, // Lowest
        }
    }
}

impl Default for StrictPriorityPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl PriorityPolicy for StrictPriorityPolicy {
    fn prioritize(&self, ctx: &PolicyContext, tasks: &mut [TaskId]) {
        tasks.sort_by_key(|t| {
            ctx.tasks
                .get(t)
                .map(|task| Self::task_priority(&task.task_type))
                .unwrap_or(u32::MAX)
        });
    }

    fn name(&self) -> &'static str {
        "strict_priority"
    }
}

/// First-in-first-out priority by task creation time
pub struct FifoPolicy;

impl FifoPolicy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FifoPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl PriorityPolicy for FifoPolicy {
    fn prioritize(&self, ctx: &PolicyContext, tasks: &mut [TaskId]) {
        tasks.sort_by(|a, b| {
            let time_a = ctx
                .tasks
                .get(a)
                .map(|t| t.created_at)
                .unwrap_or(SimTime::MAX);
            let time_b = ctx
                .tasks
                .get(b)
                .map(|t| t.created_at)
                .unwrap_or(SimTime::MAX);
            time_a
                .partial_cmp(&time_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    fn name(&self) -> &'static str {
        "fifo"
    }
}

/// Prioritizes tasks from orders with nearest due time (earliest due first)
pub struct DueTimePolicy;

impl DueTimePolicy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DueTimePolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl PriorityPolicy for DueTimePolicy {
    fn prioritize(&self, ctx: &PolicyContext, tasks: &mut [TaskId]) {
        tasks.sort_by(|a, b| {
            let due_a = ctx
                .tasks
                .get(a)
                .and_then(|t| t.order_id)
                .and_then(|oid| ctx.orders.get(&oid))
                .and_then(|o| o.due_time)
                .unwrap_or(SimTime::MAX);
            let due_b = ctx
                .tasks
                .get(b)
                .and_then(|t| t.order_id)
                .and_then(|oid| ctx.orders.get(&oid))
                .and_then(|o| o.due_time)
                .unwrap_or(SimTime::MAX);
            due_a
                .partial_cmp(&due_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    fn name(&self) -> &'static str {
        "due_time"
    }
}

// === v1: Additional Priority Policies ===

/// Weighted fair queuing across task types
/// Provides balanced processing of pick, putaway, and replenishment tasks
pub struct WeightedFairPolicy {
    /// Weight for pick tasks (lower = more frequent selection)
    pick_weight: u32,
    /// Weight for putaway tasks
    putaway_weight: u32,
    /// Weight for replenishment tasks
    replen_weight: u32,
}

impl WeightedFairPolicy {
    pub fn new(pick_weight: u32, putaway_weight: u32, replen_weight: u32) -> Self {
        Self {
            pick_weight,
            putaway_weight,
            replen_weight,
        }
    }

    fn task_weight(&self, task_type: &TaskType) -> u32 {
        match task_type {
            TaskType::Pick => self.pick_weight,
            TaskType::Putaway => self.putaway_weight,
            TaskType::Replenishment => self.replen_weight,
        }
    }
}

impl Default for WeightedFairPolicy {
    fn default() -> Self {
        // Default: picks get highest priority (weight 1)
        // Putaway and replenishment get lower priority (weight 2 and 3)
        Self::new(1, 2, 3)
    }
}

impl PriorityPolicy for WeightedFairPolicy {
    fn prioritize(&self, ctx: &PolicyContext, tasks: &mut [TaskId]) {
        // Sort by virtual timestamp: creation_time * weight
        // Lower virtual timestamp = higher priority
        tasks.sort_by(|a, b| {
            let vt_a = ctx
                .tasks
                .get(a)
                .map(|t| {
                    let weight = self.task_weight(&t.task_type);
                    t.created_at.as_seconds() * weight as f64
                })
                .unwrap_or(f64::MAX);

            let vt_b = ctx
                .tasks
                .get(b)
                .map(|t| {
                    let weight = self.task_weight(&t.task_type);
                    t.created_at.as_seconds() * weight as f64
                })
                .unwrap_or(f64::MAX);

            vt_a.partial_cmp(&vt_b).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    fn name(&self) -> &'static str {
        "weighted_fair"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use waremax_core::{NodeId, OrderId, RobotId, StationId};
    use waremax_entities::{BinLocation, Order, OrderLine, Robot, Station, Task};
    use waremax_map::WarehouseMap;
    use waremax_storage::BinAddress;

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

    fn make_task_with_time(id: u32, order_id: u32, created_at: f64) -> Task {
        Task::new_pick(
            TaskId(id),
            OrderId(order_id),
            waremax_core::SkuId(0),
            1,
            BinLocation::new(BinAddress::new(waremax_core::RackId(0), 0, 0), NodeId(0)),
            StationId(0),
            SimTime::from_seconds(created_at),
        )
    }

    fn make_order_with_due(id: u32, due_time: f64) -> Order {
        Order::new(
            OrderId(id),
            SimTime::ZERO,
            vec![OrderLine::new(waremax_core::SkuId(0), 1)],
            Some(SimTime::from_seconds(due_time)),
        )
    }

    #[test]
    fn test_fifo_orders_by_creation() {
        let map = WarehouseMap::new();
        let robots = HashMap::new();
        let stations = HashMap::new();
        let orders = HashMap::new();

        let mut tasks = HashMap::new();
        tasks.insert(TaskId(0), make_task_with_time(0, 0, 30.0)); // Created at 30s
        tasks.insert(TaskId(1), make_task_with_time(1, 1, 10.0)); // Created at 10s (earliest)
        tasks.insert(TaskId(2), make_task_with_time(2, 2, 20.0)); // Created at 20s

        let mut task_ids = vec![TaskId(0), TaskId(1), TaskId(2)];
        let ctx = test_context(&map, &robots, &tasks, &stations, &orders);

        let policy = FifoPolicy::new();
        policy.prioritize(&ctx, &mut task_ids);

        // Should be sorted by creation time: 1 (10s), 2 (20s), 0 (30s)
        assert_eq!(task_ids, vec![TaskId(1), TaskId(2), TaskId(0)]);
    }

    #[test]
    fn test_due_time_orders_by_deadline() {
        let map = WarehouseMap::new();
        let robots = HashMap::new();
        let stations = HashMap::new();

        let mut orders = HashMap::new();
        orders.insert(OrderId(0), make_order_with_due(0, 300.0)); // Due at 5 min
        orders.insert(OrderId(1), make_order_with_due(1, 60.0)); // Due at 1 min (earliest)
        orders.insert(OrderId(2), make_order_with_due(2, 180.0)); // Due at 3 min

        let mut tasks = HashMap::new();
        tasks.insert(TaskId(0), make_task_with_time(0, 0, 0.0));
        tasks.insert(TaskId(1), make_task_with_time(1, 1, 0.0));
        tasks.insert(TaskId(2), make_task_with_time(2, 2, 0.0));

        let mut task_ids = vec![TaskId(0), TaskId(1), TaskId(2)];
        let ctx = test_context(&map, &robots, &tasks, &stations, &orders);

        let policy = DueTimePolicy::new();
        policy.prioritize(&ctx, &mut task_ids);

        // Should be sorted by due time: 1 (1min), 2 (3min), 0 (5min)
        assert_eq!(task_ids, vec![TaskId(1), TaskId(2), TaskId(0)]);
    }

    #[test]
    fn test_policy_names() {
        assert_eq!(StrictPriorityPolicy::new().name(), "strict_priority");
        assert_eq!(FifoPolicy::new().name(), "fifo");
        assert_eq!(DueTimePolicy::new().name(), "due_time");
    }
}
