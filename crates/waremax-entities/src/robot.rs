//! Robot entity and state machine

use rkyv::{Archive, Deserialize, Serialize};
use std::collections::VecDeque;
use waremax_core::{
    ChargingStationId, EdgeId, MaintenanceStationId, NodeId, RobotId, SimTime, StationId, TaskId,
};

/// Robot state in the simulation
#[derive(Archive, Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
pub enum RobotState {
    /// Robot is idle and available for tasks
    #[default]
    Idle,
    /// Robot is moving to a destination
    Moving { destination: NodeId },
    /// Robot is waiting for an edge to become available
    Waiting { for_edge: EdgeId },
    /// Robot is being serviced at a station
    Servicing { at_station: StationId },
    /// Robot is picking up items at a bin
    PickingUp { at_node: NodeId },
    /// v1: Robot is charging at a charging station
    Charging { at_station: ChargingStationId },
    /// v1: Robot is moving to a charging station
    SeekingCharge { destination: ChargingStationId },
    /// v3: Robot has failed and needs repair
    Failed,
    /// v3: Robot is moving to a maintenance station
    SeekingMaintenance {
        destination: MaintenanceStationId,
        is_repair: bool,
    },
    /// v3: Robot is being serviced at a maintenance station
    InMaintenance {
        at_station: MaintenanceStationId,
        is_repair: bool,
    },
}

/// v1: Battery state for a robot
#[derive(Clone, Debug)]
pub struct BatteryState {
    /// Current state of charge (0.0 to 1.0, representing 0-100%)
    pub soc: f64,
    /// Maximum battery capacity in Watt-hours
    pub capacity_wh: f64,
    /// Minimum SOC threshold before robot seeks charging
    pub min_soc_threshold: f64,
    /// Critical SOC below which robot must stop operations
    pub critical_soc: f64,
    /// Whether battery management is enabled
    pub enabled: bool,
}

impl BatteryState {
    pub fn new(capacity_wh: f64, min_soc_threshold: f64) -> Self {
        Self {
            soc: 1.0, // Start fully charged
            capacity_wh,
            min_soc_threshold,
            critical_soc: 0.05,
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        Self {
            soc: 1.0,
            capacity_wh: 0.0,
            min_soc_threshold: 0.0,
            critical_soc: 0.0,
            enabled: false,
        }
    }

    pub fn needs_charging(&self) -> bool {
        self.enabled && self.soc <= self.min_soc_threshold
    }

    pub fn is_critical(&self) -> bool {
        self.enabled && self.soc <= self.critical_soc
    }

    pub fn current_energy_wh(&self) -> f64 {
        self.soc * self.capacity_wh
    }
}

impl Default for BatteryState {
    fn default() -> Self {
        Self::disabled()
    }
}

/// v1: Battery consumption model parameters
#[derive(Clone, Debug)]
pub struct BatteryConsumptionModel {
    /// Energy consumed per meter traveled (Wh/m)
    pub per_meter_wh: f64,
    /// Energy consumed per kg of payload per meter (Wh/kg/m)
    pub per_kg_per_meter_wh: f64,
    /// Base idle power consumption (W)
    pub idle_power_w: f64,
    /// Power during service operations (W)
    pub service_power_w: f64,
}

impl Default for BatteryConsumptionModel {
    fn default() -> Self {
        Self {
            per_meter_wh: 0.1,
            per_kg_per_meter_wh: 0.01,
            idle_power_w: 5.0,
            service_power_w: 20.0,
        }
    }
}

/// v3: Maintenance state for a robot
#[derive(Clone, Debug)]
pub struct MaintenanceState {
    /// Whether maintenance tracking is enabled
    pub enabled: bool,
    /// Operating time since last maintenance (seconds)
    pub operating_time_since_maintenance: f64,
    /// Maintenance interval threshold (seconds)
    pub maintenance_interval_s: f64,
    /// Whether robot is currently in a failed state
    pub is_failed: bool,
    /// Time when robot last failed
    pub last_failure_time: Option<SimTime>,
    /// Time when robot last completed maintenance
    pub last_maintenance_time: Option<SimTime>,
    /// Total number of failures
    pub failure_count: u32,
    /// Total number of maintenance events
    pub maintenance_count: u32,
}

impl MaintenanceState {
    pub fn new(maintenance_interval_s: f64) -> Self {
        Self {
            enabled: true,
            operating_time_since_maintenance: 0.0,
            maintenance_interval_s,
            is_failed: false,
            last_failure_time: None,
            last_maintenance_time: None,
            failure_count: 0,
            maintenance_count: 0,
        }
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            operating_time_since_maintenance: 0.0,
            maintenance_interval_s: 0.0,
            is_failed: false,
            last_failure_time: None,
            last_maintenance_time: None,
            failure_count: 0,
            maintenance_count: 0,
        }
    }

