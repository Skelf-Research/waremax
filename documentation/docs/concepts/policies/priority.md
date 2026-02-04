# Priority

Task and robot priority systems.

---

## Why Priority?

Not all tasks are equal:

- Urgent orders need faster processing
- VIP customers expect better service
- Some robots may have special capabilities

Priority ensures important work gets done first.

---

## Priority Levels

### Numeric Priority

Higher number = higher priority:

```
Priority 10: Emergency/VIP
Priority 7:  Express orders
Priority 5:  Standard orders (default)
Priority 3:  Low priority
Priority 1:  Background tasks
```

### Named Levels

```yaml
priorities:
  critical: 10
  high: 7
  normal: 5
  low: 3
  background: 1
```

---

## Task Priority

### Assignment

Tasks receive priority from:

**Order-based:**
```yaml
orders:
  - order_id: "O1"
    priority: 8  # High priority order
```

**Time-based:**
```yaml
policies:
  priority:
    type: due_time
    urgent_threshold_s: 300  # 5 min before due
    urgent_boost: 3
```

**Rule-based:**
```yaml
policies:
  priority:
    rules:
      - condition: "sku.category == 'perishable'"
        priority: 8
      - condition: "customer.tier == 'vip'"
        priority: 7
```

### Dynamic Priority

Priority can change over time:

```
Task created:     Priority 5
30 min remaining: Priority 6
10 min remaining: Priority 8
Overdue:          Priority 10
```

---

## Priority Effects

### Task Allocation

Higher priority tasks assigned first:

```
Queue:
  Task A (priority 7)  ← Assigned first
  Task B (priority 5)
  Task C (priority 3)
```

### Station Queue

Priority affects queue position:

```yaml
policies:
  station_queue:
    type: priority_based
```

```
Station Queue (with priority):
  Position 1: R1 with priority 8 task
  Position 2: R2 with priority 7 task
  Position 3: R3 with priority 5 task (arrived first)
```

### Traffic Priority

Higher priority robots may have right-of-way:

```yaml
policies:
  traffic:
    priority_routing: true
```

---

## Priority Schemes

### FIFO (First In, First Out)

No priority consideration:

```yaml
policies:
  priority:
    scheme: fifo
```

```
Queue order = Arrival order
```

### Strict Priority

Always process highest priority first:

```yaml
policies:
  priority:
    scheme: strict
```

```
Priority 10 always before Priority 9
Priority 9 always before Priority 8
...
```

**Risk**: Low priority tasks may starve.

### Weighted Fair

Balance priority with fairness:

```yaml
policies:
  priority:
    scheme: weighted_fair
    weight_factor: 2.0
```

Higher priority gets more service but doesn't completely block lower priority.

### Aging

Priority increases with wait time:

```yaml
policies:
  priority:
    scheme: aging
    age_factor: 0.1  # +0.1 priority per minute
```

```
Task at t=0:  Priority 3
Task at t=10: Priority 4 (aged)
Task at t=20: Priority 5 (aged more)
```

Prevents starvation of low-priority tasks.

---

## Preemption

### Concept

Higher priority task interrupts lower priority:

```
R1 executing Task A (priority 3)
Task B arrives (priority 9)

Preemption:
  - Task A paused
  - R1 assigned Task B
  - Task A resumed later (or reassigned)
```

### Configuration

```yaml
policies:
  preemption:
    enabled: true
    min_priority_difference: 3
```

### Preemption Points

When can preemption occur:

| Point | Description |
|-------|-------------|
| Any time | Can interrupt mid-task |
| At node | Only at node arrival |
| At completion | Only between tasks |

```yaml
policies:
  preemption:
    point: at_node
```

---

## Robot Priority

### Static Priority

Some robots have permanent priority:

```yaml
robots:
  - id: "R1"
    priority: 7  # VIP robot
  - id: "R2"
    priority: 5  # Standard
```

### Task-Inherited Priority

Robot inherits priority from current task:

```yaml
policies:
  robot_priority:
    type: task_inherited
```

```
R1 with priority 3 task → Robot priority 3
R2 with priority 8 task → Robot priority 8

Traffic conflict: R2 wins
```

---

## Configuration Example

```yaml
policies:
  priority:
    # Base scheme
    scheme: aging
    age_factor: 0.1

    # Task priority sources
    default_priority: 5
    due_time_boost: true
    urgent_threshold_s: 300
    urgent_boost: 3

    # Preemption
    preemption_enabled: true
    min_priority_difference: 4
    preemption_point: at_node

    # Robot priority
    robot_priority: task_inherited

    # Traffic
    traffic_priority: true
```

---

## Priority Metrics

### Task Metrics

| Metric | Description |
|--------|-------------|
| Avg completion by priority | Time by priority level |
| Priority violations | High priority delayed |
| Starvation count | Tasks waiting too long |

### Fairness Metrics

| Metric | Description |
|--------|-------------|
| Wait time variance | Spread of wait times |
| Completion rate by priority | Tasks completed per level |
| Preemption count | How often preemption occurs |

---

## Best Practices

### Don't Overuse High Priority

```
If everything is priority 10:
  → Nothing is really high priority
  → System behaves like FIFO
```

Reserve high priority for truly urgent tasks.

### Use Aging for Fairness

Prevents low-priority starvation:

```yaml
policies:
  priority:
    scheme: aging
    age_factor: 0.1
    max_priority: 9  # Cap priority growth
```

### Monitor Priority Distribution

Track how priorities are distributed:

```
Priority Distribution:
  10: ██ (5%)
  7-9: ████ (15%)
  4-6: ████████████████ (60%)
  1-3: █████ (20%)
```

---

## Related

- [Task Allocation](task-allocation.md)
- [Station Assignment](station-assignment.md)
- [Policy Configuration](../../configuration/policies.md)
