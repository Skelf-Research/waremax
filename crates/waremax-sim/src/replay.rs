//! Replay engine for simulation playback
//!
//! v3: Replay recorded simulations from event logs and snapshots

use std::collections::HashMap;
use std::io;
use std::path::Path;

use waremax_metrics::{EventLogReader, TraceEntry};
use crate::snapshot::{WorldSnapshot, SnapshotManager};

/// Replay playback state
#[derive(Clone, Debug, PartialEq)]
pub enum PlaybackState {
    /// Replay is paused
    Paused,
    /// Replay is playing at the given speed multiplier
    Playing { speed: f64 },
    /// Replay has reached the end
    Finished,
}

/// Event with decoded details for replay display
#[derive(Clone, Debug)]
pub struct ReplayEvent {
    /// Event timestamp in seconds
    pub time_s: f64,
    /// Event type name
    pub event_type: String,
    /// Robot ID if applicable
    pub robot_id: Option<u32>,
    /// Node ID if applicable
    pub node_id: Option<u32>,
    /// Station ID if applicable
    pub station_id: Option<u32>,
    /// Task ID if applicable
    pub task_id: Option<u32>,
    /// Additional details as JSON string
    pub details_json: String,
}

impl From<&TraceEntry> for ReplayEvent {
    fn from(entry: &TraceEntry) -> Self {
        // Extract IDs from details using JSON serialization
        let details_json = serde_json::to_string(&entry.details).unwrap_or_default();

        Self {
            time_s: entry.timestamp,
            event_type: entry.event_type.clone(),
            robot_id: None,  // Could parse from details
            node_id: None,
            station_id: None,
            task_id: None,
            details_json,
        }
    }
}

/// Replay engine for playing back recorded simulations
pub struct ReplayEngine {
    /// Event log reader
    event_reader: Option<EventLogReader>,
    /// Snapshot manager with loaded snapshots
    snapshots: SnapshotManager,
    /// Current playback time in seconds
    current_time_s: f64,
    /// Simulation duration in seconds
    duration_s: f64,
    /// Playback state
    state: PlaybackState,
    /// Cached events for current time window
    cached_events: Vec<ReplayEvent>,
    /// Current snapshot (for state at current time)
    current_snapshot: Option<WorldSnapshot>,
}

impl ReplayEngine {
    /// Create a new replay engine from an event log directory
    pub fn from_event_log(event_log_path: &Path) -> io::Result<Self> {
        let reader = EventLogReader::open(event_log_path)?;
        let duration = reader.get_duration().unwrap_or(0.0);

        Ok(Self {
            event_reader: Some(reader),
            snapshots: SnapshotManager::default(),
            current_time_s: 0.0,
            duration_s: duration,
            state: PlaybackState::Paused,
            cached_events: Vec::new(),
            current_snapshot: None,
        })
    }

    /// Create a replay engine from snapshots only (no event log)
    pub fn from_snapshots(snapshots: SnapshotManager) -> Self {
        let duration = snapshots
            .snapshots()
            .last()
            .map(|s| s.timestamp_s)
            .unwrap_or(0.0);

        Self {
            event_reader: None,
            snapshots,
            current_time_s: 0.0,
            duration_s: duration,
            state: PlaybackState::Paused,
            cached_events: Vec::new(),
            current_snapshot: None,
        }
    }

    /// Load snapshots from JSON file
    pub fn load_snapshots(&mut self, path: &Path) -> io::Result<()> {
        let json = std::fs::read_to_string(path)?;
        let snapshots: Vec<WorldSnapshot> = serde_json::from_str(&json)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.snapshots = SnapshotManager::new(60.0);
        for snap in snapshots {
            self.snapshots.store(snap);
        }

        // Update duration if snapshots extend beyond event log
        if let Some(last) = self.snapshots.snapshots().last() {
            self.duration_s = self.duration_s.max(last.timestamp_s);
        }

        Ok(())
    }

    /// Get current playback time in seconds
    pub fn current_time(&self) -> f64 {
        self.current_time_s
    }

    /// Get simulation duration in seconds
    pub fn duration(&self) -> f64 {
        self.duration_s
    }

    /// Get playback state
    pub fn state(&self) -> &PlaybackState {
        &self.state
    }

