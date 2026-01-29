//! Task entity

use rkyv::{Archive, Deserialize, Serialize};
use waremax_core::{TaskId, OrderId, SkuId, RobotId, StationId, NodeId, SimTime};
use waremax_storage::rack::BinAddress;

/// Type of task
#[derive(Archive, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum TaskType {
    Pick,
    Putaway,
    Replenishment,
}

impl Default for TaskType {
    fn default() -> Self {
        TaskType::Pick
    }
}

/// Location of a bin with its access node
#[derive(Archive, Deserialize, Serialize, Clone, Debug)]
pub struct BinLocation {
    pub bin_address: BinAddress,
    pub access_node: NodeId,
}

impl BinLocation {
    pub fn new(bin_address: BinAddress, access_node: NodeId) -> Self {
        Self { bin_address, access_node }
    }
}

/// Task status
#[derive(Archive, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Pending,
    Assigned { robot: RobotId },
    MovingToPickup,
    PickingUp,
    MovingToStation,
    AtStation,
    Completed,
    Failed { reason: String },
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

/// A task to be executed by a robot
#[derive(Clone, Debug)]
pub struct Task {
    pub id: TaskId,
    pub task_type: TaskType,
    pub order_id: Option<OrderId>,
    pub sku_id: SkuId,
    pub quantity: u32,
    pub source: BinLocation,
    pub destination_station: StationId,
    /// Destination bin for putaway/replenishment tasks
    pub destination_bin: Option<BinLocation>,
    pub status: TaskStatus,
    pub assigned_robot: Option<RobotId>,
    pub created_at: SimTime,
    pub started_at: Option<SimTime>,
    pub completed_at: Option<SimTime>,
}

impl Task {
    pub fn new_pick(
        id: TaskId,
        order_id: OrderId,
        sku_id: SkuId,
        quantity: u32,
        source: BinLocation,
        destination_station: StationId,
        created_at: SimTime,
    ) -> Self {
        Self {
            id,
            task_type: TaskType::Pick,
            order_id: Some(order_id),
            sku_id,
            quantity,
            source,
            destination_station,
            destination_bin: None,
            status: TaskStatus::Pending,
            assigned_robot: None,
            created_at,
            started_at: None,
            completed_at: None,
        }
    }

    /// Create a new putaway task (inbound station → storage bin)
    pub fn new_putaway(
        id: TaskId,
        sku_id: SkuId,
        quantity: u32,
        source: BinLocation,
        destination_bin: BinLocation,
        destination_station: StationId,
        created_at: SimTime,
    ) -> Self {
        Self {
            id,
            task_type: TaskType::Putaway,
            order_id: None,
            sku_id,
            quantity,
            source,
            destination_station,
            destination_bin: Some(destination_bin),
            status: TaskStatus::Pending,
            assigned_robot: None,
            created_at,
            started_at: None,
            completed_at: None,
        }
    }

    /// Create a new replenishment task (reserve storage → pick-face bin)
    pub fn new_replenishment(
        id: TaskId,
        sku_id: SkuId,
        quantity: u32,
        source: BinLocation,
        destination_bin: BinLocation,
        destination_station: StationId,
        created_at: SimTime,
    ) -> Self {
        Self {
            id,
            task_type: TaskType::Replenishment,
            order_id: None,
            sku_id,
            quantity,
            source,
            destination_station,
            destination_bin: Some(destination_bin),
            status: TaskStatus::Pending,
            assigned_robot: None,
            created_at,
            started_at: None,
            completed_at: None,
        }
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.status, TaskStatus::Pending)
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.status, TaskStatus::Completed)
    }

    pub fn assign(&mut self, robot: RobotId, time: SimTime) {
        self.status = TaskStatus::Assigned { robot };
        self.assigned_robot = Some(robot);
        self.started_at = Some(time);
    }

    pub fn start_moving_to_pickup(&mut self) {
        self.status = TaskStatus::MovingToPickup;
    }

    pub fn start_picking_up(&mut self) {
        self.status = TaskStatus::PickingUp;
    }

    pub fn start_moving_to_station(&mut self) {
        self.status = TaskStatus::MovingToStation;
    }

    pub fn arrive_at_station(&mut self) {
        self.status = TaskStatus::AtStation;
    }

    pub fn complete(&mut self, time: SimTime) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(time);
    }

    pub fn fail(&mut self, reason: String) {
        self.status = TaskStatus::Failed { reason };
    }

    pub fn cycle_time(&self) -> Option<SimTime> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}
