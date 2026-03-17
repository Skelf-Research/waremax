//! Batching policies

use crate::traits::{BatchingPolicy, PolicyContext};
use waremax_core::TaskId;

/// No batching - each task is independent
pub struct NoBatchingPolicy;

impl NoBatchingPolicy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoBatchingPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchingPolicy for NoBatchingPolicy {
    fn batch(&self, _ctx: &PolicyContext, pending_tasks: &[TaskId]) -> Vec<Vec<TaskId>> {
        // Each task is its own "batch"
        pending_tasks.iter().map(|&t| vec![t]).collect()
    }

    fn name(&self) -> &'static str {
        "none"
    }
}

/// Groups tasks by proximity (zone-based clustering)
pub struct ZoneBatchingPolicy {
    max_items: u32,
    zone_radius: f64,
}

impl ZoneBatchingPolicy {
    pub fn new(max_items: u32, zone_radius: f64) -> Self {
        Self { max_items, zone_radius }
    }

    /// Default zone batching: max 5 items, 10m radius
    pub fn default_zone() -> Self {
        Self::new(5, 10.0)
    }
}

impl Default for ZoneBatchingPolicy {
    fn default() -> Self {
        Self::default_zone()
    }
}

impl BatchingPolicy for ZoneBatchingPolicy {
    fn batch(&self, ctx: &PolicyContext, pending_tasks: &[TaskId]) -> Vec<Vec<TaskId>> {
        let mut batches = Vec::new();
        let mut used = vec![false; pending_tasks.len()];

        for (i, &task_id) in pending_tasks.iter().enumerate() {
            if used[i] {
                continue;
            }

            let mut batch = vec![task_id];
            used[i] = true;

            if let Some(task) = ctx.tasks.get(&task_id) {
                let anchor = task.source.access_node;

                for (j, &other_id) in pending_tasks.iter().enumerate().skip(i + 1) {
                    if used[j] || batch.len() >= self.max_items as usize {
                        continue;
                    }

                    if let Some(other) = ctx.tasks.get(&other_id) {
                        let dist = ctx.map.euclidean_distance(anchor, other.source.access_node);
                        if dist <= self.zone_radius {
                            batch.push(other_id);
                            used[j] = true;
                        }
                    }
                }
            }

            batches.push(batch);
        }

        batches
    }

    fn name(&self) -> &'static str {
        "zone"
    }
}

// === v1: Additional Batching Policies ===

/// Groups tasks by destination station
/// Respects max_items and optional max_weight constraints
pub struct StationBatchPolicy {
    max_items: u32,
    max_weight_kg: Option<f64>,
}

impl StationBatchPolicy {
    pub fn new(max_items: u32, max_weight_kg: Option<f64>) -> Self {
        Self { max_items, max_weight_kg }
    }

    pub fn items_only(max_items: u32) -> Self {
        Self::new(max_items, None)
    }
}

impl Default for StationBatchPolicy {
    fn default() -> Self {
        Self::new(5, None)
    }
}

impl BatchingPolicy for StationBatchPolicy {
    fn batch(&self, ctx: &PolicyContext, pending_tasks: &[TaskId]) -> Vec<Vec<TaskId>> {
        use std::collections::HashMap;

        // Group tasks by destination station
        let mut by_station: HashMap<waremax_core::StationId, Vec<TaskId>> = HashMap::new();

        for &task_id in pending_tasks {
            if let Some(task) = ctx.tasks.get(&task_id) {
                by_station
                    .entry(task.destination_station)
                    .or_default()
                    .push(task_id);
            }
        }

        let mut batches = Vec::new();

        // For each station group, create batches respecting limits
        for (_station_id, station_tasks) in by_station {
            let mut current_batch = Vec::new();
            let mut current_weight = 0.0;

            for task_id in station_tasks {
                // Check if we need to start a new batch
                let at_item_limit = current_batch.len() >= self.max_items as usize;
                let at_weight_limit = if let Some(max_weight) = self.max_weight_kg {
                    // Estimate task weight from quantity (assume 1kg per item as default)
                    let task_weight = ctx.tasks.get(&task_id)
                        .map(|t| t.quantity as f64)
                        .unwrap_or(1.0);
                    current_weight + task_weight > max_weight
                } else {
                    false
                };

                if !current_batch.is_empty() && (at_item_limit || at_weight_limit) {
                    batches.push(current_batch);
                    current_batch = Vec::new();
                    current_weight = 0.0;
                }

                // Add task to current batch
                if let Some(task) = ctx.tasks.get(&task_id) {
                    current_weight += task.quantity as f64;
                }
                current_batch.push(task_id);
            }

            if !current_batch.is_empty() {
                batches.push(current_batch);
            }
        }

        // If no batches were created, return individual tasks
        if batches.is_empty() {
            return pending_tasks.iter().map(|&t| vec![t]).collect();
        }

        batches
    }

