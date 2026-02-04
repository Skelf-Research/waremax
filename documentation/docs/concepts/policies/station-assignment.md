# Station Assignment

Selecting which station serves a task.

---

## Problem

When multiple stations can serve a task, which one should be used?

```
Task: Deliver item for picking

Available Pick Stations:
  S1: 3 robots queued, 10m away
  S2: 1 robot queued, 25m away
  S3: 5 robots queued, 8m away

Which station?
```

---

## Assignment Policies

### Nearest

Select closest station:

```yaml
policies:
  station_assignment: nearest
```

**Algorithm:**

1. Find stations that can handle task type
2. Calculate distance from current position
3. Select nearest

**Example:**

```
Robot at N15

S1: distance = 10m
S2: distance = 25m
S3: distance = 8m   ← Selected
```

**Pros:**

- Minimizes travel
- Simple

**Cons:**

- May create imbalanced loads
- Longer wait times at popular stations

---

### Shortest Queue

Select station with fewest waiting robots:

```yaml
policies:
  station_assignment: shortest_queue
```

**Algorithm:**

1. Count robots queued at each station
2. Select station with shortest queue
3. Tie-breaker: nearest

**Example:**

```
S1: 3 queued
S2: 1 queued   ← Selected
S3: 5 queued
```

**Pros:**

- Balances station utilization
- Reduces wait times

**Cons:**

- May increase travel distance
- Doesn't consider service speed

---

### Fastest Completion

Select station where task will complete soonest:

```yaml
policies:
  station_assignment: fastest_completion
```

**Algorithm:**

1. Estimate completion time for each station:
   - Travel time to station
   - Wait time in queue
   - Service time
2. Select station with earliest completion

**Example:**

```
S1: travel=10s, wait=45s, service=15s → 70s
S2: travel=25s, wait=15s, service=15s → 55s   ← Selected
S3: travel=8s, wait=75s, service=15s → 98s
```

**Pros:**

- Optimizes for task completion time
- Considers all factors

**Cons:**

- Requires accurate estimates
- More computation

---

### Due Time Priority

Select based on task urgency:

```yaml
policies:
  station_assignment:
    type: due_time_priority
    urgency_factor: 1.5
```

**Algorithm:**

1. For urgent tasks: fastest completion
2. For non-urgent: shortest queue or nearest

---

## Comparison

| Policy | Travel | Wait Time | Balance |
|--------|--------|-----------|---------|
| Nearest | Low | Variable | Poor |
| Shortest Queue | Medium | Medium | Good |
| Fastest Completion | Variable | Low | Good |

---

## Station Capacity

### Queue Limits

Stations may have queue limits:

```yaml
stations:
  - id: "S1"
    queue_capacity: 10
```

### Handling Full Queues

Options when queue is full:

1. **Skip**: Choose another station
2. **Wait**: Wait at distance until queue opens
3. **Overflow**: Allow temporary overflow

```yaml
policies:
  station_assignment:
    type: shortest_queue
    full_queue_behavior: skip
```

---

## Multi-Station Scenarios

### Station Zones

Assign stations by zone:

```yaml
policies:
  station_assignment:
    type: zone_based
    zones:
      zone_a: [S1, S2]
      zone_b: [S3, S4]
```

Tasks in zone A use S1 or S2.

### Fallback Stations

Primary and backup stations:

```yaml
policies:
  station_assignment:
    type: nearest
    fallback_enabled: true
    fallback_threshold: 5  # Queue length
```

Use primary if queue < 5, else fallback.

---

## Dynamic Assignment

### Real-Time Adjustment

Reassign if conditions change:

```yaml
policies:
  station_assignment:
    allow_reassignment: true
    reassignment_threshold_s: 60.0
```

### Conditions for Reassignment

- Original station queue grows significantly
- Closer station becomes available
- Original station goes offline

---

## Station Types

### Pick Stations

```yaml
policies:
  station_assignment:
    pick_stations:
      type: fastest_completion
```

### Charging Stations

```yaml
policies:
  station_assignment:
    charging_stations:
      type: nearest
```

### Maintenance Stations

```yaml
policies:
  station_assignment:
    maintenance_stations:
      type: shortest_queue
```

---

## Performance Metrics

### Station-Level

| Metric | Description |
|--------|-------------|
| Utilization | Time busy / Total time |
| Avg queue length | Mean robots waiting |
| Throughput | Tasks per hour |

### System-Level

| Metric | Description |
|--------|-------------|
| Queue imbalance | Stddev of queue lengths |
| Avg wait time | Mean time in queue |
| Task completion time | Total time per task |

---

## Configuration Examples

### Speed-Focused

```yaml
policies:
  station_assignment: nearest
```

### Balance-Focused

```yaml
policies:
  station_assignment:
    type: shortest_queue
    tie_breaker: nearest
```

### Throughput-Focused

```yaml
policies:
  station_assignment:
    type: fastest_completion
    service_time_estimate: moving_average
```

---

## Simulation Testing

### Compare Policies

```bash
waremax compare scenario.yaml \
  --param policies.station_assignment=nearest \
  --param policies.station_assignment=shortest_queue \
  --param policies.station_assignment=fastest_completion
```

### What to Measure

- Station utilization variance
- Average queue length
- Task completion time
- Travel time

---

## Related

- [Stations](../warehouse/stations.md)
- [Station Configuration](../../configuration/stations.md)
- [Task Allocation](task-allocation.md)