    /// Check if scheduled maintenance is due
    pub fn needs_maintenance(&self) -> bool {
        self.enabled && self.operating_time_since_maintenance >= self.maintenance_interval_s
    }

    /// Add operating time and return new total
    pub fn add_operating_time(&mut self, duration_s: f64) {
        if self.enabled {
            self.operating_time_since_maintenance += duration_s;
        }
    }

    /// Mark robot as failed
    pub fn mark_failed(&mut self, time: SimTime) {
        self.is_failed = true;
        self.last_failure_time = Some(time);
        self.failure_count += 1;
    }

    /// Complete maintenance (repair or scheduled)
    pub fn complete_maintenance(&mut self, time: SimTime) {
        self.is_failed = false;
        self.operating_time_since_maintenance = 0.0;
        self.last_maintenance_time = Some(time);
        self.maintenance_count += 1;
    }
}

impl Default for MaintenanceState {
    fn default() -> Self {
        Self::disabled()
    }
}

/// v3: Failure model parameters using exponential distribution
#[derive(Clone, Debug)]
pub struct FailureModel {
    /// Mean Time Between Failures in seconds
    pub mtbf_s: f64,
    /// Whether random failures are enabled
    pub enabled: bool,
}

impl FailureModel {
    pub fn new(mtbf_s: f64) -> Self {
        Self {
            mtbf_s,
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        Self {
            mtbf_s: 0.0,
            enabled: false,
        }
    }

    /// Calculate probability of failure over a time duration
    /// Uses exponential distribution: P(fail) = 1 - exp(-duration / MTBF)
    pub fn failure_probability(&self, duration_s: f64) -> f64 {
        if !self.enabled || self.mtbf_s <= 0.0 {
            return 0.0;
        }
        1.0 - (-duration_s / self.mtbf_s).exp()
    }

    /// Check if robot should fail given a random value [0, 1)
    pub fn should_fail(&self, duration_s: f64, random_value: f64) -> bool {
        self.enabled && random_value < self.failure_probability(duration_s)
    }
}

impl Default for FailureModel {
    fn default() -> Self {
        Self::disabled()
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

    // v1: Battery state
    pub battery: BatteryState,
    pub consumption_model: BatteryConsumptionModel,
    pub current_payload_kg: f64,
    pub seeking_charging: bool,
    pub assigned_charging_station: Option<ChargingStationId>,

    // v3: Maintenance and failure state
    pub maintenance: MaintenanceState,
    pub failure_model: FailureModel,
    pub seeking_maintenance: bool,
    pub assigned_maintenance_station: Option<MaintenanceStationId>,

    // Statistics
    pub total_distance: f64,
    pub total_idle_time: SimTime,
    pub total_wait_time: SimTime,
    pub total_service_time: SimTime,
    pub total_move_time: SimTime,
    pub total_charging_time: SimTime,
    pub last_state_change: SimTime,
    pub tasks_completed: u32,
    pub total_energy_consumed_wh: f64,
    pub charging_events: u32,

    // v3: Maintenance statistics
    pub total_maintenance_time: SimTime,
    pub total_failed_time: SimTime,
    pub tasks_interrupted_by_failure: u32,
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
            battery: BatteryState::default(),
            consumption_model: BatteryConsumptionModel::default(),
            current_payload_kg: 0.0,
            seeking_charging: false,
            assigned_charging_station: None,
            maintenance: MaintenanceState::default(),
            failure_model: FailureModel::default(),
            seeking_maintenance: false,
            assigned_maintenance_station: None,
            total_distance: 0.0,
            total_idle_time: SimTime::ZERO,
            total_wait_time: SimTime::ZERO,
            total_service_time: SimTime::ZERO,
            total_move_time: SimTime::ZERO,
            total_charging_time: SimTime::ZERO,
            last_state_change: SimTime::ZERO,
            tasks_completed: 0,
            total_energy_consumed_wh: 0.0,
            charging_events: 0,
            total_maintenance_time: SimTime::ZERO,
            total_failed_time: SimTime::ZERO,
            tasks_interrupted_by_failure: 0,
        }
    }

