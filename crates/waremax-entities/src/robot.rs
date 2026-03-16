//! Robot entity and state machine

use rkyv::{Archive, Deserialize, Serialize};
use waremax_core::{RobotId, NodeId, EdgeId, StationId, TaskId, SimTime};
use std::collections::VecDeque;

/// Robot state in the simulation
#[derive(Archive, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum RobotState {
    /// Robot is idle and available for tasks
    Idle,
    /// Robot is moving to a destination
    Moving { destination: NodeId },
    /// Robot is waiting for an edge to become available
    Waiting { for_edge: EdgeId },
    /// Robot is being serviced at a station
    Servicing { at_station: StationId },
    /// Robot is picking up items at a bin
    PickingUp { at_node: NodeId },
}

impl Default for RobotState {
    fn default() -> Self {
        RobotState::Idle
    }
}

/// A robot in the warehouse
#[derive(Clone, Debug)]
pub struct Robot {
    pub id: RobotId,
    pub current_node: NodeId,
    pub state: RobotState,
    pub max_speed_mps: f64,
    pub max_payload_kg: f64,
    pub task_queue: VecDeque<TaskId>,
    pub current_task: Option<TaskId>,
    pub current_path: Vec<NodeId>,
    pub path_index: usize,

    // Statistics
    pub total_distance: f64,
    pub total_idle_time: SimTime,
    pub total_wait_time: SimTime,
    pub total_service_time: SimTime,
    pub total_move_time: SimTime,
    pub last_state_change: SimTime,
    pub tasks_completed: u32,
}

impl Robot {
    pub fn new(id: RobotId, start_node: NodeId, max_speed_mps: f64, max_payload_kg: f64) -> Self {
        Self {
            id,
            current_node: start_node,
            state: RobotState::Idle,
            max_speed_mps,
            max_payload_kg,
            task_queue: VecDeque::new(),
            current_task: None,
            current_path: Vec::new(),
            path_index: 0,
            total_distance: 0.0,
            total_idle_time: SimTime::ZERO,
            total_wait_time: SimTime::ZERO,
            total_service_time: SimTime::ZERO,
            total_move_time: SimTime::ZERO,
            last_state_change: SimTime::ZERO,
            tasks_completed: 0,
        }
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.state, RobotState::Idle)
    }

    pub fn is_available(&self) -> bool {
        self.is_idle() && self.current_task.is_none()
    }

    pub fn travel_time(&self, distance: f64) -> SimTime {
        SimTime::from_seconds(distance / self.max_speed_mps)
    }

    pub fn assign_task(&mut self, task_id: TaskId) {
        self.task_queue.push_back(task_id);
    }

    pub fn start_task(&mut self, task_id: TaskId) {
        self.current_task = Some(task_id);
    }

    pub fn complete_task(&mut self) {
        self.current_task = None;
        self.tasks_completed += 1;
    }

    pub fn set_path(&mut self, path: Vec<NodeId>) {
        self.current_path = path;
        self.path_index = 0;
    }

    pub fn next_node_in_path(&self) -> Option<NodeId> {
        if self.path_index + 1 < self.current_path.len() {
            Some(self.current_path[self.path_index + 1])
        } else {
            None
        }
    }

    pub fn advance_path(&mut self) {
        if self.path_index + 1 < self.current_path.len() {
            self.path_index += 1;
        }
    }

    pub fn has_reached_destination(&self) -> bool {
        self.path_index + 1 >= self.current_path.len()
    }

    pub fn update_stats(&mut self, current_time: SimTime) {
        let duration = current_time - self.last_state_change;
        match &self.state {
            RobotState::Idle => self.total_idle_time += duration,
            RobotState::Moving { .. } => self.total_move_time += duration,
            RobotState::Waiting { .. } => self.total_wait_time += duration,
            RobotState::Servicing { .. } | RobotState::PickingUp { .. } => {
                self.total_service_time += duration
            }
        }
        self.last_state_change = current_time;
    }

    pub fn utilization(&self, total_time: SimTime) -> f64 {
        if total_time.is_zero() {
            return 0.0;
        }
        let active_time = self.total_move_time + self.total_service_time;
        active_time.as_seconds() / total_time.as_seconds()
    }
}
