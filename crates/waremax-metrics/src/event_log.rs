//! Persistent event log using sled
//!
//! v3: Store simulation events in a persistent database for replay

use crate::trace::{TraceDetails, TraceEntry};
use sled::{Db, Tree};
use std::io;
use std::path::Path;

/// Event log configuration
#[derive(Clone, Debug)]
pub struct EventLogConfig {
    /// Flush frequency (events between flushes)
    pub flush_every: usize,
    /// Maximum log size in bytes (0 = unlimited)
    pub max_size_bytes: u64,
    /// Enable compression
    pub compress: bool,
}

impl Default for EventLogConfig {
    fn default() -> Self {
        Self {
            flush_every: 1000,
            max_size_bytes: 0,
            compress: false,
        }
    }
}

/// Persistent event log for simulation replay
pub struct EventLog {
    db: Db,
    events: Tree,
    metadata: Tree,
    event_count: u64,
    config: EventLogConfig,
}

impl EventLog {
    /// Open or create an event log at the specified path
    pub fn open(path: &Path, config: EventLogConfig) -> io::Result<Self> {
        let db = sled::open(path).map_err(|e| io::Error::other(format!("sled error: {:?}", e)))?;

        let events = db
            .open_tree("events")
            .map_err(|e| io::Error::other(format!("sled error: {:?}", e)))?;

        let metadata = db
            .open_tree("metadata")
            .map_err(|e| io::Error::other(format!("sled error: {:?}", e)))?;

        // Get current event count from metadata
        let event_count = metadata
            .get("event_count")
            .map_err(|e| io::Error::other(format!("sled error: {:?}", e)))?
            .map(|v| {
                let bytes: [u8; 8] = v.as_ref().try_into().unwrap_or([0; 8]);
                u64::from_be_bytes(bytes)
            })
            .unwrap_or(0);

        Ok(Self {
            db,
            events,
            metadata,
            event_count,
            config,
        })
    }

    /// Append an event to the log
    pub fn append(&mut self, entry: &TraceEntry) -> io::Result<u64> {
        let event_id = self.event_count;
        self.event_count += 1;

        // Create key from timestamp and event ID for ordering
        let key = self.make_key(entry.timestamp, event_id);

        // Serialize event to JSON (could use rkyv for efficiency)
        let value =
            serde_json::to_vec(entry).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Insert into database
        self.events
            .insert(key, value)
            .map_err(|e| io::Error::other(format!("sled error: {:?}", e)))?;

        // Periodic flush
        if self
            .event_count
            .is_multiple_of(self.config.flush_every as u64)
        {
            self.flush()?;
        }

        Ok(event_id)
    }

    /// Flush pending writes to disk
    pub fn flush(&self) -> io::Result<()> {
        // Update event count in metadata
        self.metadata
            .insert("event_count", &self.event_count.to_be_bytes())
            .map_err(|e| io::Error::other(format!("sled error: {:?}", e)))?;

        self.db
            .flush()
            .map_err(|e| io::Error::other(format!("sled error: {:?}", e)))?;

        Ok(())
    }

    /// Get event count
    pub fn event_count(&self) -> u64 {
        self.event_count
    }

    /// Iterate over events in a time range
    pub fn iter_range(&self, start_time_s: f64, end_time_s: f64) -> EventLogIterator {
        let start_key = self.make_key(start_time_s, 0);
        let end_key = self.make_key(end_time_s, u64::MAX);

        EventLogIterator {
            inner: self.events.range(start_key..=end_key),
        }
    }

    /// Iterate over all events
    pub fn iter_all(&self) -> EventLogIterator {
        EventLogIterator {
            inner: self.events.range::<&[u8], _>(..),
        }
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> io::Result<Option<String>> {
        self.metadata
            .get(key)
            .map_err(|e| io::Error::other(format!("sled error: {:?}", e)))?
            .map(|v| String::from_utf8(v.to_vec()))
            .transpose()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Set metadata value
    pub fn set_metadata(&self, key: &str, value: &str) -> io::Result<()> {
        self.metadata
            .insert(key, value.as_bytes())
            .map_err(|e| io::Error::other(format!("sled error: {:?}", e)))?;
        Ok(())
    }

    /// Create a key from timestamp and event ID
    fn make_key(&self, timestamp_s: f64, event_id: u64) -> [u8; 16] {
        let mut key = [0u8; 16];
        let ts_bytes = (timestamp_s * 1_000_000.0) as u64; // Microsecond precision
        key[0..8].copy_from_slice(&ts_bytes.to_be_bytes());
        key[8..16].copy_from_slice(&event_id.to_be_bytes());
        key
    }
}

impl Drop for EventLog {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// Iterator over events in the log
pub struct EventLogIterator {
    inner: sled::Iter,
}

impl Iterator for EventLogIterator {
    type Item = io::Result<TraceEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|result| {
            result
                .map_err(|e| io::Error::other(format!("sled error: {:?}", e)))
                .and_then(|(_, value)| {
                    serde_json::from_slice(&value)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                })
        })
    }
}

/// Event log writer that can be used during simulation
pub struct EventLogWriter {
    log: EventLog,
    enabled: bool,
}

impl EventLogWriter {
    /// Create a new event log writer
    pub fn new(path: &Path) -> io::Result<Self> {
        let log = EventLog::open(path, EventLogConfig::default())?;
        Ok(Self { log, enabled: true })
    }

