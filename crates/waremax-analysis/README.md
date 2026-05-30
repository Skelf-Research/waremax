# waremax-analysis

**Delay attribution, critical-path analysis, bottleneck detection, anomaly detection, and root-cause reporting for [WareMax](../../README.md) simulations.**

Part of the [WareMax](../../README.md) workspace — a deterministic warehouse-robotics simulator and RL benchmark for Robotic Mobile Fulfillment Systems (RMFS).

## Role

Turns the simulator's event stream into *causal explanations*: where time is lost, which orders sit on the critical path, what's anomalous about a run. The same attribution data also drives the **attribution-shaped RL reward** in [`waremax-rl`](../waremax-rl/) — making this crate dual-purpose: post-hoc RCA *and* an in-loop signal source.

## Key types

| Item | Purpose |
|---|---|
| `AttributionCollector` | Per-task delay decomposition built incrementally during a run; enabled via `.enable()`. |
| `DelayCategory` | `RobotAssignment`, `TravelToPickup`, `TravelToStation`, `CongestionWait`, `StationQueue`, `StationService`, `ChargingDetour`, `MaintenanceDetour`, `FailureRecovery`. `is_waste()` marks non-productive categories. |
| `TaskAttribution` | Per-task record: `time_breakdown: HashMap<DelayCategory, f64>`, `congestion_events`, `queue_waits`, robot/order linkage. |
| `DelayAttributionSummary` | Aggregate stats: per-category averages, percentages, waste ratio. |
| `OrderCriticalPath` / `CriticalPathSummary` | Per-order phase decomposition; identifies the slowest phase. |
| `BottleneckDetector` / `BottleneckAnalysis` | Ranks congested nodes/edges and overloaded stations. |
| `AnomalyDetector` / `Anomaly` | Statistical flagging of unusual orders/tasks. |
| `RCAReporter` | Generates actionable root-cause recommendations (Markdown/HTML). |

## Use as a reward signal

The `Attribution` and `AttributionFull` reward modes in [`waremax-rl`](../waremax-rl/) sum `attr.time_breakdown` over `completed_attributions()` between consecutive decisions; the controllability principle says to include only the categories the agent's decision actually controls (e.g. `RobotAssignment`, `TravelToPickup`).

## See also

- [`waremax-sim`](../waremax-sim/) — the handlers that populate the collector.
- [`waremax-rl`](../waremax-rl/) — uses attribution as a reward signal.
