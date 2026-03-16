//! Station entity with queue management

use rkyv::{Archive, Deserialize, Serialize};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use waremax_core::{StationId, NodeId, RobotId, SimTime};
use std::collections::VecDeque;

/// Type of station
#[derive(Archive, Deserialize, Serialize, SerdeDeserialize, SerdeSerialize, Clone, Debug, PartialEq)]
pub enum StationType {
    Pick,
    Drop,
    Inbound,
    Outbound,
}

impl Default for StationType {
    fn default() -> Self {
        StationType::Pick
    }
}

/// Service time model for a station
#[derive(Clone, Debug)]
pub struct ServiceTimeModel {
    pub base_s: f64,
    pub per_item_s: f64,
}

impl ServiceTimeModel {
    pub fn new(base_s: f64, per_item_s: f64) -> Self {
        Self { base_s, per_item_s }
    }

    pub fn calculate(&self, item_count: u32) -> SimTime {
        SimTime::from_seconds(self.base_s + self.per_item_s * item_count as f64)
    }
}

impl Default for ServiceTimeModel {
    fn default() -> Self {
        Self::new(10.0, 2.0)
    }
}

/// A station in the warehouse
#[derive(Clone, Debug)]
pub struct Station {
    pub id: StationId,
    pub string_id: String,
    pub node: NodeId,
    pub station_type: StationType,
    pub concurrency: u32,
    pub queue_capacity: Option<u32>,
    pub service_time: ServiceTimeModel,

    // Runtime state
    pub queue: VecDeque<RobotId>,
    pub serving: Vec<RobotId>,

    // Statistics
    pub total_served: u32,
    pub total_service_time: SimTime,
    pub total_queue_time: SimTime,
    pub max_queue_length: usize,
}

impl Station {
    pub fn new(
        id: StationId,
        string_id: String,
        node: NodeId,
        station_type: StationType,
        concurrency: u32,
        queue_capacity: Option<u32>,
        service_time: ServiceTimeModel,
    ) -> Self {
        Self {
            id,
            string_id,
            node,
            station_type,
            concurrency,
            queue_capacity,
            service_time,
            queue: VecDeque::new(),
            serving: Vec::new(),
            total_served: 0,
            total_service_time: SimTime::ZERO,
            total_queue_time: SimTime::ZERO,
            max_queue_length: 0,
        }
    }

    pub fn can_accept(&self) -> bool {
        match self.queue_capacity {
            Some(cap) => (self.queue.len() as u32) < cap,
            None => true,
        }
    }

    pub fn can_serve(&self) -> bool {
        (self.serving.len() as u32) < self.concurrency
    }

    pub fn enqueue(&mut self, robot: RobotId) {
        self.queue.push_back(robot);
        self.max_queue_length = self.max_queue_length.max(self.queue.len());
    }

    pub fn start_service(&mut self) -> Option<RobotId> {
        if self.can_serve() {
            if let Some(robot) = self.queue.pop_front() {
                self.serving.push(robot);
                return Some(robot);
            }
        }
        None
    }

    pub fn end_service(&mut self, robot: RobotId, service_time: SimTime) {
        if let Some(pos) = self.serving.iter().position(|&r| r == robot) {
            self.serving.remove(pos);
            self.total_served += 1;
            self.total_service_time += service_time;
        }
    }

    pub fn queue_length(&self) -> usize {
        self.queue.len()
    }

    pub fn serving_count(&self) -> usize {
        self.serving.len()
    }

    pub fn is_robot_in_queue(&self, robot: RobotId) -> bool {
        self.queue.contains(&robot)
    }

    pub fn is_robot_being_served(&self, robot: RobotId) -> bool {
        self.serving.contains(&robot)
    }

    pub fn utilization(&self, total_time: SimTime) -> f64 {
        if total_time.is_zero() || self.concurrency == 0 {
            return 0.0;
        }
        let capacity_seconds = total_time.as_seconds() * self.concurrency as f64;
        self.total_service_time.as_seconds() / capacity_seconds
    }
}
