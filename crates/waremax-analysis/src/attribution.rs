//! Delay Attribution Collector
//!
//! Tracks time breakdown per task by category to understand where time is spent.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use waremax_core::{SimTime, TaskId, OrderId, RobotId, StationId, NodeId, EdgeId};

/// Categories of delay/time spent during task execution
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DelayCategory {
    /// Waiting for a robot to be assigned
    RobotAssignment,
    /// Travel time to pickup location
    TravelToPickup,
    /// Travel time to station/destination
    TravelToStation,
    /// Waiting due to traffic congestion
    CongestionWait,
    /// Waiting in queue at station
    StationQueue,
    /// Actual service time at station
    StationService,
    /// Detour for charging
    ChargingDetour,
    /// Detour for scheduled maintenance
    MaintenanceDetour,
    /// Recovery from failure
    FailureRecovery,
}

impl DelayCategory {
    /// Get all delay categories
    pub fn all() -> Vec<DelayCategory> {
        vec![
            DelayCategory::RobotAssignment,
            DelayCategory::TravelToPickup,
            DelayCategory::TravelToStation,
            DelayCategory::CongestionWait,
            DelayCategory::StationQueue,
            DelayCategory::StationService,
            DelayCategory::ChargingDetour,
            DelayCategory::MaintenanceDetour,
            DelayCategory::FailureRecovery,
        ]
    }

    /// Get a human-readable name for the category
    pub fn name(&self) -> &'static str {
        match self {
            DelayCategory::RobotAssignment => "Robot Assignment Wait",
            DelayCategory::TravelToPickup => "Travel to Pickup",
            DelayCategory::TravelToStation => "Travel to Station",
            DelayCategory::CongestionWait => "Congestion Wait",
            DelayCategory::StationQueue => "Station Queue Wait",
            DelayCategory::StationService => "Station Service",
            DelayCategory::ChargingDetour => "Charging Detour",
            DelayCategory::MaintenanceDetour => "Maintenance Detour",
            DelayCategory::FailureRecovery => "Failure Recovery",
        }
    }

    /// Returns true if this category represents "waste" time (not productive)
    pub fn is_waste(&self) -> bool {
        matches!(
            self,
            DelayCategory::RobotAssignment
                | DelayCategory::CongestionWait
                | DelayCategory::StationQueue
                | DelayCategory::ChargingDetour
                | DelayCategory::MaintenanceDetour
                | DelayCategory::FailureRecovery
        )
    }
}

/// A recorded congestion event during task execution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CongestionEvent {
    /// When the wait started (seconds)
    pub start_time_s: f64,
    /// Duration of the wait
    pub duration_s: f64,
    /// Node where the wait occurred
    pub at_node: Option<NodeId>,
    /// Edge that was blocked
    pub blocked_edge: Option<EdgeId>,
    /// Robots that were blocking
    pub blocked_by: Vec<RobotId>,
}

/// A recorded queue wait event at a station
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueueWaitEvent {
    /// Station where the wait occurred
    pub station_id: StationId,
    /// When the wait started (seconds)
    pub start_time_s: f64,
    /// Duration of the wait
    pub duration_s: f64,
    /// Queue position when arriving
    pub queue_position: usize,
}

/// Attribution data for a single task
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskAttribution {
    /// The task being tracked
    pub task_id: TaskId,
    /// Associated order (if any)
    pub order_id: Option<OrderId>,
    /// Robot that executed the task (if assigned)
    pub robot_id: Option<RobotId>,
    /// Time breakdown by category (in seconds)
    pub time_breakdown: HashMap<DelayCategory, f64>,
    /// Individual congestion events
    pub congestion_events: Vec<CongestionEvent>,
    /// Individual queue wait events
    pub queue_waits: Vec<QueueWaitEvent>,
    /// When the task was created (seconds)
    pub created_at_s: f64,
    /// When the task was completed (seconds, if completed)
    pub completed_at_s: Option<f64>,
    /// Current tracking state
    tracking_state: TaskTrackingState,
}

/// Internal state for tracking in-progress timing
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct TaskTrackingState {
    /// When current phase started (seconds)
    current_phase_start_s: Option<f64>,
    /// What category is currently being tracked
    current_category: Option<DelayCategory>,
}

impl TaskAttribution {
    /// Create a new task attribution tracker
    pub fn new(task_id: TaskId, order_id: Option<OrderId>, created_at: SimTime) -> Self {
        Self {
            task_id,
            order_id,
            robot_id: None,
            time_breakdown: HashMap::new(),
            congestion_events: Vec::new(),
            queue_waits: Vec::new(),
            created_at_s: created_at.as_seconds(),
            completed_at_s: None,
            tracking_state: TaskTrackingState::default(),
        }
    }

