//! Simulation events for the discrete-event simulation

use crate::{
    BinId, ChargingStationId, EdgeId, EventId, MaintenanceStationId, NodeId, OrderId, RobotId,
    ShipmentId, SimTime, SkuId, StationId, TaskId,
};
use rkyv::{Archive, Deserialize, Serialize};
use std::cmp::Ordering;

/// All possible simulation events
#[derive(Archive, Deserialize, Serialize, Clone, Debug)]
pub enum SimEvent {
    /// New order arrives in the system
    OrderArrival { order_id: OrderId },

    /// Task is assigned to a robot
    TaskAssignment { task_id: TaskId, robot_id: RobotId },

    /// Robot departs from a node to traverse an edge
    RobotDepartNode {
        robot_id: RobotId,
        from_node: NodeId,
        to_node: NodeId,
        edge_id: EdgeId,
    },

    /// Robot arrives at a node after traversing an edge
    RobotArriveNode {
        robot_id: RobotId,
        node_id: NodeId,
        from_node: NodeId,
    },

    /// Robot begins service at a station
    StationServiceStart {
        robot_id: RobotId,
        station_id: StationId,
        task_id: TaskId,
    },

    /// Robot completes service at a station
    StationServiceEnd {
        robot_id: RobotId,
        station_id: StationId,
        task_id: TaskId,
    },

    /// Inventory is updated (pick decrements, putaway increments)
    InventoryUpdate {
        sku_id: SkuId,
        bin_id: BinId,
        delta: i32,
        task_id: TaskId,
    },

    /// Robot starts waiting for an edge to become available
    RobotWaitStart {
        robot_id: RobotId,
        at_node: NodeId,
        waiting_for_edge: EdgeId,
    },

    /// Robot wait ends, can attempt to proceed
    RobotWaitEnd { robot_id: RobotId, at_node: NodeId },

    /// Robot pickup at bin location
    RobotPickup {
        robot_id: RobotId,
        task_id: TaskId,
        node_id: NodeId,
    },

    /// Try to dispatch pending tasks to available robots
    DispatchTasks,

    // === v1: Inbound/Outbound Flow Events ===
    /// Shipment arrives at inbound station
    InboundArrival {
        shipment_id: ShipmentId,
        station_id: StationId,
    },

    /// Putaway task created from inbound shipment
    PutawayTaskCreated {
        task_id: TaskId,
        shipment_id: ShipmentId,
    },

    /// Order is ready for outbound (all picks complete)
    OutboundReady { order_id: OrderId },

    /// Shipment departs from outbound station
    ShipmentDeparture {
        shipment_id: ShipmentId,
        station_id: StationId,
    },

    /// Replenishment triggered due to low inventory
    ReplenishmentTrigger {
        sku_id: SkuId,
        bin_id: BinId,
        current_qty: u32,
        threshold: u32,
    },

    // === v1: Battery & Charging Events ===
    /// Robot starts charging at a station
    RobotChargingStart {
        robot_id: RobotId,
        station_id: ChargingStationId,
    },

    /// Robot completes charging
    RobotChargingEnd {
        robot_id: RobotId,
        station_id: ChargingStationId,
        energy_charged_wh: f64,
    },

    /// Robot battery drops below threshold
    RobotLowBattery { robot_id: RobotId, soc: f64 },

    // === v1: Metrics Events ===
    /// Periodic metrics sampling tick
    MetricsSampleTick,

    // === v2: Traffic & Safety Events ===
    /// Deadlock detected between robots
    DeadlockDetected {
        /// Robot IDs involved in the deadlock cycle
        robots: Vec<RobotId>,
    },

    /// Deadlock resolution action taken
    DeadlockResolved {
        /// Robot IDs that were in the deadlock
        robots: Vec<RobotId>,
        /// Robot that was selected to resolve the deadlock
        resolver_robot: RobotId,
    },