    /// Create a robot with battery enabled
    pub fn with_battery(
        id: RobotId,
        start_node: NodeId,
        max_speed_mps: f64,
        max_payload_kg: f64,
        battery_capacity_wh: f64,
        min_soc_threshold: f64,
        consumption_model: BatteryConsumptionModel,
    ) -> Self {
        let mut robot = Self::new(id, start_node, max_speed_mps, max_payload_kg);
        robot.battery = BatteryState::new(battery_capacity_wh, min_soc_threshold);
        robot.consumption_model = consumption_model;
        robot
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.state, RobotState::Idle)
    }

    pub fn is_available(&self) -> bool {
        self.is_idle()
            && self.current_task.is_none()
            && !self.maintenance.is_failed
            && !self.maintenance.needs_maintenance()
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
            RobotState::Moving { .. }
            | RobotState::SeekingCharge { .. }
            | RobotState::SeekingMaintenance { .. } => self.total_move_time += duration,
            RobotState::Waiting { .. } => self.total_wait_time += duration,
            RobotState::Servicing { .. } | RobotState::PickingUp { .. } => {
                self.total_service_time += duration
            }
            RobotState::Charging { .. } => self.total_charging_time += duration,
            RobotState::Failed => self.total_failed_time += duration,
            RobotState::InMaintenance { .. } => self.total_maintenance_time += duration,
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

    // === v1: Battery methods ===

    /// Calculate energy consumed for traveling a distance with current payload
    pub fn energy_for_distance(&self, distance_m: f64) -> f64 {
        let base = self.consumption_model.per_meter_wh * distance_m;
        let payload =
            self.consumption_model.per_kg_per_meter_wh * self.current_payload_kg * distance_m;
        base + payload
    }

    /// Consume battery for travel
    pub fn consume_travel_energy(&mut self, distance_m: f64) {
        if !self.battery.enabled {
            return;
        }
        let energy = self.energy_for_distance(distance_m);
        let soc_delta = energy / self.battery.capacity_wh;
        self.battery.soc = (self.battery.soc - soc_delta).max(0.0);
        self.total_energy_consumed_wh += energy;
    }

    /// Consume battery for time-based operations (idle, service)
    pub fn consume_time_energy(&mut self, duration: SimTime, power_w: f64) {
        if !self.battery.enabled {
            return;
        }
        let energy = power_w * (duration.as_seconds() / 3600.0);
        let soc_delta = energy / self.battery.capacity_wh;
        self.battery.soc = (self.battery.soc - soc_delta).max(0.0);
        self.total_energy_consumed_wh += energy;
    }

    /// Charge battery
    pub fn charge(&mut self, energy_wh: f64, duration: SimTime) {
        if !self.battery.enabled {
            return;
        }
        let soc_delta = energy_wh / self.battery.capacity_wh;
        self.battery.soc = (self.battery.soc + soc_delta).min(1.0);
        self.charging_events += 1;
        self.total_charging_time += duration;
    }

    /// Check if battery needs charging
    pub fn needs_charging(&self) -> bool {
        self.battery.needs_charging()
    }

    /// Check if battery is at critical level
    pub fn is_battery_critical(&self) -> bool {
        self.battery.is_critical()
    }

    /// Check if battery is enabled
    pub fn has_battery(&self) -> bool {
        self.battery.enabled
    }

    /// Get current state of charge
    pub fn soc(&self) -> f64 {
        self.battery.soc
    }

    // === v3: Maintenance methods ===

    /// Enable maintenance tracking with given interval
    pub fn enable_maintenance(&mut self, interval_s: f64) {
        self.maintenance = MaintenanceState::new(interval_s);
    }

    /// Enable random failures with given MTBF
    pub fn enable_failures(&mut self, mtbf_s: f64) {
        self.failure_model = FailureModel::new(mtbf_s);
    }

    /// Check if robot needs scheduled maintenance
    pub fn needs_maintenance(&self) -> bool {
        self.maintenance.needs_maintenance()
    }

    /// Check if robot is currently failed
    pub fn is_failed(&self) -> bool {
        self.maintenance.is_failed
    }

    /// Check if robot has maintenance enabled
    pub fn has_maintenance(&self) -> bool {
        self.maintenance.enabled
    }

    /// Check if robot has failure model enabled
    pub fn has_failure_model(&self) -> bool {
        self.failure_model.enabled
    }

    /// Add operating time for maintenance tracking
    pub fn add_operating_time(&mut self, duration_s: f64) {
        self.maintenance.add_operating_time(duration_s);
    }

    /// Check if robot should fail given a duration and random value
    pub fn should_fail(&self, duration_s: f64, random_value: f64) -> bool {
        self.failure_model.should_fail(duration_s, random_value)
    }

    /// Mark robot as failed
    pub fn mark_failed(&mut self, time: SimTime) {
        self.maintenance.mark_failed(time);
        if self.current_task.is_some() {
            self.tasks_interrupted_by_failure += 1;
        }
    }

    /// Complete maintenance or repair
    pub fn complete_maintenance(&mut self, time: SimTime) {
        self.maintenance.complete_maintenance(time);
        self.seeking_maintenance = false;
        self.assigned_maintenance_station = None;
    }

    /// Get availability percentage (time not failed or in maintenance)
    pub fn availability(&self, total_time: SimTime) -> f64 {
        if total_time.is_zero() {
            return 1.0;
        }
        let unavailable_time = self.total_failed_time + self.total_maintenance_time;
        1.0 - (unavailable_time.as_seconds() / total_time.as_seconds())
    }
}
