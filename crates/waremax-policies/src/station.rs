//! Station assignment policies

use crate::traits::{StationAssignmentPolicy, PolicyContext};
use waremax_core::StationId;
use waremax_entities::{Task, StationType};

/// Assign tasks to the station with the least queue
pub struct LeastQueuePolicy {
    station_type: StationType,
}

impl LeastQueuePolicy {
    pub fn new(station_type: StationType) -> Self {
        Self { station_type }
    }

    pub fn for_pick() -> Self {
        Self::new(StationType::Pick)
    }
}

impl Default for LeastQueuePolicy {
    fn default() -> Self {
        Self::for_pick()
    }
}

impl StationAssignmentPolicy for LeastQueuePolicy {
    fn assign(&self, ctx: &PolicyContext, _task: &Task) -> Option<StationId> {
        ctx.stations
            .values()
            .filter(|s| s.station_type == self.station_type)
            .filter(|s| s.can_accept())
            .min_by_key(|s| s.queue_length())
            .map(|s| s.id)
    }

    fn name(&self) -> &'static str {
        "least_queue"
    }
}

/// Assign tasks to the nearest station of matching type
pub struct NearestStationPolicy {
    station_type: StationType,
}

impl NearestStationPolicy {
    pub fn new(station_type: StationType) -> Self {
        Self { station_type }
    }

    pub fn for_pick() -> Self {
        Self::new(StationType::Pick)
    }
}

impl Default for NearestStationPolicy {
    fn default() -> Self {
        Self::for_pick()
    }
}

impl StationAssignmentPolicy for NearestStationPolicy {
    fn assign(&self, ctx: &PolicyContext, task: &Task) -> Option<StationId> {
        let task_node = task.source.access_node;

        ctx.stations
            .values()
            .filter(|s| s.station_type == self.station_type)
            .filter(|s| s.can_accept())
            .min_by(|a, b| {
                let dist_a = ctx.map.euclidean_distance(task_node, a.node);
                let dist_b = ctx.map.euclidean_distance(task_node, b.node);
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.id)
    }

    fn name(&self) -> &'static str {
        "nearest_station"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use waremax_core::{NodeId, OrderId, RobotId, TaskId, SimTime};
    use waremax_entities::{Robot, Task, BinLocation, Station, Order, ServiceTimeModel};
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

    fn make_station(id: u32, node: u32, queue_len: usize) -> Station {
        let mut station = Station::new(
            StationId(id),
            format!("S{}", id),
            NodeId(node),
            StationType::Pick,
            2,
            None,
            ServiceTimeModel::default(),
        );
        // Add items to queue to simulate queue length
        for i in 0..queue_len {
            station.enqueue(RobotId(100 + i as u32));
        }
        station
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

    fn make_map_with_nodes() -> WarehouseMap {
        let mut map = WarehouseMap::new();
        // Create nodes at specific positions
        map.add_node(Node::new(NodeId(0), "N0".to_string(), 0.0, 0.0, NodeType::Aisle));
        map.add_node(Node::new(NodeId(1), "N1".to_string(), 10.0, 0.0, NodeType::StationPick));
        map.add_node(Node::new(NodeId(2), "N2".to_string(), 5.0, 0.0, NodeType::StationPick));
        map.add_node(Node::new(NodeId(3), "N3".to_string(), 20.0, 0.0, NodeType::StationPick));
        map
    }

    #[test]
    fn test_least_queue_selects_shortest() {
        let map = WarehouseMap::new();
        let robots = HashMap::new();
        let orders = HashMap::new();

        let mut stations = HashMap::new();
        stations.insert(StationId(0), make_station(0, 1, 3)); // Queue of 3
        stations.insert(StationId(1), make_station(1, 2, 1)); // Queue of 1 (shortest)
        stations.insert(StationId(2), make_station(2, 3, 2)); // Queue of 2

        let task = make_task(0, 0);
        let mut tasks = HashMap::new();
        tasks.insert(TaskId(0), task.clone());

        let ctx = test_context(&map, &robots, &tasks, &stations, &orders);
        let policy = LeastQueuePolicy::default();

        // Should select station 1 with shortest queue
        assert_eq!(policy.assign(&ctx, &task), Some(StationId(1)));
    }

    #[test]
    fn test_nearest_station_selects_closest() {
        let map = make_map_with_nodes();
        let robots = HashMap::new();
        let orders = HashMap::new();

        let mut stations = HashMap::new();
        stations.insert(StationId(0), make_station(0, 1, 0)); // At x=10
        stations.insert(StationId(1), make_station(1, 2, 0)); // At x=5 (closest to task at x=0)
        stations.insert(StationId(2), make_station(2, 3, 0)); // At x=20

        let task = make_task(0, 0); // Task at node 0 (x=0)
        let mut tasks = HashMap::new();
        tasks.insert(TaskId(0), task.clone());

        let ctx = test_context(&map, &robots, &tasks, &stations, &orders);
        let policy = NearestStationPolicy::default();

        // Should select station 1 at node 2 (x=5), closest to task at x=0
        assert_eq!(policy.assign(&ctx, &task), Some(StationId(1)));
    }

    #[test]
    fn test_policy_names() {
        assert_eq!(LeastQueuePolicy::default().name(), "least_queue");
        assert_eq!(NearestStationPolicy::default().name(), "nearest_station");
    }
}
