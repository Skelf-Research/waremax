//! Time-series metrics collection for observability

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use waremax_core::{ChargingStationId, EdgeId, NodeId, SimTime, StationId};

/// Time-series data point
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataPoint<T: Clone> {
    pub time_s: f64,
    pub value: T,
}

impl<T: Clone> DataPoint<T> {
    pub fn new(time: SimTime, value: T) -> Self {
        Self {
            time_s: time.as_seconds(),
            value,
        }
    }
}

/// Station time-series data
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StationTimeSeriesData {
    pub queue_length: Vec<DataPoint<usize>>,
    pub utilization: Vec<DataPoint<f64>>,
    pub throughput: Vec<DataPoint<u32>>, // Orders served in interval
}

impl StationTimeSeriesData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_queue_length(&mut self, time: SimTime, length: usize) {
        self.queue_length.push(DataPoint::new(time, length));
    }

    pub fn record_utilization(&mut self, time: SimTime, utilization: f64) {
        self.utilization.push(DataPoint::new(time, utilization));
    }

    pub fn record_throughput(&mut self, time: SimTime, count: u32) {
        self.throughput.push(DataPoint::new(time, count));
    }

    /// Get average queue length over all samples
    pub fn avg_queue_length(&self) -> f64 {
        if self.queue_length.is_empty() {
            return 0.0;
        }
        let sum: usize = self.queue_length.iter().map(|d| d.value).sum();
        sum as f64 / self.queue_length.len() as f64
    }

    /// Get maximum queue length
    pub fn max_queue_length(&self) -> usize {
        self.queue_length.iter().map(|d| d.value).max().unwrap_or(0)
    }
}

/// Charging station time-series data
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChargingTimeSeriesData {
    pub queue_length: Vec<DataPoint<usize>>,
    pub bays_in_use: Vec<DataPoint<usize>>,
    pub utilization: Vec<DataPoint<f64>>,
}

impl ChargingTimeSeriesData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_queue_length(&mut self, time: SimTime, length: usize) {
        self.queue_length.push(DataPoint::new(time, length));
    }

    pub fn record_bays_in_use(&mut self, time: SimTime, count: usize) {
        self.bays_in_use.push(DataPoint::new(time, count));
    }

    pub fn record_utilization(&mut self, time: SimTime, utilization: f64) {
        self.utilization.push(DataPoint::new(time, utilization));
    }
}

/// Congestion metrics for nodes and edges
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CongestionMetrics {
    pub total_wait_time_s: f64,
    pub wait_event_count: u32,
    pub max_occupancy: usize,
    pub total_traversals: u32,
}

impl CongestionMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_wait(&mut self, wait_time: SimTime) {
        self.total_wait_time_s += wait_time.as_seconds();
        self.wait_event_count += 1;
    }

    pub fn record_occupancy(&mut self, occupancy: usize) {
        self.max_occupancy = self.max_occupancy.max(occupancy);
    }

    pub fn record_traversal(&mut self) {
        self.total_traversals += 1;
    }

    /// Calculate congestion score (higher = more congested)
    /// Weighted combination of wait events and total wait time
    pub fn congestion_score(&self) -> f64 {
        (self.wait_event_count as f64 * 10.0) + self.total_wait_time_s
    }

    /// Average wait time per event
    pub fn avg_wait_time_s(&self) -> f64 {
        if self.wait_event_count == 0 {
            0.0
        } else {
            self.total_wait_time_s / self.wait_event_count as f64
        }
    }
}

/// Node congestion ranking entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CongestionRanking {
    pub node_id: Option<NodeId>,
    pub edge_id: Option<EdgeId>,
    pub score: f64,
    pub wait_events: u32,
    pub total_wait_s: f64,
    pub max_occupancy: usize,
}

/// Time-series metrics collector
#[derive(Clone, Debug, Default)]
pub struct TimeSeriesCollector {
    /// Sample interval in seconds
    pub sample_interval_s: f64,
    /// Last sample time
    last_sample_time: f64,
    /// Station time-series data
    pub station_series: HashMap<StationId, StationTimeSeriesData>,
    /// Charging station time-series data
    pub charging_series: HashMap<ChargingStationId, ChargingTimeSeriesData>,
    /// Node congestion metrics
    pub node_congestion: HashMap<NodeId, CongestionMetrics>,
    /// Edge congestion metrics
    pub edge_congestion: HashMap<EdgeId, CongestionMetrics>,
}