    // === v3: Robot Failures & Maintenance Events ===
    /// Robot has failed and needs repair
    RobotFailure {
        robot_id: RobotId,
        /// Task that was interrupted by the failure, if any
        interrupted_task: Option<TaskId>,
    },

    /// Robot scheduled maintenance is due
    RobotMaintenanceDue {
        robot_id: RobotId,
        /// Operating hours since last maintenance
        operating_hours: f64,
    },

    /// Robot starts maintenance or repair at a station
    MaintenanceStart {
        robot_id: RobotId,
        station_id: MaintenanceStationId,
        /// True if this is a repair (failure recovery), false if scheduled maintenance
        is_repair: bool,
    },

    /// Robot completes maintenance or repair
    MaintenanceEnd {
        robot_id: RobotId,
        station_id: MaintenanceStationId,
        /// True if this was a repair, false if scheduled maintenance
        is_repair: bool,
        /// Duration of the maintenance/repair in seconds
        duration_s: f64,
    },
}

impl SimEvent {
    /// Get a string name for the event type
    pub fn event_type_name(&self) -> &'static str {
        match self {
            SimEvent::OrderArrival { .. } => "order_arrival",
            SimEvent::TaskAssignment { .. } => "task_assignment",
            SimEvent::RobotDepartNode { .. } => "robot_depart_node",
            SimEvent::RobotArriveNode { .. } => "robot_arrive_node",
            SimEvent::StationServiceStart { .. } => "station_service_start",
            SimEvent::StationServiceEnd { .. } => "station_service_end",
            SimEvent::InventoryUpdate { .. } => "inventory_update",
            SimEvent::RobotWaitStart { .. } => "robot_wait_start",
            SimEvent::RobotWaitEnd { .. } => "robot_wait_end",
            SimEvent::RobotPickup { .. } => "robot_pickup",
            SimEvent::DispatchTasks => "dispatch_tasks",
            // v1: Inbound/Outbound flow events
            SimEvent::InboundArrival { .. } => "inbound_arrival",
            SimEvent::PutawayTaskCreated { .. } => "putaway_task_created",
            SimEvent::OutboundReady { .. } => "outbound_ready",
            SimEvent::ShipmentDeparture { .. } => "shipment_departure",
            SimEvent::ReplenishmentTrigger { .. } => "replenishment_trigger",
            // v1: Battery & Charging events
            SimEvent::RobotChargingStart { .. } => "robot_charging_start",
            SimEvent::RobotChargingEnd { .. } => "robot_charging_end",
            SimEvent::RobotLowBattery { .. } => "robot_low_battery",
            // v1: Metrics events
            SimEvent::MetricsSampleTick => "metrics_sample_tick",
            // v2: Traffic & Safety events
            SimEvent::DeadlockDetected { .. } => "deadlock_detected",
            SimEvent::DeadlockResolved { .. } => "deadlock_resolved",
            // v3: Robot Failures & Maintenance events
            SimEvent::RobotFailure { .. } => "robot_failure",
            SimEvent::RobotMaintenanceDue { .. } => "robot_maintenance_due",
            SimEvent::MaintenanceStart { .. } => "maintenance_start",
            SimEvent::MaintenanceEnd { .. } => "maintenance_end",
        }
    }

    /// Get the robot ID associated with this event, if any
    pub fn robot_id(&self) -> Option<RobotId> {
        match self {
            SimEvent::TaskAssignment { robot_id, .. } => Some(*robot_id),
            SimEvent::RobotDepartNode { robot_id, .. } => Some(*robot_id),
            SimEvent::RobotArriveNode { robot_id, .. } => Some(*robot_id),
            SimEvent::StationServiceStart { robot_id, .. } => Some(*robot_id),
            SimEvent::StationServiceEnd { robot_id, .. } => Some(*robot_id),
            SimEvent::RobotWaitStart { robot_id, .. } => Some(*robot_id),
            SimEvent::RobotWaitEnd { robot_id, .. } => Some(*robot_id),
            SimEvent::RobotPickup { robot_id, .. } => Some(*robot_id),
            SimEvent::RobotChargingStart { robot_id, .. } => Some(*robot_id),
            SimEvent::RobotChargingEnd { robot_id, .. } => Some(*robot_id),
            SimEvent::RobotLowBattery { robot_id, .. } => Some(*robot_id),
            SimEvent::DeadlockResolved { resolver_robot, .. } => Some(*resolver_robot),
            SimEvent::RobotFailure { robot_id, .. } => Some(*robot_id),
            SimEvent::RobotMaintenanceDue { robot_id, .. } => Some(*robot_id),
            SimEvent::MaintenanceStart { robot_id, .. } => Some(*robot_id),
            SimEvent::MaintenanceEnd { robot_id, .. } => Some(*robot_id),
            _ => None,
        }
    }

    /// Get the task ID associated with this event, if any
    pub fn task_id(&self) -> Option<TaskId> {
        match self {
            SimEvent::TaskAssignment { task_id, .. } => Some(*task_id),
            SimEvent::StationServiceStart { task_id, .. } => Some(*task_id),
            SimEvent::StationServiceEnd { task_id, .. } => Some(*task_id),
            SimEvent::InventoryUpdate { task_id, .. } => Some(*task_id),
            SimEvent::RobotPickup { task_id, .. } => Some(*task_id),
            SimEvent::PutawayTaskCreated { task_id, .. } => Some(*task_id),
            _ => None,
        }
    }

    /// Get the shipment ID associated with this event, if any
    pub fn shipment_id(&self) -> Option<ShipmentId> {
        match self {
            SimEvent::InboundArrival { shipment_id, .. } => Some(*shipment_id),
            SimEvent::PutawayTaskCreated { shipment_id, .. } => Some(*shipment_id),
            SimEvent::ShipmentDeparture { shipment_id, .. } => Some(*shipment_id),
            _ => None,
        }
    }

    /// Get the charging station ID associated with this event, if any
    pub fn charging_station_id(&self) -> Option<ChargingStationId> {
        match self {
            SimEvent::RobotChargingStart { station_id, .. } => Some(*station_id),
            SimEvent::RobotChargingEnd { station_id, .. } => Some(*station_id),
            _ => None,
        }
    }

    /// Get the maintenance station ID associated with this event, if any
    pub fn maintenance_station_id(&self) -> Option<MaintenanceStationId> {
        match self {
            SimEvent::MaintenanceStart { station_id, .. } => Some(*station_id),
            SimEvent::MaintenanceEnd { station_id, .. } => Some(*station_id),
            _ => None,
        }
    }
}