    /// Get progress as a fraction (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        if self.duration_s <= 0.0 {
            return 0.0;
        }
        (self.current_time_s / self.duration_s).clamp(0.0, 1.0)
    }

    /// Seek to a specific time
    pub fn seek(&mut self, time_s: f64) {
        self.current_time_s = time_s.clamp(0.0, self.duration_s);
        self.update_snapshot();
        self.cached_events.clear();

        if self.current_time_s >= self.duration_s {
            self.state = PlaybackState::Finished;
        }
    }

    /// Start playback at given speed (1.0 = real-time, 10.0 = 10x faster)
    pub fn play(&mut self, speed: f64) {
        if self.current_time_s >= self.duration_s {
            return; // Already at end
        }
        self.state = PlaybackState::Playing { speed };
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
    }

    /// Step forward by a time delta (used during playback)
    pub fn step(&mut self, wall_clock_delta_s: f64) -> Vec<ReplayEvent> {
        if let PlaybackState::Playing { speed } = self.state {
            let sim_delta = wall_clock_delta_s * speed;
            let old_time = self.current_time_s;
            self.current_time_s = (self.current_time_s + sim_delta).min(self.duration_s);

            // Get events in the time window
            let events = self.get_events_in_range(old_time, self.current_time_s);

            // Update snapshot if needed
            self.update_snapshot();

            // Check if finished
            if self.current_time_s >= self.duration_s {
                self.state = PlaybackState::Finished;
            }

            events
        } else {
            Vec::new()
        }
    }

    /// Step forward by one event
    pub fn step_event(&mut self) -> Option<ReplayEvent> {
        if let Some(ref reader) = self.event_reader {
            // Get next event after current time
            let events = reader.get_events(self.current_time_s, self.current_time_s + 3600.0);
            if let Some(entry) = events.into_iter()
                .find(|e| e.timestamp > self.current_time_s)
            {
                self.current_time_s = entry.timestamp;
                self.update_snapshot();
                return Some(ReplayEvent::from(&entry));
            }
        }

        self.state = PlaybackState::Finished;
        None
    }

    /// Get events in a time range
    pub fn get_events_in_range(&self, start_s: f64, end_s: f64) -> Vec<ReplayEvent> {
        if let Some(ref reader) = self.event_reader {
            reader.get_events(start_s, end_s)
                .into_iter()
                .map(|e| ReplayEvent::from(&e))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get current snapshot (state at current time)
    pub fn snapshot(&self) -> Option<&WorldSnapshot> {
        self.current_snapshot.as_ref()
    }

    /// Get robot positions at current time
    pub fn robot_positions(&self) -> HashMap<u32, u32> {
        self.current_snapshot
            .as_ref()
            .map(|s| s.robot_positions())
            .unwrap_or_default()
    }

    /// Get count of robots in each state
    pub fn robot_state_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        if let Some(ref snap) = self.current_snapshot {
            for robot in snap.robots.values() {
                // Parse state type (e.g., "Idle", "Moving { .. }")
                let state_name = robot.state_type
                    .split_whitespace()
                    .next()
                    .unwrap_or(&robot.state_type)
                    .to_string();
                *counts.entry(state_name).or_insert(0) += 1;
            }
        }
        counts
    }

    /// Update current snapshot based on current time
    fn update_snapshot(&mut self) {
        self.current_snapshot = self.snapshots
            .find_at_or_before(self.current_time_s)
            .cloned();
    }

    /// Get simulation metadata
    pub fn metadata(&self) -> ReplayMetadata {
        ReplayMetadata {
            duration_s: self.duration_s,
            event_count: self.event_reader.as_ref().map(|r| r.event_count()).unwrap_or(0),
            snapshot_count: self.snapshots.count(),
            seed: self.event_reader.as_ref().and_then(|r| r.get_seed()),
        }
    }
}

/// Metadata about a recorded simulation
#[derive(Clone, Debug)]
pub struct ReplayMetadata {
    /// Total simulation duration
    pub duration_s: f64,
    /// Number of events in the log
    pub event_count: u64,
    /// Number of snapshots
    pub snapshot_count: usize,
    /// Random seed used in simulation
    pub seed: Option<u64>,
}

/// Replay session for managing multiple replays or comparisons
pub struct ReplaySession {
    /// Named replay engines
    replays: HashMap<String, ReplayEngine>,
    /// Active replay name
    active: Option<String>,
}

impl ReplaySession {
    /// Create a new empty session
    pub fn new() -> Self {
        Self {
            replays: HashMap::new(),
            active: None,
        }
    }

    /// Add a replay to the session
    pub fn add(&mut self, name: &str, engine: ReplayEngine) {
        let is_first = self.replays.is_empty();
        self.replays.insert(name.to_string(), engine);
        if is_first {
            self.active = Some(name.to_string());
        }
    }

