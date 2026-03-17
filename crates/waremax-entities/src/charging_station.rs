//! Charging station entity for robot battery management

use waremax_core::{ChargingStationId, NodeId, RobotId, SimTime};
use std::collections::VecDeque;

/// A charging station in the warehouse
#[derive(Clone, Debug)]
pub struct ChargingStation {
    pub id: ChargingStationId,
    pub string_id: String,
    pub node: NodeId,

    /// Number of charging bays
    pub bays: u32,
    /// Charge rate per bay (Watts)
    pub charge_rate_w: f64,
    /// Optional queue capacity limit
    pub queue_capacity: Option<u32>,

    // Runtime state
    pub queue: VecDeque<RobotId>,
    /// Robots currently charging: (robot_id, charge_start_time)
    pub charging: Vec<(RobotId, SimTime)>,

    // Statistics
    pub total_robots_charged: u32,
    pub total_energy_delivered_wh: f64,
    pub total_charging_time: SimTime,
    pub max_queue_length: usize,
}

impl ChargingStation {
    pub fn new(
        id: ChargingStationId,
        string_id: String,
        node: NodeId,
        bays: u32,
        charge_rate_w: f64,
    ) -> Self {
        Self {
            id,
            string_id,
            node,
            bays,
            charge_rate_w,
            queue_capacity: None,
            queue: VecDeque::new(),
            charging: Vec::new(),
            total_robots_charged: 0,
            total_energy_delivered_wh: 0.0,
            total_charging_time: SimTime::ZERO,
            max_queue_length: 0,
        }
    }

    pub fn with_queue_capacity(mut self, capacity: u32) -> Self {
        self.queue_capacity = Some(capacity);
        self
    }

    /// Check if the station can accept another robot in queue
    pub fn can_accept(&self) -> bool {
        match self.queue_capacity {
            Some(limit) => (self.queue.len() as u32) < limit,
            None => true,
        }
    }

    /// Check if there's a free charging bay
    pub fn has_free_bay(&self) -> bool {
        (self.charging.len() as u32) < self.bays
    }

    /// Add a robot to the queue
    pub fn enqueue(&mut self, robot: RobotId) {
        self.queue.push_back(robot);
        self.max_queue_length = self.max_queue_length.max(self.queue.len());
    }

    /// Start charging a robot (must have free bay)
    pub fn start_charging(&mut self, robot: RobotId, start_time: SimTime) -> bool {
        if self.has_free_bay() {
            // Remove from queue if present
            self.queue.retain(|&r| r != robot);
            self.charging.push((robot, start_time));
            true
        } else {
            false
        }
    }

    /// End charging for a robot
    pub fn end_charging(&mut self, robot: RobotId, energy_wh: f64, duration: SimTime) {
        self.charging.retain(|(r, _)| *r != robot);
        self.total_robots_charged += 1;
        self.total_energy_delivered_wh += energy_wh;
        self.total_charging_time += duration;
    }

    /// Get the next robot in queue (if any) to start charging
    pub fn next_in_queue(&mut self) -> Option<RobotId> {
        if self.has_free_bay() {
            self.queue.pop_front()
        } else {
            None
        }
    }

    /// Calculate time to charge from current_soc to target_soc
    pub fn charging_duration(&self, current_soc: f64, target_soc: f64, capacity_wh: f64) -> SimTime {
        let energy_needed_wh = (target_soc - current_soc) * capacity_wh;
        let hours = energy_needed_wh / self.charge_rate_w;
        SimTime::from_seconds(hours * 3600.0)
    }

    /// Calculate energy delivered in a given duration
    pub fn energy_for_duration(&self, duration: SimTime) -> f64 {
        self.charge_rate_w * (duration.as_seconds() / 3600.0)
    }

    pub fn queue_length(&self) -> usize {
        self.queue.len()
    }

    pub fn charging_count(&self) -> usize {
        self.charging.len()
    }

    pub fn is_robot_in_queue(&self, robot: RobotId) -> bool {
        self.queue.contains(&robot)
    }

    pub fn is_robot_charging(&self, robot: RobotId) -> bool {
        self.charging.iter().any(|(r, _)| *r == robot)
    }

    pub fn utilization(&self, total_time: SimTime) -> f64 {
        if total_time.is_zero() || self.bays == 0 {
            return 0.0;
        }
        let capacity_seconds = total_time.as_seconds() * self.bays as f64;
        self.total_charging_time.as_seconds() / capacity_seconds
    }
}
