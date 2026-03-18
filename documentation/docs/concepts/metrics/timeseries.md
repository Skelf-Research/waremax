# Time Series Data

Metrics tracked over simulation time.

---

## What is Time Series Data?

Time series data captures metric values at regular intervals throughout the simulation, showing how the system behaves over time.

### Aggregate vs. Time Series

**Aggregate**: Single value for entire simulation
```
Average throughput: 1,000 tasks/hour
```

**Time Series**: Values at each time point
```
t=0m:  800 tasks/hour
t=5m:  950 tasks/hour
t=10m: 1,100 tasks/hour
t=15m: 900 tasks/hour
```

---

## Why Time Series?

### Identify Patterns

See trends and cycles:

```
Throughput
    │
    │    ╱╲      ╱╲
    │   ╱  ╲    ╱  ╲
    │  ╱    ╲  ╱    ╲
    │ ╱      ╲╱      ╲
    │╱
    └─────────────────── Time
         Peak hours pattern
```

### Detect Issues

Find when problems occur:

```
Queue Length
    │
    │                 ████
    │           ████████████
    │     ██████████████████
    │ ████████████████████████
    └─────────────────────────── Time
                  ↑
         Problem starts here
```

### Validate Warmup

Confirm system reaches steady state:

```
Utilization
    │        ─────────────
    │      ╱
    │     ╱
    │    ╱
    │   ╱
    │  ╱
    └────────────────────── Time
      ↑
    Warmup   Steady State
```

---

## Common Time Series Metrics

### Throughput Over Time

```
Time (min) | Tasks Completed | Rate (tasks/hr)
-----------|-----------------|----------------
0          | 0               | 0
5          | 68              | 816
10         | 152             | 912
15         | 241             | 964
20         | 335             | 1,005
```

### Queue Lengths

```
Time | Station S1 | Station S2 | Total
-----|------------|------------|-------
0    | 0          | 0          | 0
5    | 3          | 2          | 5
10   | 5          | 4          | 9
15   | 4          | 3          | 7
```

### Utilization Over Time

```
Time | Robot Util | Station Util
-----|------------|-------------
0    | 0%         | 0%
5    | 65%        | 58%
10   | 78%        | 72%
15   | 82%        | 80%
20   | 80%        | 78%
```

---

## Configuration

### Enable Time Series

```yaml
metrics:
  timeseries:
    enabled: true
    interval_s: 60  # Sample every minute
```

### Select Metrics

```yaml
metrics:
  timeseries:
    include:
      - throughput
      - utilization
      - queue_length
      - wait_time
      - task_count
```

### Sampling Interval

| Interval | Use Case |
|----------|----------|
| 1s | Detailed debugging |
| 10s | Fine-grained analysis |
| 60s | Standard monitoring |
| 300s | Long simulations |

---

## Output Format

### CSV Format

```csv
timestamp,throughput,robot_utilization,station_utilization,avg_queue_length
0,0,0.00,0.00,0.0
60,82,0.65,0.58,2.3
120,95,0.78,0.72,3.5
180,91,0.80,0.78,3.2
```

### JSON Format

```json
{
  "timeseries": [
    {
      "timestamp": 0,
      "throughput": 0,
      "robot_utilization": 0.00,
      "queue_length": 0.0
    },
    {
      "timestamp": 60,
      "throughput": 82,
      "robot_utilization": 0.65,
      "queue_length": 2.3
    }
  ]
}
```

---

## Analysis Techniques

### Trend Analysis

Identify overall direction:

```
Is throughput increasing, stable, or decreasing?

Early:   ████████░░ (800)
Middle:  ██████████ (1000)
Late:    ████████░░ (800)

Pattern: Peak in middle, degradation later
```

### Moving Average

Smooth out noise:

```
Raw data:      82, 95, 78, 102, 85, 91, 88
3-point avg:   -, 85, 92, 88, 93, 88, -
```

### Correlation

Find relationships:

```
Queue length ↑ → Wait time ↑
Utilization ↑ → Congestion ↑

These correlations help identify root causes
```

---

## Visualization

### Line Charts

Best for continuous metrics:

```
Throughput
1200│         ___
1000│    ____/   \____
 800│   /             \
 600│  /
 400│ /
 200│/
   0└─────────────────────
    0   15   30   45   60
              Time (min)
```

### Stacked Area

For composition:

```
Time Breakdown (cumulative)
100%│████████████████████
    │████ Idle ██████████
    │████████████████████
 50%│▓▓▓▓ Travel ▓▓▓▓▓▓▓▓
    │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
    │░░░░ Working ░░░░░░░
  0%└─────────────────────
```

### Heat Maps

For spatial-temporal data:

```
     t=0  t=5  t=10 t=15 t=20
N1   ░    ░    ▒    ▓    ▓
N2   ░    ▒    ▓    ▓    █
N3   ░    ░    ▒    ▒    ▓
N4   ░    ░    ░    ▒    ▒

░ = Low   ▓ = High
▒ = Med   █ = Max
```

---

## Common Patterns

### Warmup Period

```
      _______________
     /
    /
   /
  /
 /
/
  Warmup → Steady State
```

Exclude warmup from analysis.

### Periodic Behavior

```
    /\    /\    /\    /\
   /  \  /  \  /  \  /  \
  /    \/    \/    \/    \

Regular cycles (e.g., order waves)
```

### Degradation

```
________
        \
         \
          \
           \

Performance declining over time
```

Investigate cause (queue buildup, resource exhaustion).

---

## Best Practices

### Choose Appropriate Interval

- Too short: Noisy data, large files
- Too long: Miss important details

Rule of thumb: 30-100 data points per simulation.

### Include Key Metrics

At minimum:
- Throughput
- Utilization
- Queue length

### Exclude Warmup

```yaml
metrics:
  warmup_s: 300  # Exclude first 5 minutes
```

### Align with Analysis Goals

Debugging: Short interval, many metrics
Comparison: Standard interval, key metrics only

---

## Related

- [KPIs](kpis.md)
- [Metrics Configuration](../../configuration/metrics.md)
- [Analysis Command](../../cli/analyze.md)