    fn name(&self) -> &'static str {
        "station_batch"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use waremax_core::{NodeId, OrderId, RobotId, StationId, SimTime};
    use waremax_entities::{Robot, Task, BinLocation, Station, Order};
    use waremax_map::{WarehouseMap, Node, NodeType};
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

    fn make_task_at_node(id: u32, node: u32) -> Task {
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

    fn make_map_with_nodes() -> WarehouseMap {
        let mut map = WarehouseMap::new();
        // Create nodes at specific positions for zone testing
        map.add_node(Node::new(NodeId(0), "N0".to_string(), 0.0, 0.0, NodeType::Rack));
        map.add_node(Node::new(NodeId(1), "N1".to_string(), 5.0, 0.0, NodeType::Rack)); // 5m from 0
        map.add_node(Node::new(NodeId(2), "N2".to_string(), 8.0, 0.0, NodeType::Rack)); // 8m from 0
        map.add_node(Node::new(NodeId(3), "N3".to_string(), 50.0, 0.0, NodeType::Rack)); // 50m from 0 (far)
        map.add_node(Node::new(NodeId(4), "N4".to_string(), 52.0, 0.0, NodeType::Rack)); // 52m from 0 (far, close to 3)
        map
    }

    #[test]
    fn test_no_batching_creates_individual_batches() {
        let map = WarehouseMap::new();
        let robots = HashMap::new();
        let stations = HashMap::new();
        let orders = HashMap::new();
        let tasks = HashMap::new();

        let ctx = test_context(&map, &robots, &tasks, &stations, &orders);
        let policy = NoBatchingPolicy::new();

        let pending = vec![TaskId(0), TaskId(1), TaskId(2)];
        let batches = policy.batch(&ctx, &pending);

        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0], vec![TaskId(0)]);
        assert_eq!(batches[1], vec![TaskId(1)]);
        assert_eq!(batches[2], vec![TaskId(2)]);
    }

    #[test]
    fn test_zone_batching_groups_nearby() {
        let map = make_map_with_nodes();
        let robots = HashMap::new();
        let stations = HashMap::new();
        let orders = HashMap::new();

        let mut tasks = HashMap::new();
        tasks.insert(TaskId(0), make_task_at_node(0, 0)); // At x=0
        tasks.insert(TaskId(1), make_task_at_node(1, 1)); // At x=5 (within 10m radius)
        tasks.insert(TaskId(2), make_task_at_node(2, 2)); // At x=8 (within 10m radius)
        tasks.insert(TaskId(3), make_task_at_node(3, 3)); // At x=50 (far away)
        tasks.insert(TaskId(4), make_task_at_node(4, 4)); // At x=52 (close to task 3)

        let ctx = test_context(&map, &robots, &tasks, &stations, &orders);
        let policy = ZoneBatchingPolicy::new(5, 10.0); // 10m radius

        let pending = vec![TaskId(0), TaskId(1), TaskId(2), TaskId(3), TaskId(4)];
        let batches = policy.batch(&ctx, &pending);

        // Should create 2 batches:
        // Batch 1: tasks 0, 1, 2 (all within 10m of each other)
        // Batch 2: tasks 3, 4 (both far from batch 1, but close to each other)
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0], vec![TaskId(0), TaskId(1), TaskId(2)]);
        assert_eq!(batches[1], vec![TaskId(3), TaskId(4)]);
    }

    #[test]
    fn test_zone_batching_respects_max_items() {
        let map = make_map_with_nodes();
        let robots = HashMap::new();
        let stations = HashMap::new();
        let orders = HashMap::new();

        let mut tasks = HashMap::new();
        tasks.insert(TaskId(0), make_task_at_node(0, 0));
        tasks.insert(TaskId(1), make_task_at_node(1, 1));
        tasks.insert(TaskId(2), make_task_at_node(2, 2));

        let ctx = test_context(&map, &robots, &tasks, &stations, &orders);
        let policy = ZoneBatchingPolicy::new(2, 10.0); // Max 2 items per batch

        let pending = vec![TaskId(0), TaskId(1), TaskId(2)];
        let batches = policy.batch(&ctx, &pending);

        // Should create 2 batches due to max_items=2
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0], vec![TaskId(0), TaskId(1)]);
        assert_eq!(batches[1], vec![TaskId(2)]);
    }

    #[test]
    fn test_policy_names() {
        assert_eq!(NoBatchingPolicy::new().name(), "none");
        assert_eq!(ZoneBatchingPolicy::default().name(), "zone");
    }
}