impl TimeSeriesCollector {
    pub fn new(sample_interval_s: f64) -> Self {
        Self {
            sample_interval_s,
            last_sample_time: 0.0,
            station_series: HashMap::new(),
            charging_series: HashMap::new(),
            node_congestion: HashMap::new(),
            edge_congestion: HashMap::new(),
        }
    }

    /// Check if it's time to take a sample
    pub fn should_sample(&self, current_time: SimTime) -> bool {
        let current = current_time.as_seconds();
        current - self.last_sample_time >= self.sample_interval_s
    }

    /// Update last sample time
    pub fn mark_sampled(&mut self, time: SimTime) {
        self.last_sample_time = time.as_seconds();
    }

    /// Record station queue length
    pub fn record_station_queue(&mut self, station_id: StationId, time: SimTime, length: usize) {
        self.station_series
            .entry(station_id)
            .or_default()
            .record_queue_length(time, length);
    }

    /// Record station utilization
    pub fn record_station_utilization(&mut self, station_id: StationId, time: SimTime, utilization: f64) {
        self.station_series
            .entry(station_id)
            .or_default()
            .record_utilization(time, utilization);
    }

    /// Record station throughput
    pub fn record_station_throughput(&mut self, station_id: StationId, time: SimTime, count: u32) {
        self.station_series
            .entry(station_id)
            .or_default()
            .record_throughput(time, count);
    }

    /// Record charging station queue length
    pub fn record_charging_queue(&mut self, station_id: ChargingStationId, time: SimTime, length: usize) {
        self.charging_series
            .entry(station_id)
            .or_default()
            .record_queue_length(time, length);
    }

    /// Record charging station bays in use
    pub fn record_charging_bays(&mut self, station_id: ChargingStationId, time: SimTime, count: usize) {
        self.charging_series
            .entry(station_id)
            .or_default()
            .record_bays_in_use(time, count);
    }

    /// Record charging station utilization
    pub fn record_charging_utilization(&mut self, station_id: ChargingStationId, time: SimTime, utilization: f64) {
        self.charging_series
            .entry(station_id)
            .or_default()
            .record_utilization(time, utilization);
    }

    /// Record wait event at a node
    pub fn record_node_wait(&mut self, node_id: NodeId, wait_time: SimTime) {
        self.node_congestion
            .entry(node_id)
            .or_default()
            .record_wait(wait_time);
    }

    /// Record node occupancy
    pub fn record_node_occupancy(&mut self, node_id: NodeId, occupancy: usize) {
        self.node_congestion
            .entry(node_id)
            .or_default()
            .record_occupancy(occupancy);
    }

    /// Record node traversal
    pub fn record_node_traversal(&mut self, node_id: NodeId) {
        self.node_congestion
            .entry(node_id)
            .or_default()
            .record_traversal();
    }

    /// Record wait event at an edge
    pub fn record_edge_wait(&mut self, edge_id: EdgeId, wait_time: SimTime) {
        self.edge_congestion
            .entry(edge_id)
            .or_default()
            .record_wait(wait_time);
    }

    /// Record edge occupancy
    pub fn record_edge_occupancy(&mut self, edge_id: EdgeId, occupancy: usize) {
        self.edge_congestion
            .entry(edge_id)
            .or_default()
            .record_occupancy(occupancy);
    }

    /// Record edge traversal
    pub fn record_edge_traversal(&mut self, edge_id: EdgeId) {
        self.edge_congestion
            .entry(edge_id)
            .or_default()
            .record_traversal();
    }

