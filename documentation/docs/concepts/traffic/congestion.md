# Congestion

Traffic buildup and its measurement.

---

## What is Congestion?

Congestion occurs when demand for resources exceeds capacity, causing delays.

### Congestion Indicators

- Robots waiting for access
- Increased travel times
- Queue formation
- Reduced throughput

---

## Measuring Congestion

### Wait Time

Time spent waiting for resources:

```
Wait time = Actual travel time - Theoretical travel time
```

### Occupancy

Resource utilization over time:

```
Occupancy = Time occupied / Total time
```

### Queue Length

Number of robots waiting:

```
Avg queue = Σ(queue_length × duration) / Total time
```

---

## Congestion Levels

### Free Flow

```
Occupancy: ████░░░░░░ (40%)
Wait time: ~0
Throughput: 100%

Robots move without delay
```

### Light Congestion

```
Occupancy: ██████░░░░ (60%)
Wait time: Low
Throughput: 95%

Occasional brief waits
```

### Moderate Congestion

```
Occupancy: ████████░░ (80%)
Wait time: Noticeable
Throughput: 80%

Frequent waits, queues forming
```

### Heavy Congestion

```
Occupancy: ██████████ (100%)
Wait time: High
Throughput: 50%

Long queues, significant delays
```

### Gridlock

```
Occupancy: ██████████ (100%)
Wait time: Infinite
Throughput: 0%

No movement, deadlock risk
```

---

## Congestion Patterns

### Hotspots

Locations with consistently high congestion:

```
Warehouse Map:
  ░░░░░░░░░░
  ░░░░██░░░░  ← Hotspot
  ░░░░░░░░░░
  ░░░░░░░░░░
```

Common hotspots:
- Station approaches
- Main intersections
- Narrow passages

### Time-Based Patterns

Congestion varies over time:

```
Congestion
    │
    │     ╱╲
    │    ╱  ╲
    │   ╱    ╲──────╱╲
    │  ╱              ╲
    │ ╱                ╲
    │╱                  ╲
    └─────────────────────── Time
         Peak   Off-peak
```

### Wave Propagation

Congestion spreads from source:

```
t=0:   ░░░█░░░  Initial congestion
t=1:   ░░██░░░  Spreads backward
t=2:   ░███░░░  Wave grows
t=3:   ████░░░  Propagates further
```

---

## Congestion Impact

### On Individual Robots

| Metric | Impact |
|--------|--------|
| Travel time | Increased |
| Task completion | Delayed |
| Battery usage | Increased (waiting) |

### On System

| Metric | Impact |
|--------|--------|
| Throughput | Reduced |
| Predictability | Decreased |
| Resource utilization | May decrease |

---

## Congestion-Aware Routing

### Standard Routing

Finds shortest path regardless of traffic:

```
Shortest: A → B → C (congested)
```

### Congestion-Aware

Avoids congested areas:

```yaml
routing:
  policy: congestion_aware
  congestion_weight: 1.5
```

May choose longer but faster path:

```
Alternative: A → D → E → C (clear)
```

### Path Cost

```
cost = distance + congestion_weight × congestion_score
```

---

## Congestion Metrics

### Real-Time Metrics

| Metric | Description |
|--------|-------------|
| Current occupancy | Robots/Capacity ratio |
| Wait queue length | Robots waiting |
| Wait time (current) | Accumulated wait |

### Aggregate Metrics

| Metric | Description |
|--------|-------------|
| Avg occupancy | Mean over time period |
| Max occupancy | Peak congestion |
| Total wait time | Sum of all waits |
| Wait time per task | Average per completed task |

---

## Congestion Visualization

### Heat Map

```
Traffic Density:
  ░ = Low    (0-25%)
  ▒ = Medium (25-50%)
  ▓ = High   (50-75%)
  █ = Full   (75-100%)

  ░░░░▒▒░░░░
  ░░▒▓▓▓▒░░░
  ░▒▓███▓▒░░
  ░░▒▓▓▓▒░░░
  ░░░░▒▒░░░░
```

### Time Series

```
Occupancy %
100│        ███
   │      ███████
 75│    ███████████
   │  █████████████
 50│███████████████████
   │█████████████████████
 25│███████████████████████
   │█████████████████████████
  0└──────────────────────────
    0   5   10  15  20  25  30
              Time (min)
```

---

## Managing Congestion

### Prevention

- Better route planning
- Load balancing
- Off-peak scheduling

### Mitigation

- Dynamic rerouting
- Traffic signals
- Priority systems

### Design Solutions

- Wider aisles at hotspots
- Parallel routes
- Better station placement

---

## Configuration

```yaml
traffic:
  # Congestion thresholds
  reroute_on_congestion: true
  reroute_threshold: 3.0  # Wait time trigger

routing:
  # Congestion-aware pathfinding
  policy: congestion_aware
  congestion_weight: 1.5

metrics:
  # Congestion monitoring
  track_congestion: true
  congestion_sample_interval_s: 10
```

---

## Related

- [Capacity](capacity.md)
- [Deadlock](deadlock.md)
- [Routing Configuration](../../configuration/routing.md)