    /// Start tracking a new phase
    pub fn start_phase(&mut self, category: DelayCategory, at_time: SimTime) {
        // End any previous phase
        self.end_current_phase(at_time);

        self.tracking_state.current_phase_start_s = Some(at_time.as_seconds());
        self.tracking_state.current_category = Some(category);
    }

    /// End the current phase and record its duration
    pub fn end_current_phase(&mut self, at_time: SimTime) {
        if let (Some(start_s), Some(category)) = (
            self.tracking_state.current_phase_start_s,
            self.tracking_state.current_category.take(),
        ) {
            let duration = at_time.as_seconds() - start_s;
            if duration > 0.0 {
                *self.time_breakdown.entry(category).or_insert(0.0) += duration;
            }
        }
        self.tracking_state.current_phase_start_s = None;
    }

    /// Record time for a specific category (accumulative)
    pub fn record_time(&mut self, category: DelayCategory, duration_s: f64) {
        if duration_s > 0.0 {
            *self.time_breakdown.entry(category).or_insert(0.0) += duration_s;
        }
    }

    /// Record a congestion event
    pub fn record_congestion(&mut self, event: CongestionEvent) {
        self.congestion_events.push(event);
    }

    /// Record a queue wait event
    pub fn record_queue_wait(&mut self, event: QueueWaitEvent) {
        self.queue_waits.push(event);
    }

    /// Mark robot assignment
    pub fn assign_robot(&mut self, robot_id: RobotId, at_time: SimTime) {
        self.robot_id = Some(robot_id);
        // Record assignment wait time
        let wait_time = at_time.as_seconds() - self.created_at_s;
        if wait_time > 0.0 {
            *self.time_breakdown
                .entry(DelayCategory::RobotAssignment)
                .or_insert(0.0) += wait_time;
        }
    }

    /// Mark task as completed
    pub fn complete(&mut self, at_time: SimTime) {
        self.end_current_phase(at_time);
        self.completed_at_s = Some(at_time.as_seconds());
    }

    /// Get total time spent in a category
    pub fn time_in_category(&self, category: &DelayCategory) -> f64 {
        *self.time_breakdown.get(category).unwrap_or(&0.0)
    }

    /// Get total cycle time (if completed)
    pub fn cycle_time(&self) -> Option<f64> {
        self.completed_at_s.map(|end_s| end_s - self.created_at_s)
    }

    /// Get total time across all categories
    pub fn total_tracked_time(&self) -> f64 {
        self.time_breakdown.values().sum()
    }

    /// Get waste time (non-productive time)
    pub fn waste_time(&self) -> f64 {
        self.time_breakdown
            .iter()
            .filter(|(cat, _)| cat.is_waste())
            .map(|(_, time)| time)
            .sum()
    }

    /// Get percentage breakdown
    pub fn percentage_breakdown(&self) -> HashMap<DelayCategory, f64> {
        let total = self.total_tracked_time();
        if total <= 0.0 {
            return HashMap::new();
        }
        self.time_breakdown
            .iter()
            .map(|(cat, time)| (cat.clone(), (time / total) * 100.0))
            .collect()
    }
}

/// Aggregated delay attribution summary across multiple tasks
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DelayAttributionSummary {
    /// Number of tasks analyzed
    pub task_count: usize,
    /// Average time by category (seconds)
    pub avg_by_category: HashMap<DelayCategory, f64>,
    /// Total time by category (seconds)
    pub total_by_category: HashMap<DelayCategory, f64>,
    /// Percentage by category
    pub pct_by_category: HashMap<DelayCategory, f64>,
    /// Categories ranked by total time (descending)
    pub ranked_categories: Vec<(DelayCategory, f64, f64)>, // (category, total_s, pct)
    /// Average waste time per task
    pub avg_waste_time_s: f64,
    /// Average cycle time per task
    pub avg_cycle_time_s: f64,
    /// Waste ratio (waste_time / cycle_time)
    pub waste_ratio: f64,
}

impl Default for DelayAttributionSummary {
    fn default() -> Self {
        Self {
            task_count: 0,
            avg_by_category: HashMap::new(),
            total_by_category: HashMap::new(),
            pct_by_category: HashMap::new(),
            ranked_categories: Vec::new(),
            avg_waste_time_s: 0.0,
            avg_cycle_time_s: 0.0,
            waste_ratio: 0.0,
        }
    }
}