    /// Create with custom config
    pub fn with_config(path: &Path, config: EventLogConfig) -> io::Result<Self> {
        let log = EventLog::open(path, config)?;
        Ok(Self { log, enabled: true })
    }

    /// Enable or disable logging
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Record an event
    pub fn record(
        &mut self,
        timestamp_s: f64,
        event_type: &str,
        details: TraceDetails,
    ) -> io::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let entry = TraceEntry {
            timestamp: timestamp_s,
            event_type: event_type.to_string(),
            details,
        };

        self.log.append(&entry)?;
        Ok(())
    }

    /// Flush pending writes
    pub fn flush(&self) -> io::Result<()> {
        self.log.flush()
    }

    /// Get event count
    pub fn event_count(&self) -> u64 {
        self.log.event_count()
    }

    /// Set simulation metadata
    pub fn set_simulation_info(&self, duration_s: f64, seed: u64) -> io::Result<()> {
        self.log
            .set_metadata("duration_s", &duration_s.to_string())?;
        self.log.set_metadata("seed", &seed.to_string())?;
        Ok(())
    }
}

/// Event log reader for replay
pub struct EventLogReader {
    log: EventLog,
    current_time_s: f64,
}

impl EventLogReader {
    /// Open an event log for reading
    pub fn open(path: &Path) -> io::Result<Self> {
        let log = EventLog::open(path, EventLogConfig::default())?;
        Ok(Self {
            log,
            current_time_s: 0.0,
        })
    }

    /// Seek to a specific time
    pub fn seek(&mut self, time_s: f64) {
        self.current_time_s = time_s;
    }

    /// Get events in a time window
    pub fn get_events(&self, start_s: f64, end_s: f64) -> Vec<TraceEntry> {
        self.log
            .iter_range(start_s, end_s)
            .filter_map(|r| r.ok())
            .collect()
    }

    /// Get all events
    pub fn get_all_events(&self) -> Vec<TraceEntry> {
        self.log.iter_all().filter_map(|r| r.ok()).collect()
    }

    /// Get event count
    pub fn event_count(&self) -> u64 {
        self.log.event_count()
    }

    /// Get simulation duration from metadata
    pub fn get_duration(&self) -> Option<f64> {
        self.log
            .get_metadata("duration_s")
            .ok()
            .flatten()
            .and_then(|s| s.parse().ok())
    }

    /// Get simulation seed from metadata
    pub fn get_seed(&self) -> Option<u64> {
        self.log
            .get_metadata("seed")
            .ok()
            .flatten()
            .and_then(|s| s.parse().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_event_log_write_read() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test_log");

        // Write some events
        {
            let mut writer = EventLogWriter::new(&path).unwrap();
            writer
                .record(
                    0.0,
                    "start",
                    TraceDetails::Generic {
                        message: "test".to_string(),
                    },
                )
                .unwrap();
            writer
                .record(
                    1.0,
                    "tick",
                    TraceDetails::Generic {
                        message: "tick1".to_string(),
                    },
                )
                .unwrap();
            writer
                .record(
                    2.0,
                    "end",
                    TraceDetails::Generic {
                        message: "done".to_string(),
                    },
                )
                .unwrap();
            writer.flush().unwrap();
            assert_eq!(writer.event_count(), 3);
        }

        // Read events back
        {
            let reader = EventLogReader::open(&path).unwrap();
            let events = reader.get_all_events();
            assert_eq!(events.len(), 3);
            assert_eq!(events[0].event_type, "start");
            assert_eq!(events[2].event_type, "end");
        }
    }
}