/// A scheduled event with timestamp and unique ID
#[derive(Archive, Deserialize, Serialize, Clone, Debug)]
pub struct ScheduledEvent {
    pub id: EventId,
    pub time: SimTime,
    pub event: SimEvent,
}

impl ScheduledEvent {
    /// Create a new scheduled event
    pub fn new(id: EventId, time: SimTime, event: SimEvent) -> Self {
        Self { id, time, event }
    }
}

impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ScheduledEvent {}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (earliest time first)
        // If times are equal, use event ID for deterministic ordering
        match other.time.0.partial_cmp(&self.time.0) {
            Some(Ordering::Equal) | None => other.id.0.cmp(&self.id.0),
            Some(ord) => ord,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_ordering() {
        let e1 = ScheduledEvent::new(
            EventId(1),
            SimTime::from_seconds(10.0),
            SimEvent::DispatchTasks,
        );
        let e2 = ScheduledEvent::new(
            EventId(2),
            SimTime::from_seconds(5.0),
            SimEvent::DispatchTasks,
        );

        // e2 should come first (earlier time)
        assert!(e2 > e1);
    }

    #[test]
    fn test_event_type_name() {
        let event = SimEvent::OrderArrival {
            order_id: OrderId(1),
        };
        assert_eq!(event.event_type_name(), "order_arrival");
    }
}
