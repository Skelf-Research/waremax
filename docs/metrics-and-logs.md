# Metrics and Logs

Waremax produces structured outputs that can be consumed by dashboards, analysis scripts, or visualization tools.

## Key KPIs

- **Throughput**: orders completed per hour
- **Order Cycle Time**: average and tail latency (p95)
- **Robot Utilization**: moving, waiting, servicing, charging
- **Station Utilization**: busy time vs idle time
- **Queue Lengths**: mean and max per station
- **Congestion**: wait time per edge/node, hotspot ranking
- **SLA Miss Rate**: percentage of late orders

## Event Log

Event log is append-only and records all state transitions.

Suggested fields:

- `ts` (simulation time)
- `event_type`
- `robot_id` (optional)
- `task_id` (optional)
- `node_id` / `edge_id` (optional)
- `station_id` (optional)
- `details` (structured JSON)

Example event:

```json
{
  "ts": 123.4,
  "event_type": "robot_arrive_node",
  "robot_id": "R12",
  "node_id": "N18",
  "details": {"from": "N17", "travel_time": 2.1}
}
```

## Trace Data

Trace data is optimized for visualization and replay.

- Per-robot position samples over time
- Per-station queue lengths over time
- Optional edge occupancy over time

## Scenario Summary Report

At the end of a run, output a summary report with:

- Scenario hash and seed
- Duration and warmup configuration
- Headline KPIs
- Policy configuration

This report is intended for experiment tracking and comparisons.
