//! Order entity

use rkyv::{Archive, Deserialize, Serialize};
use waremax_core::{OrderId, SimTime, SkuId};

/// A line item in an order
#[derive(Archive, Deserialize, Serialize, Clone, Debug)]
pub struct OrderLine {
    pub sku_id: SkuId,
    pub quantity: u32,
}

impl OrderLine {
    pub fn new(sku_id: SkuId, quantity: u32) -> Self {
        Self { sku_id, quantity }
    }
}

/// Order status
#[derive(Archive, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum OrderStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl Default for OrderStatus {
    fn default() -> Self {
        OrderStatus::Pending
    }
}

/// An order in the system
#[derive(Clone, Debug)]
pub struct Order {
    pub id: OrderId,
    pub arrival_time: SimTime,
    pub due_time: Option<SimTime>,
    pub lines: Vec<OrderLine>,
    pub status: OrderStatus,
    pub completion_time: Option<SimTime>,
    pub tasks_total: u32,
    pub tasks_completed: u32,
}

impl Order {
    pub fn new(
        id: OrderId,
        arrival_time: SimTime,
        lines: Vec<OrderLine>,
        due_time: Option<SimTime>,
    ) -> Self {
        let tasks_total = lines.len() as u32;
        Self {
            id,
            arrival_time,
            due_time,
            lines,
            status: OrderStatus::Pending,
            completion_time: None,
            tasks_total,
            tasks_completed: 0,
        }
    }

    pub fn total_items(&self) -> u32 {
        self.lines.iter().map(|l| l.quantity).sum()
    }

    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn is_complete(&self) -> bool {
        self.status == OrderStatus::Completed
    }

    pub fn is_late(&self, current_time: SimTime) -> bool {
        self.due_time
            .map_or(false, |due| current_time > due && !self.is_complete())
    }

    pub fn cycle_time(&self) -> Option<SimTime> {
        self.completion_time.map(|c| c - self.arrival_time)
    }

    pub fn mark_task_complete(&mut self) {
        self.tasks_completed += 1;
    }

    pub fn all_tasks_complete(&self) -> bool {
        self.tasks_completed >= self.tasks_total
    }

    pub fn complete(&mut self, completion_time: SimTime) {
        self.status = OrderStatus::Completed;
        self.completion_time = Some(completion_time);
    }

    pub fn start(&mut self) {
        if self.status == OrderStatus::Pending {
            self.status = OrderStatus::InProgress;
        }
    }
}