    /// Load a replay from an event log path
    pub fn load(&mut self, name: &str, path: &Path) -> io::Result<()> {
        let engine = ReplayEngine::from_event_log(path)?;
        self.add(name, engine);
        Ok(())
    }

    /// Set the active replay
    pub fn set_active(&mut self, name: &str) -> bool {
        if self.replays.contains_key(name) {
            self.active = Some(name.to_string());
            true
        } else {
            false
        }
    }

    /// Get the active replay
    pub fn active(&self) -> Option<&ReplayEngine> {
        self.active.as_ref().and_then(|n| self.replays.get(n))
    }

    /// Get the active replay mutably
    pub fn active_mut(&mut self) -> Option<&mut ReplayEngine> {
        let name = self.active.clone()?;
        self.replays.get_mut(&name)
    }

    /// Get a replay by name
    pub fn get(&self, name: &str) -> Option<&ReplayEngine> {
        self.replays.get(name)
    }

    /// Get all replay names
    pub fn names(&self) -> Vec<&str> {
        self.replays.keys().map(|s| s.as_str()).collect()
    }

    /// Synchronize all replays to the same time
    pub fn sync_time(&mut self, time_s: f64) {
        for engine in self.replays.values_mut() {
            engine.seek(time_s);
        }
    }
}

impl Default for ReplaySession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_engine_from_snapshots() {
        let mut manager = SnapshotManager::new(60.0);
        manager.store(WorldSnapshot {
            timestamp_s: 0.0,
            robots: HashMap::new(),
            stations: HashMap::new(),
            orders: HashMap::new(),
            tasks: HashMap::new(),
            pending_tasks: vec![],
            pending_events: vec![],
            node_occupancy: HashMap::new(),
        });
        manager.store(WorldSnapshot {
            timestamp_s: 60.0,
            robots: HashMap::new(),
            stations: HashMap::new(),
            orders: HashMap::new(),
            tasks: HashMap::new(),
            pending_tasks: vec![],
            pending_events: vec![],
            node_occupancy: HashMap::new(),
        });

        let engine = ReplayEngine::from_snapshots(manager);
        assert_eq!(engine.duration(), 60.0);
        assert_eq!(engine.current_time(), 0.0);
    }

    #[test]
    fn test_replay_seek() {
        let mut manager = SnapshotManager::new(60.0);
        for i in 0..5 {
            manager.store(WorldSnapshot {
                timestamp_s: i as f64 * 60.0,
                robots: HashMap::new(),
                stations: HashMap::new(),
                orders: HashMap::new(),
                tasks: HashMap::new(),
                pending_tasks: vec![],
                pending_events: vec![],
                node_occupancy: HashMap::new(),
            });
        }

        let mut engine = ReplayEngine::from_snapshots(manager);
        engine.seek(150.0);
        assert_eq!(engine.current_time(), 150.0);

        // Snapshot should be at 120s (most recent before 150s)
        assert!(engine.snapshot().is_some());
        assert_eq!(engine.snapshot().unwrap().timestamp_s, 120.0);
    }

    #[test]
    fn test_playback_state() {
        let mut manager = SnapshotManager::new(60.0);
        // Add a snapshot so duration is non-zero
        manager.store(WorldSnapshot {
            timestamp_s: 60.0,
            robots: HashMap::new(),
            stations: HashMap::new(),
            orders: HashMap::new(),
            tasks: HashMap::new(),
            pending_tasks: vec![],
            pending_events: vec![],
            node_occupancy: HashMap::new(),
        });
        let mut engine = ReplayEngine::from_snapshots(manager);

        assert_eq!(*engine.state(), PlaybackState::Paused);
        engine.play(2.0);
        match engine.state() {
            PlaybackState::Playing { speed } => assert!((speed - 2.0).abs() < 0.001),
            _ => panic!("Expected Playing state"),
        }
        engine.pause();
        assert_eq!(*engine.state(), PlaybackState::Paused);
    }

    #[test]
    fn test_replay_session() {
        let mut session = ReplaySession::new();

        let manager1 = SnapshotManager::new(60.0);
        let manager2 = SnapshotManager::new(60.0);

        session.add("run1", ReplayEngine::from_snapshots(manager1));
        session.add("run2", ReplayEngine::from_snapshots(manager2));

        assert!(session.active().is_some());
        assert!(session.set_active("run2"));
        assert_eq!(session.names().len(), 2);
    }
}