impl DelayAttributionSummary {
    /// Create a summary from a collection of task attributions
    pub fn from_attributions(attributions: &[TaskAttribution]) -> Self {
        if attributions.is_empty() {
            return Self::default();
        }

        let task_count = attributions.len();
        let mut total_by_category: HashMap<DelayCategory, f64> = HashMap::new();
        let mut total_waste = 0.0;
        let mut total_cycle = 0.0;
        let mut completed_count = 0;

        // Aggregate totals
        for attr in attributions {
            for (cat, time) in &attr.time_breakdown {
                *total_by_category.entry(cat.clone()).or_insert(0.0) += time;
            }
            total_waste += attr.waste_time();
            if let Some(cycle) = attr.cycle_time() {
                total_cycle += cycle;
                completed_count += 1;
            }
        }

        // Calculate averages
        let avg_by_category: HashMap<DelayCategory, f64> = total_by_category
            .iter()
            .map(|(cat, total)| (cat.clone(), total / task_count as f64))
            .collect();

        // Calculate percentages
        let grand_total: f64 = total_by_category.values().sum();
        let pct_by_category: HashMap<DelayCategory, f64> = if grand_total > 0.0 {
            total_by_category
                .iter()
                .map(|(cat, total)| (cat.clone(), (total / grand_total) * 100.0))
                .collect()
        } else {
            HashMap::new()
        };

        // Rank categories
        let mut ranked_categories: Vec<(DelayCategory, f64, f64)> = total_by_category
            .iter()
            .map(|(cat, total)| {
                let pct = if grand_total > 0.0 {
                    (total / grand_total) * 100.0
                } else {
                    0.0
                };
                (cat.clone(), *total, pct)
            })
            .collect();
        ranked_categories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let avg_waste_time_s = total_waste / task_count as f64;
        let avg_cycle_time_s = if completed_count > 0 {
            total_cycle / completed_count as f64
        } else {
            0.0
        };
        let waste_ratio = if avg_cycle_time_s > 0.0 {
            avg_waste_time_s / avg_cycle_time_s
        } else {
            0.0
        };

        Self {
            task_count,
            avg_by_category,
            total_by_category,
            pct_by_category,
            ranked_categories,
            avg_waste_time_s,
            avg_cycle_time_s,
            waste_ratio,
        }
    }

    /// Format the summary as a human-readable string
    pub fn to_string(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Delay Attribution Summary ({} tasks)\n", self.task_count));
        output.push_str(&"=".repeat(50));
        output.push('\n');

        for (i, (cat, total_s, pct)) in self.ranked_categories.iter().enumerate() {
            let avg = self.avg_by_category.get(cat).unwrap_or(&0.0);
            output.push_str(&format!(
                "{}. {}: {:.1}% ({:.1}s total, {:.1}s avg)\n",
                i + 1,
                cat.name(),
                pct,
                total_s,
                avg
            ));
        }

        output.push_str(&"-".repeat(50));
        output.push('\n');
        output.push_str(&format!("Average Cycle Time: {:.1}s\n", self.avg_cycle_time_s));
        output.push_str(&format!("Average Waste Time: {:.1}s ({:.1}%)\n",
            self.avg_waste_time_s,
            self.waste_ratio * 100.0
        ));

        output
    }
}

/// Collector for task attributions during simulation
#[derive(Clone, Debug, Default)]
pub struct AttributionCollector {
    /// Active task attributions (in-progress tasks)
    active: HashMap<TaskId, TaskAttribution>,
    /// Completed task attributions
    completed: Vec<TaskAttribution>,
    /// Whether collection is enabled
    pub enabled: bool,
}

impl AttributionCollector {
    /// Create a new attribution collector
    pub fn new() -> Self {
        Self {
            active: HashMap::new(),
            completed: Vec::new(),
            enabled: false,
        }
    }

    /// Create an enabled attribution collector
    pub fn enabled() -> Self {
        Self {
            active: HashMap::new(),
            completed: Vec::new(),
            enabled: true,
        }
    }

    /// Enable the attribution collector
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable the attribution collector
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if collection is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Start tracking a new task
    pub fn start_task(&mut self, task_id: TaskId, order_id: Option<OrderId>, created_at: SimTime) {
        if !self.enabled {
            return;
        }
        let attribution = TaskAttribution::new(task_id, order_id, created_at);
        self.active.insert(task_id, attribution);
    }

    /// Get mutable reference to active task attribution
    pub fn get_mut(&mut self, task_id: TaskId) -> Option<&mut TaskAttribution> {
        if !self.enabled {
            return None;
        }
        self.active.get_mut(&task_id)
    }

    /// Get reference to active task attribution
    pub fn get(&self, task_id: TaskId) -> Option<&TaskAttribution> {
        self.active.get(&task_id)
    }

    /// Record robot assignment for a task
    pub fn record_robot_assignment(&mut self, task_id: TaskId, robot_id: RobotId, at_time: SimTime) {
        if let Some(attr) = self.active.get_mut(&task_id) {
            attr.assign_robot(robot_id, at_time);
        }
    }

    /// Start a new phase for a task
    pub fn start_phase(&mut self, task_id: TaskId, category: DelayCategory, at_time: SimTime) {
        if let Some(attr) = self.active.get_mut(&task_id) {
            attr.start_phase(category, at_time);
        }
    }

