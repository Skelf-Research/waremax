# Policy Configuration

Configuration for dispatching and scheduling policies.

---

## Schema

```yaml
policies:
  task_allocation:
    type: <string>
    # Additional options depend on type

  station_assignment:
    type: <string>

  batching:
    type: <string>
    max_items: <integer>
    max_weight_kg: <float>

  priority:
    type: <string>
    pick_weight: <integer>
    putaway_weight: <integer>
    replen_weight: <integer>
```

---

## Task Allocation

Determines which robot receives each task.

### nearest_robot

Assigns task to the closest available robot.

```yaml
policies:
  task_allocation:
    type: nearest_robot
```

**Pros**: Simple, minimizes immediate travel
**Cons**: Can lead to uneven workload

### auction

Robots bid based on weighted factors.

```yaml
policies:
  task_allocation:
    type: auction
    travel_weight: 1.0
    queue_weight: 0.5
```

| Parameter | Default | Description |
|-----------|---------|-------------|
| `travel_weight` | 1.0 | Weight for travel distance |
| `queue_weight` | 0.5 | Weight for station queue length |

**Score** = travel_weight × distance + queue_weight × queue_length

Lower score wins.

### workload_balanced

Balances workload across robots.

```yaml
policies:
  task_allocation:
    type: workload_balanced
```

**Pros**: Even utilization
**Cons**: May increase travel distance

---

## Station Assignment

Determines which station handles each task.

### least_queue

Assigns to station with shortest queue.

```yaml
policies:
  station_assignment:
    type: least_queue
```

**Pros**: Balances load across stations
**Cons**: Doesn't consider travel distance

### fastest_service

Assigns to station that will complete service soonest.

```yaml
policies:
  station_assignment:
    type: fastest_service
```

**Considers**: Current queue + service times

### due_time_priority

Prioritizes stations based on order due times.

```yaml
policies:
  station_assignment:
    type: due_time_priority
```

**Best for**: Scenarios with strict SLAs

---

## Batching

Controls how items are grouped for picking.

### none

No batching; each task is independent.

```yaml
policies:
  batching:
    type: none
```

### station_batch

Batch items going to the same station.

```yaml
policies:
  batching:
    type: station_batch
    max_items: 20
    max_weight_kg: 25.0
```

| Parameter | Description |
|-----------|-------------|
| `max_items` | Maximum items per batch |
| `max_weight_kg` | Maximum weight per batch |

### zone_batch

Batch items from the same zone.

```yaml
policies:
  batching:
    type: zone_batch
    max_items: 15
```

---

## Priority Arbitration

Determines how different task types are prioritized.

### strict_priority

Fixed priority order: pick > replenishment > putaway.

```yaml
policies:
  priority:
    type: strict_priority
```

### weighted_fair

Weighted round-robin across task types.

```yaml
policies:
  priority:
    type: weighted_fair
    pick_weight: 3
    putaway_weight: 1
    replen_weight: 2
```

| Parameter | Default | Description |
|-----------|---------|-------------|
| `pick_weight` | 1 | Weight for pick tasks |
| `putaway_weight` | 1 | Weight for putaway tasks |
| `replen_weight` | 1 | Weight for replenishment tasks |

### sla_driven

Prioritizes based on order due times.

```yaml
policies:
  priority:
    type: sla_driven
```

---

## Complete Examples

### Basic Configuration

```yaml
policies:
  task_allocation:
    type: nearest_robot
  station_assignment:
    type: least_queue
  batching:
    type: none
  priority:
    type: strict_priority
```

### High-Throughput Configuration

```yaml
policies:
  task_allocation:
    type: auction
    travel_weight: 1.0
    queue_weight: 0.8
  station_assignment:
    type: fastest_service
  batching:
    type: station_batch
    max_items: 25
    max_weight_kg: 30.0
  priority:
    type: strict_priority
```

### SLA-Focused Configuration

```yaml
policies:
  task_allocation:
    type: workload_balanced
  station_assignment:
    type: due_time_priority
  batching:
    type: none
  priority:
    type: sla_driven
```

### Balanced Multi-Flow

```yaml
policies:
  task_allocation:
    type: auction
    travel_weight: 1.0
    queue_weight: 0.5
  station_assignment:
    type: least_queue
  batching:
    type: zone_batch
    max_items: 20
  priority:
    type: weighted_fair
    pick_weight: 4
    putaway_weight: 2
    replen_weight: 3
```

---

## Policy Selection Guide

### Task Allocation

| Scenario | Recommended |
|----------|-------------|
| Simple, small warehouse | `nearest_robot` |
| High congestion | `auction` |
| Uneven workload | `workload_balanced` |

### Station Assignment

| Scenario | Recommended |
|----------|-------------|
| Balanced stations | `least_queue` |
| Variable service times | `fastest_service` |
| Strict SLAs | `due_time_priority` |

### Batching

| Scenario | Recommended |
|----------|-------------|
| Individual orders | `none` |
| High item density | `station_batch` |
| Zone-based layout | `zone_batch` |

### Priority

| Scenario | Recommended |
|----------|-------------|
| Pick-only operations | `strict_priority` |
| Mixed operations | `weighted_fair` |
| SLA requirements | `sla_driven` |

---

## Related

- [Policy Concepts](../concepts/policies/index.md)
- [Task Allocation Details](../concepts/policies/task-allocation.md)
- [Station Assignment Details](../concepts/policies/station-assignment.md)