    /// Get top N congested nodes
    pub fn top_congested_nodes(&self, n: usize) -> Vec<CongestionRanking> {
        let mut rankings: Vec<_> = self
            .node_congestion
            .iter()
            .map(|(node_id, metrics)| CongestionRanking {
                node_id: Some(*node_id),
                edge_id: None,
                score: metrics.congestion_score(),
                wait_events: metrics.wait_event_count,
                total_wait_s: metrics.total_wait_time_s,
                max_occupancy: metrics.max_occupancy,
            })
            .collect();

        rankings.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        rankings.truncate(n);
        rankings
    }

    /// Get top N congested edges
    pub fn top_congested_edges(&self, n: usize) -> Vec<CongestionRanking> {
        let mut rankings: Vec<_> = self
            .edge_congestion
            .iter()
            .map(|(edge_id, metrics)| CongestionRanking {
                node_id: None,
                edge_id: Some(*edge_id),
                score: metrics.congestion_score(),
                wait_events: metrics.wait_event_count,
                total_wait_s: metrics.total_wait_time_s,
                max_occupancy: metrics.max_occupancy,
            })
            .collect();

        rankings.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        rankings.truncate(n);
        rankings
    }

    /// Get total wait events across all nodes
    pub fn total_node_wait_events(&self) -> u32 {
        self.node_congestion.values().map(|m| m.wait_event_count).sum()
    }

    /// Get total wait events across all edges
    pub fn total_edge_wait_events(&self) -> u32 {
        self.edge_congestion.values().map(|m| m.wait_event_count).sum()
    }

    /// Get total wait time across all nodes
    pub fn total_node_wait_time(&self) -> f64 {
        self.node_congestion.values().map(|m| m.total_wait_time_s).sum()
    }

    /// Get total wait time across all edges
    pub fn total_edge_wait_time(&self) -> f64 {
        self.edge_congestion.values().map(|m| m.total_wait_time_s).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_congestion_metrics() {
        let mut metrics = CongestionMetrics::new();
        metrics.record_wait(SimTime::from_seconds(2.0));
        metrics.record_wait(SimTime::from_seconds(3.0));
        metrics.record_occupancy(3);
        metrics.record_traversal();

        assert_eq!(metrics.wait_event_count, 2);
        assert!((metrics.total_wait_time_s - 5.0).abs() < 0.001);
        assert_eq!(metrics.max_occupancy, 3);
        assert_eq!(metrics.total_traversals, 1);
        assert!((metrics.avg_wait_time_s() - 2.5).abs() < 0.001);
        assert!((metrics.congestion_score() - 25.0).abs() < 0.001); // 2*10 + 5
    }

    #[test]
    fn test_time_series_collector() {
        let mut collector = TimeSeriesCollector::new(60.0);

        // Record some data
        collector.record_station_queue(StationId(0), SimTime::from_seconds(0.0), 3);
        collector.record_station_queue(StationId(0), SimTime::from_seconds(60.0), 5);

        collector.record_node_wait(NodeId(1), SimTime::from_seconds(1.5));
        collector.record_edge_wait(EdgeId(0), SimTime::from_seconds(2.0));

        // Check data
        let station_data = collector.station_series.get(&StationId(0)).unwrap();
        assert_eq!(station_data.queue_length.len(), 2);
        assert_eq!(station_data.max_queue_length(), 5);

        // Check congestion
        assert_eq!(collector.total_node_wait_events(), 1);
        assert_eq!(collector.total_edge_wait_events(), 1);
    }

    #[test]
    fn test_top_congested_nodes() {
        let mut collector = TimeSeriesCollector::new(60.0);

        // Create congestion on different nodes
        for _ in 0..5 {
            collector.record_node_wait(NodeId(1), SimTime::from_seconds(1.0));
        }
        for _ in 0..10 {
            collector.record_node_wait(NodeId(2), SimTime::from_seconds(0.5));
        }
        for _ in 0..2 {
            collector.record_node_wait(NodeId(3), SimTime::from_seconds(2.0));
        }

        let top = collector.top_congested_nodes(2);
        assert_eq!(top.len(), 2);
        // Node 2 should be first (10 events * 10 + 5s = 105)
        // Node 1 should be second (5 events * 10 + 5s = 55)
        assert_eq!(top[0].node_id, Some(NodeId(2)));
        assert_eq!(top[1].node_id, Some(NodeId(1)));
    }
}