    /// Record time for a task
    pub fn record_time(&mut self, task_id: TaskId, category: DelayCategory, duration_s: f64) {
        if let Some(attr) = self.active.get_mut(&task_id) {
            attr.record_time(category, duration_s);
        }
    }

    /// Record a congestion event
    pub fn record_congestion(&mut self, task_id: TaskId, event: CongestionEvent) {
        if let Some(attr) = self.active.get_mut(&task_id) {
            attr.record_congestion(event.clone());
            attr.record_time(DelayCategory::CongestionWait, event.duration_s);
        }
    }

    /// Record a queue wait event
    pub fn record_queue_wait(&mut self, task_id: TaskId, event: QueueWaitEvent) {
        if let Some(attr) = self.active.get_mut(&task_id) {
            attr.record_queue_wait(event.clone());
            attr.record_time(DelayCategory::StationQueue, event.duration_s);
        }
    }

    /// Complete a task and move it to completed list
    pub fn complete_task(&mut self, task_id: TaskId, at_time: SimTime) {
        if let Some(mut attr) = self.active.remove(&task_id) {
            attr.complete(at_time);
            self.completed.push(attr);
        }
    }

    /// Get all completed task attributions
    pub fn completed_attributions(&self) -> &[TaskAttribution] {
        &self.completed
    }

    /// Get active task attributions
    pub fn active_attributions(&self) -> impl Iterator<Item = &TaskAttribution> {
        self.active.values()
    }

    /// Get summary of all completed attributions
    pub fn summary(&self) -> DelayAttributionSummary {
        DelayAttributionSummary::from_attributions(&self.completed)
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.active.clear();
        self.completed.clear();
    }

    /// Get total number of tracked tasks
    pub fn total_tasks(&self) -> usize {
        self.active.len() + self.completed.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_category_properties() {
        assert!(DelayCategory::RobotAssignment.is_waste());
        assert!(DelayCategory::CongestionWait.is_waste());
        assert!(!DelayCategory::TravelToPickup.is_waste());
        assert!(!DelayCategory::StationService.is_waste());
    }

    #[test]
    fn test_task_attribution_time_tracking() {
        let mut attr = TaskAttribution::new(
            TaskId(1),
            Some(OrderId(100)),
            SimTime::from_seconds(0.0),
        );

        // Simulate assignment after 5 seconds
        attr.assign_robot(RobotId(1), SimTime::from_seconds(5.0));
        assert_eq!(attr.time_in_category(&DelayCategory::RobotAssignment), 5.0);

        // Record travel time
        attr.record_time(DelayCategory::TravelToPickup, 10.0);
        assert_eq!(attr.time_in_category(&DelayCategory::TravelToPickup), 10.0);

        // Record service time
        attr.record_time(DelayCategory::StationService, 8.0);

        // Complete
        attr.complete(SimTime::from_seconds(30.0));

        assert_eq!(attr.cycle_time(), Some(30.0));
        assert_eq!(attr.total_tracked_time(), 23.0); // 5 + 10 + 8
        assert_eq!(attr.waste_time(), 5.0); // Only assignment wait
    }

    #[test]
    fn test_attribution_collector() {
        let mut collector = AttributionCollector::enabled();

        // Start tracking a task
        collector.start_task(TaskId(1), Some(OrderId(100)), SimTime::from_seconds(0.0));

        // Record assignment
        collector.record_robot_assignment(TaskId(1), RobotId(1), SimTime::from_seconds(3.0));

        // Record travel
        collector.record_time(TaskId(1), DelayCategory::TravelToPickup, 15.0);

        // Complete
        collector.complete_task(TaskId(1), SimTime::from_seconds(25.0));

        assert_eq!(collector.completed_attributions().len(), 1);
        let summary = collector.summary();
        assert_eq!(summary.task_count, 1);
    }

    #[test]
    fn test_phase_tracking() {
        let mut attr = TaskAttribution::new(
            TaskId(1),
            None,
            SimTime::from_seconds(0.0),
        );

        // Start travel phase at t=5
        attr.start_phase(DelayCategory::TravelToPickup, SimTime::from_seconds(5.0));

        // End travel (start queue) at t=20
        attr.start_phase(DelayCategory::StationQueue, SimTime::from_seconds(20.0));

        // End queue (start service) at t=25
        attr.start_phase(DelayCategory::StationService, SimTime::from_seconds(25.0));

        // Complete at t=35
        attr.complete(SimTime::from_seconds(35.0));

        assert_eq!(attr.time_in_category(&DelayCategory::TravelToPickup), 15.0);
        assert_eq!(attr.time_in_category(&DelayCategory::StationQueue), 5.0);
        assert_eq!(attr.time_in_category(&DelayCategory::StationService), 10.0);
    }
}
