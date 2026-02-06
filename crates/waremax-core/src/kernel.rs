//! Discrete Event Simulation (DES) kernel
//!
//! The kernel maintains a priority queue of events ordered by time,
//! and advances simulation time by processing events in order.

use crate::{EventId, IdGenerator, ScheduledEvent, SimEvent, SimTime};
use std::collections::BinaryHeap;

/// The discrete-event simulation kernel
#[derive(Debug)]
pub struct Kernel {
    /// Priority queue of scheduled events (min-heap by time)
    event_queue: BinaryHeap<ScheduledEvent>,
    /// Current simulation time
    current_time: SimTime,
    /// Event ID generator
    event_id_gen: IdGenerator<EventId>,
    /// Total events processed
    events_processed: u64,
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}

impl Kernel {
    /// Create a new kernel starting at time 0
    pub fn new() -> Self {
        Self {
            event_queue: BinaryHeap::new(),
            current_time: SimTime::ZERO,
            event_id_gen: IdGenerator::new(),
            events_processed: 0,
        }
    }

    /// Get current simulation time
    #[inline]
    pub fn now(&self) -> SimTime {
        self.current_time
    }

    /// Schedule an event at a specific absolute time
    ///
    /// # Panics
    /// Panics if the event time is in the past (before current time)
    pub fn schedule_at(&mut self, time: SimTime, event: SimEvent) -> EventId {
        debug_assert!(
            time >= self.current_time,
            "Cannot schedule event in the past: {:?} < {:?}",
            time,
            self.current_time
        );

        let id = self.event_id_gen.next_id();
        self.event_queue.push(ScheduledEvent::new(id, time, event));
        id
    }

    /// Schedule an event after a delay from current time
    pub fn schedule_after(&mut self, delay: SimTime, event: SimEvent) -> EventId {
        self.schedule_at(self.current_time + delay, event)
    }

    /// Schedule an event immediately (at current time)
    pub fn schedule_now(&mut self, event: SimEvent) -> EventId {
        self.schedule_at(self.current_time, event)
    }

    /// Pop the next event from the queue, advancing simulation time
    pub fn pop_next(&mut self) -> Option<ScheduledEvent> {
        if let Some(event) = self.event_queue.pop() {
            self.current_time = event.time;
            self.events_processed += 1;
            Some(event)
        } else {
            None
        }
    }

    /// Peek at the next event without removing it
    pub fn peek_next(&self) -> Option<&ScheduledEvent> {
        self.event_queue.peek()
    }

    /// Get the time of the next scheduled event, if any
    pub fn next_event_time(&self) -> Option<SimTime> {
        self.event_queue.peek().map(|e| e.time)
    }

    /// Check if any events are pending
    #[inline]
    pub fn has_events(&self) -> bool {
        !self.event_queue.is_empty()
    }

    /// Get number of pending events
    #[inline]
    pub fn pending_count(&self) -> usize {
        self.event_queue.len()
    }

    /// Get total events processed
    #[inline]
    pub fn events_processed(&self) -> u64 {
        self.events_processed
    }

    /// Cancel an event by ID
    ///
    /// Returns true if the event was found and removed.
    /// Note: This operation is O(n) as it requires rebuilding the heap.
    pub fn cancel(&mut self, id: EventId) -> bool {
        let original_len = self.event_queue.len();
        let events: Vec<_> = self.event_queue.drain().filter(|e| e.id != id).collect();
        let found = events.len() < original_len;
        self.event_queue = events.into_iter().collect();
        found
    }

    /// Clear all pending events
    pub fn clear(&mut self) {
        self.event_queue.clear();
    }

    /// Reset the kernel to initial state
    pub fn reset(&mut self) {
        self.event_queue.clear();
        self.current_time = SimTime::ZERO;
        self.event_id_gen = IdGenerator::new();
        self.events_processed = 0;
    }

    /// Advance time to a specific point, processing all events up to that time
    ///
    /// Returns the number of events processed.
    pub fn advance_to<F>(&mut self, target_time: SimTime, mut handler: F) -> u64
    where
        F: FnMut(&mut Self, ScheduledEvent),
    {
        let mut count = 0;
        while let Some(event) = self.peek_next() {
            if event.time > target_time {
                break;
            }
            if let Some(event) = self.pop_next() {
                handler(self, event);
                count += 1;
            }
        }
        // Advance time even if no events
        if self.current_time < target_time {
            self.current_time = target_time;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OrderId;

    #[test]
    fn test_kernel_basic() {
        let mut kernel = Kernel::new();
        assert_eq!(kernel.now(), SimTime::ZERO);
        assert!(!kernel.has_events());
    }

    #[test]
    fn test_event_scheduling() {
        let mut kernel = Kernel::new();

        kernel.schedule_at(SimTime::from_seconds(10.0), SimEvent::DispatchTasks);
        kernel.schedule_at(SimTime::from_seconds(5.0), SimEvent::DispatchTasks);
        kernel.schedule_at(SimTime::from_seconds(15.0), SimEvent::DispatchTasks);

        assert_eq!(kernel.pending_count(), 3);

        // Events should come out in time order
        let e1 = kernel.pop_next().unwrap();
        assert_eq!(e1.time, SimTime::from_seconds(5.0));
        assert_eq!(kernel.now(), SimTime::from_seconds(5.0));

        let e2 = kernel.pop_next().unwrap();
        assert_eq!(e2.time, SimTime::from_seconds(10.0));

        let e3 = kernel.pop_next().unwrap();
        assert_eq!(e3.time, SimTime::from_seconds(15.0));

        assert!(!kernel.has_events());
    }

    #[test]
    fn test_schedule_after() {
        let mut kernel = Kernel::new();

        // Advance time first
        kernel.schedule_at(SimTime::from_seconds(10.0), SimEvent::DispatchTasks);
        kernel.pop_next();

        // Now schedule after current time
        kernel.schedule_after(SimTime::from_seconds(5.0), SimEvent::DispatchTasks);

        let event = kernel.pop_next().unwrap();
        assert_eq!(event.time, SimTime::from_seconds(15.0));
    }

    #[test]
    fn test_event_cancellation() {
        let mut kernel = Kernel::new();

        let id1 = kernel.schedule_at(SimTime::from_seconds(10.0), SimEvent::DispatchTasks);
        let _id2 = kernel.schedule_at(SimTime::from_seconds(5.0), SimEvent::DispatchTasks);

        assert_eq!(kernel.pending_count(), 2);

        let found = kernel.cancel(id1);
        assert!(found);
        assert_eq!(kernel.pending_count(), 1);

        // The remaining event should be at t=5
        let event = kernel.pop_next().unwrap();
        assert_eq!(event.time, SimTime::from_seconds(5.0));
    }
}
