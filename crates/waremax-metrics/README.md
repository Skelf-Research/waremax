# waremax-metrics

**Metrics collection, time-series, SLA tracking, and CSV/JSON/HTML/PDF reporting for [WareMax](../../README.md).**

Part of the [WareMax](../../README.md) workspace — a deterministic warehouse-robotics simulator and RL benchmark for Robotic Mobile Fulfillment Systems (RMFS).

## Role

Records simulation events and aggregates them into reports: throughput, SLA on-time / lateness, cycle-time distributions, per-robot and per-station utilization, queue dynamics, congestion heatmaps, battery, reliability. Designed to be read **mid-run** (so the RL reward can use incremental aggregates) and to render publication-ready summaries.

## Key types

| Item | Purpose |
|---|---|
| `MetricsCollector` | Incremental event recorder; `record_event`, `record_order_complete`, getters `orders_completed`, `orders_late`, `avg_cycle_time`, `p95_cycle_time`, `sla_metrics`. |
| `SimulationReport` | Final report: duration, events, throughput/h, p95 lateness, robot/station utilization, optional `sla`, `congestion`, `battery`, `reliability`, `robot_reports`, `station_reports`, `heatmap`. |
| `SLAReport`, `CongestionReport`, `BatteryReport`, `ReliabilityReport`, `RobotReport`, `StationReport`, `HeatmapData` | Sub-reports. |
| `TimeSeriesCollector` | Per-interval samples (station queue, node/edge wait, occupancy). |
| `EventTraceCollector` | Sampled per-event trace for replay/debugging. |
| `write_exports`, `to_json`, `summary` | CSV / JSON / human export. |

## Outputs

- `report.json` — full report as JSON.
- `robots.csv`, `stations.csv` — per-entity breakdowns.
- `node_congestion.csv`, `edge_congestion.csv` — heatmap data.
- `timeseries.csv`, `trace.csv` — optional time-series and event traces.
- HTML/PDF reports with charts (throughput, queue dynamics, utilization).

## See also

- [`waremax-sim::SimulationRunner::generate_full_report`](../waremax-sim/) — the report producer.
- [`waremax-analysis`](../waremax-analysis/) — delay attribution and root-cause analysis built on top.
