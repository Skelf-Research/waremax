# Metrics & Analysis

Understanding and measuring simulation performance.

---

## Overview

Metrics quantify simulation outcomes, enabling analysis, comparison, and optimization.

| Topic | Description |
|-------|-------------|
| [Key Performance Indicators](kpis.md) | Core success metrics |
| [Time Series Data](timeseries.md) | Metrics over time |
| [Congestion Metrics](congestion-metrics.md) | Traffic analysis |
| [Root Cause Analysis](rca.md) | Finding bottlenecks |

---

## Why Metrics?

### Quantify Performance

Replace intuition with data:

```
"The system seems slow" → "Avg task time is 45s, 30% higher than target"
```

### Enable Comparison

Compare configurations objectively:

```
Config A: 850 tasks/hour
Config B: 920 tasks/hour (+8.2%)
```

### Guide Optimization

Focus effort where it matters:

```
Analysis: 40% of time spent in station queues
Action:   Add station capacity or adjust policies
```

---

## Metric Categories

### Throughput Metrics

How much work gets done:

| Metric | Description |
|--------|-------------|
| Tasks completed | Total finished tasks |
| Throughput | Tasks per hour |
| Orders fulfilled | Complete orders |

### Time Metrics

How long things take:

| Metric | Description |
|--------|-------------|
| Task time | Creation to completion |
| Travel time | Movement duration |
| Wait time | Blocked/queued time |
| Service time | At station |

### Utilization Metrics

How resources are used:

| Metric | Description |
|--------|-------------|
| Robot utilization | Busy vs. idle time |
| Station utilization | Service vs. idle |
| Edge utilization | Traffic density |

### Quality Metrics

How well targets are met:

| Metric | Description |
|--------|-------------|
| On-time rate | Tasks meeting due time |
| Late tasks | Count/percentage |
| Average lateness | For late tasks |

---

## Metric Levels

### Aggregate Metrics

Summary across simulation:

```
Total tasks: 1,250
Avg task time: 42.3s
Robot utilization: 78%
```

### Time Series Metrics

Values over time:

```
Time | Throughput | Queue Length
-----|------------|-------------
0m   | 0          | 0
5m   | 82         | 3
10m  | 95         | 5
15m  | 88         | 4
```

### Distribution Metrics

Statistical breakdown:

```
Task Time Distribution:
  Min:    12s
  25th:   28s
  Median: 38s
  75th:   52s
  Max:    145s
  Stddev: 18s
```

---

## Output Formats

### Summary (stdout)

Quick overview:

```
=== Simulation Results ===
Duration: 3600s
Tasks completed: 1,250
Throughput: 1,250 tasks/hour
Avg task time: 42.3s
Robot utilization: 78%
```

### JSON

Structured data for processing:

```json
{
  "summary": {
    "duration_s": 3600,
    "tasks_completed": 1250,
    "throughput_per_hour": 1250
  },
  "tasks": {
    "avg_time_s": 42.3,
    "median_time_s": 38.0
  }
}
```

### CSV

For spreadsheet analysis:

```csv
timestamp,throughput,queue_length,utilization
0,0,0,0.00
300,82,3,0.65
600,95,5,0.78
```

---

## Configuration

### Basic Metrics

```yaml
metrics:
  enabled: true
  output_format: json
```

### Time Series

```yaml
metrics:
  timeseries:
    enabled: true
    interval_s: 60
```

### Detailed Breakdown

```yaml
metrics:
  detail_level: high
  include:
    - tasks
    - robots
    - stations
    - traffic
```

---

## Using Metrics

### Analysis Workflow

```
1. Run simulation
   └→ Generate metrics

2. Review summary
   └→ Identify areas of concern

3. Examine time series
   └→ Find when issues occur

4. Drill into details
   └→ Understand root cause

5. Adjust configuration
   └→ Re-run and compare
```

### Example Analysis

```
Summary shows: Low throughput (800/hr vs. 1000/hr target)

Time series shows: Throughput drops at t=1800s

Detail shows: Station S1 queue spikes at t=1800s

Root cause: S1 capacity insufficient during peak

Solution: Add concurrency to S1 or second station
```

---

## Metrics CLI Commands

### Run with Metrics

```bash
waremax run scenario.yaml -o results/
```

### Analyze Results

```bash
waremax analyze results/
```

### Compare Runs

```bash
waremax compare scenario.yaml \
  --param robots.count=10 \
  --param robots.count=15
```

---

## Related

- [Metrics Configuration](../../configuration/metrics.md)
- [Analysis Command](../../cli/analyze.md)
- [Compare Command](../../cli/compare.md)
