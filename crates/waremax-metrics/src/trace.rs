//! Event trace collector for debugging and analysis

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use waremax_core::SimTime;

/// A single event trace entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraceEntry {
    pub timestamp: f64,
    pub event_type: String,
    pub details: TraceDetails,
}

/// Details for different trace event types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TraceDetails {
    RobotMove {
        robot_id: u32,
        from_node: u32,
        to_node: u32,
    },
    TaskAssign {
        task_id: u32,
        robot_id: u32,
    },
    TaskComplete {
        task_id: u32,
        robot_id: u32,
    },
    OrderComplete {
        order_id: u32,
        cycle_time_s: f64,
        is_late: bool,
    },
    StationService {
        station_id: u32,
        robot_id: u32,
        duration_s: f64,
    },
    RobotFailure {
        robot_id: u32,
    },
    RobotMaintenance {
        robot_id: u32,
        station_id: u32,
        is_repair: bool,
    },
    ChargingStart {
        robot_id: u32,
        station_id: u32,
        soc: f64,
    },
    ChargingEnd {
        robot_id: u32,
        energy_wh: f64,
    },
    Generic {
        message: String,
    },
}

impl TraceEntry {
    pub fn new(timestamp: SimTime, event_type: &str, details: TraceDetails) -> Self {
        Self {
            timestamp: timestamp.as_seconds(),
            event_type: event_type.to_string(),
            details,
        }
    }
}

/// Ring-buffer based event trace collector with optional sampling
#[derive(Clone)]
pub struct EventTraceCollector {
    entries: VecDeque<TraceEntry>,
    max_entries: usize,
    sample_rate: f64,
    enabled: bool,
    sample_counter: u64,
}

impl Default for EventTraceCollector {
    fn default() -> Self {
        Self::new(10000, 1.0)
    }
}

impl EventTraceCollector {
    /// Create a new trace collector
    /// - max_entries: Maximum number of entries to keep (ring buffer)
    /// - sample_rate: 1.0 = all events, 0.1 = 10% of events
    pub fn new(max_entries: usize, sample_rate: f64) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries.min(10000)),
            max_entries,
            sample_rate: sample_rate.clamp(0.0, 1.0),
            enabled: false,
            sample_counter: 0,
        }
    }

    /// Enable or disable the collector
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if this event should be sampled (deterministic based on counter)
    pub fn should_sample(&mut self) -> bool {
        if !self.enabled {
            return false;
        }
        if self.sample_rate >= 1.0 {
            return true;
        }
        self.sample_counter += 1;
        // Deterministic sampling based on sample rate
        let threshold = (self.sample_rate * 100.0) as u64;
        (self.sample_counter % 100) < threshold
    }

    /// Record an event trace entry
    pub fn record(&mut self, timestamp: SimTime, event_type: &str, details: TraceDetails) {
        if !self.enabled {
            return;
        }

        let entry = TraceEntry::new(timestamp, event_type, details);

        // Ring buffer behavior
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Record if sampling says we should
    pub fn record_sampled(&mut self, timestamp: SimTime, event_type: &str, details: TraceDetails) {
        if self.should_sample() {
            self.record(timestamp, event_type, details);
        }
    }

    /// Get all entries
    pub fn entries(&self) -> &VecDeque<TraceEntry> {
        &self.entries
    }

    /// Convert to Vec for serialization
    pub fn to_vec(&self) -> Vec<TraceEntry> {
        self.entries.iter().cloned().collect()
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get entries in a time range
    pub fn entries_in_range(&self, start_s: f64, end_s: f64) -> Vec<&TraceEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp >= start_s && e.timestamp <= end_s)
            .collect()
    }

    /// Get entries by event type
    pub fn entries_by_type(&self, event_type: &str) -> Vec<&TraceEntry> {
        self.entries
            .iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_collector_basic() {
        let mut collector = EventTraceCollector::new(100, 1.0);
        collector.set_enabled(true);

        collector.record(
            SimTime::from_seconds(1.0),
            "RobotMove",
            TraceDetails::RobotMove {
                robot_id: 1,
                from_node: 10,
                to_node: 20,
            },
        );

        assert_eq!(collector.len(), 1);
        let entry = &collector.entries()[0];
        assert_eq!(entry.timestamp, 1.0);
        assert_eq!(entry.event_type, "RobotMove");
    }

    #[test]
    fn test_trace_collector_ring_buffer() {
        let mut collector = EventTraceCollector::new(3, 1.0);
        collector.set_enabled(true);

        for i in 0..5 {
            collector.record(
                SimTime::from_seconds(i as f64),
                "Test",
                TraceDetails::Generic {
                    message: format!("Event {}", i),
                },
            );
        }

        // Should only have last 3 entries
        assert_eq!(collector.len(), 3);
        assert_eq!(collector.entries()[0].timestamp, 2.0);
        assert_eq!(collector.entries()[1].timestamp, 3.0);
        assert_eq!(collector.entries()[2].timestamp, 4.0);
    }

    #[test]
    fn test_trace_collector_disabled() {
        let mut collector = EventTraceCollector::new(100, 1.0);
        // Not enabled by default

        collector.record(
            SimTime::from_seconds(1.0),
            "Test",
            TraceDetails::Generic {
                message: "test".to_string(),
            },
        );

        assert_eq!(collector.len(), 0);
    }

    #[test]
    fn test_trace_collector_sampling() {
        let mut collector = EventTraceCollector::new(100, 0.5);
        collector.set_enabled(true);

        let mut sampled_count = 0;
        for _ in 0..100 {
            if collector.should_sample() {
                sampled_count += 1;
            }
        }

        // With 50% sample rate, should sample approximately 50
        assert!(sampled_count >= 40 && sampled_count <= 60);
    }
}
