# Congestion Metrics

Measuring and analyzing traffic congestion.

---

## Why Congestion Metrics?

Congestion directly impacts performance:

- Longer travel times
- Reduced throughput
- Wasted energy
- Deadlock risk

Measuring congestion enables:

- Identifying bottlenecks
- Comparing routing policies
- Optimizing layout

---

## Core Congestion Metrics

### Wait Time

Time robots spend waiting for resources:

```
Wait Time = Actual Travel - Free Flow Travel
```

**Example:**
```
Free flow: 10s
Actual: 18s
Wait time: 8s
```

### Aggregate Wait Metrics

| Metric | Formula |
|--------|---------|
| Total wait | Σ wait_time for all robots |
| Avg wait | Total wait / Task count |
| Wait ratio | Total wait / Total travel time |

---

### Occupancy

Resource utilization over time:

```
Occupancy = Time Occupied / Total Time
```

**For nodes:**
```
Node N5: Occupied 720s out of 3600s
Occupancy: 20%
```

**For edges:**
```
Edge E1: Occupied 1800s out of 3600s
Occupancy: 50%
```

---

### Queue Length

Robots waiting for resources:

```
Instantaneous: Current queue at time t
Average: Mean queue over time period
Maximum: Peak queue length
```

**Station queues:**
```
Station S1 Queue:
  Avg: 3.2 robots
  Max: 8 robots
  Time > 5: 15% of simulation
```

---

## Congestion Hotspot Metrics

### Hotspot Identification

Find locations with highest congestion:

```
Node Congestion Ranking:
1. N15 (intersection): 85% occupancy, 12s avg wait
2. N22 (station approach): 72% occupancy, 8s avg wait
3. N8 (crossing): 65% occupancy, 6s avg wait
```

### Congestion Score

Composite metric for each location:

```
Score = occupancy × wait_time × blocking_events
```

Higher score = worse congestion.

---

## Time-Based Congestion

### Congestion Over Time

Track how congestion varies:

```
Time    | Avg Occupancy | Avg Wait | Blocked Events
--------|---------------|----------|----------------
0-15m   | 15%           | 1.2s     | 45
15-30m  | 45%           | 5.8s     | 180
30-45m  | 62%           | 9.3s     | 320
45-60m  | 38%           | 4.1s     | 125
```

### Peak Congestion

When congestion is highest:

```
Peak Congestion Period: 30-45 minutes
  - Coincides with order peak
  - Focus optimization here
```

---

## Deadlock Metrics

### Deadlock Count

Total deadlocks detected:

```
Deadlocks: 3
  - t=1823s: R5, R12 at N15/N16
  - t=2156s: R3, R8, R21 at intersection
  - t=3012s: R7, R15 at station approach
```

### Resolution Metrics

| Metric | Value |
|--------|-------|
| Avg resolution time | 2.3s |
| Robots affected | 8 |
| Tasks delayed | 12 |

---

## Spatial Congestion Analysis

### Congestion Heat Map

```
Warehouse Map (congestion intensity):

░░░░░░░░░░░░░░░░░░░░
░░░░░▒▒▒▒░░░░░░░░░░░
░░░▒▒▓▓▓▓▒▒░░░░░░░░░
░░▒▓▓████▓▓▒░░░░░░░░  ← Hotspot
░░░▒▒▓▓▓▓▒▒░░░░░░░░░
░░░░░▒▒▒▒░░░░░░░░░░░
░░░░░░░░░░░░░░░░░░░░

░ = Low    ▓ = High
▒ = Med    █ = Critical
```

### Path Congestion

Average congestion along common paths:

```
Path A→B (via main aisle): Avg wait 8.5s
Path A→B (via side aisle): Avg wait 2.1s
```

---

## Throughput Impact

### Congestion vs. Throughput

```
Congestion Level | Throughput | Efficiency
-----------------|------------|------------
Low (<20%)       | 1,000/hr   | 100%
Medium (20-40%)  | 920/hr     | 92%
High (40-60%)    | 780/hr     | 78%
Critical (>60%)  | 550/hr     | 55%
```

### Marginal Impact

```
Each 10% increase in congestion → ~8% throughput decrease
```

---

## Configuration

### Enable Congestion Tracking

```yaml
metrics:
  congestion:
    enabled: true
    track_locations: true
    track_timeseries: true
    sample_interval_s: 30
```

### Hotspot Detection

```yaml
metrics:
  congestion:
    hotspot_detection: true
    hotspot_threshold: 0.5  # 50% occupancy
    top_n_hotspots: 10
```

### Detailed Tracking

```yaml
metrics:
  congestion:
    per_node: true
    per_edge: true
    wait_time_breakdown: true
```

---

## Output Example

### Summary

```
=== Congestion Summary ===
Total wait time: 4,523s
Avg wait per task: 3.6s
Peak congestion: 62% at t=2100s

Hotspots:
  1. N15: 85% occupancy
  2. N22: 72% occupancy
  3. N8: 65% occupancy

Deadlocks: 3
Avg resolution: 2.3s
```

### JSON Detail

```json
{
  "congestion": {
    "total_wait_s": 4523,
    "avg_wait_s": 3.6,
    "peak_occupancy": 0.62,
    "peak_time_s": 2100,
    "hotspots": [
      {"node": "N15", "occupancy": 0.85, "avg_wait_s": 12.3},
      {"node": "N22", "occupancy": 0.72, "avg_wait_s": 8.1}
    ],
    "deadlocks": {
      "count": 3,
      "avg_resolution_s": 2.3
    }
  }
}
```

---

## Using Congestion Metrics

### Identify Problems

```
High wait time + Low throughput = Congestion issue

Look at:
1. Hotspot locations
2. Peak congestion times
3. Deadlock frequency
```

### Guide Solutions

| Finding | Solution |
|---------|----------|
| Single hotspot | Increase capacity or add bypass |
| Widespread congestion | Reduce fleet or improve layout |
| Time-based peaks | Adjust order scheduling |
| Frequent deadlocks | Improve traffic policies |

### Validate Improvements

```
Before: Avg wait 8.5s, 3 deadlocks
After:  Avg wait 3.2s, 0 deadlocks

Improvement: 62% reduction in wait time
```

---

## Related

- [Congestion Concepts](../traffic/congestion.md)
- [Traffic Configuration](../../configuration/traffic.md)
- [Root Cause Analysis](rca.md)
