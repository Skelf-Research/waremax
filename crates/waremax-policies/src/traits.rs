//! Policy trait definitions

use std::collections::HashMap;
use waremax_core::{RobotId, SimTime, StationId, TaskId};
use waremax_entities::{Order, Robot, Station, Task};
use waremax_map::WarehouseMap;

/// Context provided to policies for decision-making
pub struct PolicyContext<'a> {
    pub current_time: SimTime,
    pub map: &'a WarehouseMap,
    pub robots: &'a HashMap<RobotId, Robot>,
    pub tasks: &'a HashMap<TaskId, Task>,
    pub stations: &'a HashMap<StationId, Station>,
    pub orders: &'a HashMap<waremax_core::OrderId, Order>,
}

/// Task allocation policy: which robot should handle a task
pub trait TaskAllocationPolicy: Send + Sync {
    /// Select a robot for the given task
    fn allocate(&self, ctx: &PolicyContext, task_id: TaskId) -> Option<RobotId>;

    /// Policy name for logging
    fn name(&self) -> &'static str;
}

/// Station assignment policy: which station should receive a task
pub trait StationAssignmentPolicy: Send + Sync {
    /// Select a station for the given task
    fn assign(&self, ctx: &PolicyContext, task: &Task) -> Option<StationId>;

    /// Policy name for logging
    fn name(&self) -> &'static str;
}

/// Batching policy: how to group tasks
pub trait BatchingPolicy: Send + Sync {
    /// Given pending tasks, return groups to batch together
    fn batch(&self, ctx: &PolicyContext, pending_tasks: &[TaskId]) -> Vec<Vec<TaskId>>;

    /// Policy name for logging
    fn name(&self) -> &'static str;
}

/// Priority arbitration policy: ordering of task types
pub trait PriorityPolicy: Send + Sync {
    /// Sort tasks by priority (highest priority first)
    fn prioritize(&self, ctx: &PolicyContext, tasks: &mut [TaskId]);

    /// Policy name for logging
    fn name(&self) -> &'static str;
}
