//! Maintenance station entity for robot maintenance and repair

use crate::station::ServiceTimeModel;
use std::collections::VecDeque;
use waremax_core::{MaintenanceStationId, NodeId, RobotId, SimRng, SimTime};

/// A maintenance station in the warehouse for robot servicing and repair
#[derive(Clone, Debug)]
pub struct MaintenanceStation {
    pub id: MaintenanceStationId,
    pub string_id: String,
    pub node: NodeId,

    /// Number of maintenance bays
    pub bays: u32,
    /// Duration for scheduled maintenance (seconds)
    pub maintenance_duration_s: f64,
    /// Service time model for repairs (variable duration)
    pub repair_time_model: ServiceTimeModel,
    /// Optional queue capacity limit
    pub queue_capacity: Option<u32>,

    // Runtime state
    pub queue: VecDeque<RobotId>,
    /// Robots currently being serviced: (robot_id, start_time, is_repair)
    pub servicing: Vec<(RobotId, SimTime, bool)>,

    // Statistics
    pub total_maintenance_events: u32,
    pub total_repair_events: u32,
    pub total_maintenance_time: SimTime,
    pub total_repair_time: SimTime,
    pub max_queue_length: usize,
}

impl MaintenanceStation {
    pub fn new(
        id: MaintenanceStationId,
        string_id: String,
        node: NodeId,
        bays: u32,
        maintenance_duration_s: f64,
    ) -> Self {
        Self {
            id,
            string_id,
            node,
            bays,
            maintenance_duration_s,
            repair_time_model: ServiceTimeModel::default(),
            queue_capacity: None,
            queue: VecDeque::new(),
            servicing: Vec::new(),
            total_maintenance_events: 0,
            total_repair_events: 0,
            total_maintenance_time: SimTime::ZERO,
            total_repair_time: SimTime::ZERO,
            max_queue_length: 0,
        }
    }

    pub fn with_repair_time_model(mut self, model: ServiceTimeModel) -> Self {
        self.repair_time_model = model;
        self
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

    /// Check if there's a free maintenance bay
    pub fn has_free_bay(&self) -> bool {
        (self.servicing.len() as u32) < self.bays
    }

    /// Add a robot to the queue
    pub fn enqueue(&mut self, robot: RobotId) {
        self.queue.push_back(robot);
        self.max_queue_length = self.max_queue_length.max(self.queue.len());
    }

    /// Start maintenance or repair for a robot (must have free bay)
    /// Returns the service duration if started
    pub fn start_service(
        &mut self,
        robot: RobotId,
        start_time: SimTime,
        is_repair: bool,
        rng: &mut SimRng,
    ) -> Option<SimTime> {
        if self.has_free_bay() {
            // Remove from queue if present
            self.queue.retain(|&r| r != robot);
            self.servicing.push((robot, start_time, is_repair));

            let duration = if is_repair {
                // Use repair time model for repairs (item_count=1 for simplicity)
                self.repair_time_model.sample(rng, 1)
            } else {
                // Fixed duration for scheduled maintenance
                SimTime::from_seconds(self.maintenance_duration_s)
            };

            Some(duration)
        } else {
            None
        }
    }

    /// End service for a robot
    pub fn end_service(&mut self, robot: RobotId, duration: SimTime, is_repair: bool) {
        self.servicing.retain(|(r, _, _)| *r != robot);
        if is_repair {
            self.total_repair_events += 1;
            self.total_repair_time += duration;
        } else {
            self.total_maintenance_events += 1;
            self.total_maintenance_time += duration;
        }
    }

    /// Get the next robot in queue (if any) to start servicing
    pub fn next_in_queue(&mut self) -> Option<RobotId> {
        if self.has_free_bay() {
            self.queue.pop_front()
        } else {
            None
        }
    }

    pub fn queue_length(&self) -> usize {
        self.queue.len()
    }

    pub fn servicing_count(&self) -> usize {
        self.servicing.len()
    }

    pub fn is_robot_in_queue(&self, robot: RobotId) -> bool {
        self.queue.contains(&robot)
    }

    pub fn is_robot_servicing(&self, robot: RobotId) -> bool {
        self.servicing.iter().any(|(r, _, _)| *r == robot)
    }

    pub fn utilization(&self, total_time: SimTime) -> f64 {
        if total_time.is_zero() || self.bays == 0 {
            return 0.0;
        }
        let capacity_seconds = total_time.as_seconds() * self.bays as f64;
        let used_seconds = (self.total_maintenance_time + self.total_repair_time).as_seconds();
        used_seconds / capacity_seconds
    }

    /// Get mean time to repair (MTTR) in seconds
    pub fn mttr_s(&self) -> f64 {
        if self.total_repair_events == 0 {
            0.0
        } else {
            self.total_repair_time.as_seconds() / self.total_repair_events as f64
        }
    }
}
