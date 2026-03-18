# Task Allocation

Assigning tasks to robots.

---

## Problem

When a task needs execution, which robot should handle it?

```
New Task: Pick from R5, deliver to S1

Available Robots:
  R1: Idle at N10 (close to R5)
  R2: Idle at N50 (far from R5)
  R3: Busy (1 task queued)

Which robot gets the task?
```

---

## Allocation Policies

### Nearest Idle

Assign to closest available robot:

```yaml
policies:
  task_allocation: nearest_idle
```

**Algorithm:**

1. Find all idle robots
2. Calculate distance to task origin
3. Select robot with shortest distance

**Example:**

```
Task at R5 (position N20)

R1 at N22: distance = 3m  ← Selected
R2 at N50: distance = 30m
R3: Busy (skip)
```

**Pros:**

- Minimizes travel time
- Simple to implement
- Fast decisions

**Cons:**

- May create unbalanced utilization
- Doesn't consider future tasks

---

### Least Busy

Assign to robot with fewest pending tasks:

```yaml
policies:
  task_allocation: least_busy
```

**Algorithm:**

1. Count queued tasks per robot
2. Select robot with lowest count
3. Tie-breaker: nearest

**Example:**

```
R1: 2 tasks queued
R2: 0 tasks queued  ← Selected
R3: 1 task queued
```

**Pros:**

- Balances workload
- Better for high-volume scenarios

**Cons:**

- May assign distant robots
- More travel overall

---

### Round Robin

Cycle through robots in order:

```yaml
policies:
  task_allocation: round_robin
```

**Algorithm:**

1. Track last assigned robot
2. Assign to next robot in sequence
3. Skip busy robots

**Example:**

```
Last assigned: R2

Sequence: R1 → R2 → R3 → R1...

Next task → R3
```

**Pros:**

- Perfect balance over time
- Predictable

**Cons:**

- Ignores distance
- Ignores current state

---

### Due Time Priority

Assign based on task urgency:

```yaml
policies:
  task_allocation:
    type: due_time_priority
    urgency_weight: 2.0
```

**Algorithm:**

1. Score = distance + urgency_weight × time_to_due
2. Select robot minimizing score

**Example:**

```
Task: Due in 60s

R1: 10m away, score = 10 + 2×60 = 130
R2: 30m away, score = 30 + 2×60 = 150

R1 selected (lower score)
```

---

## Comparison

### Metrics Impact

| Policy | Travel | Balance | Throughput |
|--------|--------|---------|------------|
| Nearest Idle | Low | Medium | High |
| Least Busy | Medium | High | Medium |
| Round Robin | High | Perfect | Low |
| Due Time | Variable | Medium | High |

### Best For

| Policy | Best When |
|--------|-----------|
| Nearest Idle | Speed matters most |
| Least Busy | Fleet utilization matters |
| Round Robin | Simple fairness needed |
| Due Time | Orders have SLAs |

---

## Visual Comparison

### Nearest Idle

```
Task: ★

Robots:     R1●        R2●            R3●
            5m         15m            25m

Selected: R1 (nearest)
```

### Least Busy

```
Task: ★

Robots:     R1●(2)     R2●(0)         R3●(1)
            5m         15m            25m

Selected: R2 (zero queue)
```

---

## Dynamic Allocation

### Real-Time Reassignment

Tasks can be reassigned if:

- Better robot becomes available
- Original robot delayed
- Priority changes

```yaml
policies:
  task_allocation:
    type: nearest_idle
    allow_reassignment: true
    reassignment_threshold_s: 30.0
```

### Preemption

Higher priority tasks can bump lower priority:

```yaml
policies:
  task_allocation:
    preemption: true
    priority_threshold: 8
```

---

## Multi-Factor Allocation

### Weighted Scoring

Combine multiple factors:

```yaml
policies:
  task_allocation:
    type: weighted
    distance_weight: 1.0
    queue_weight: 0.5
    battery_weight: 0.3
```

**Score calculation:**

```
score = distance × distance_weight
      + queue_length × queue_weight
      + (100 - battery_pct) × battery_weight
```

Lower score wins.

---

## Configuration Examples

### Speed-Focused

```yaml
policies:
  task_allocation:
    type: nearest_idle
```

### Balance-Focused

```yaml
policies:
  task_allocation:
    type: least_busy
    tie_breaker: nearest
```

### SLA-Focused

```yaml
policies:
  task_allocation:
    type: due_time_priority
    urgency_weight: 2.0
    late_penalty: 10.0
```

---

## Performance Testing

### Compare Policies

```bash
waremax compare scenario.yaml \
  --param policies.task_allocation=nearest_idle \
  --param policies.task_allocation=least_busy \
  --param policies.task_allocation=round_robin
```

### Key Metrics to Watch

| Metric | Description |
|--------|-------------|
| avg_travel_time | Lower is better for nearest |
| utilization_stddev | Lower is better for balanced |
| throughput | Tasks per hour |
| late_tasks | Missed due times |

---

## Related

- [Policy Configuration](../../configuration/policies.md)
- [Station Assignment](station-assignment.md)
- [Priority](priority.md)
