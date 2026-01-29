# Dispatching Policies

Policies in Waremax are modular and configurable. This allows side-by-side comparisons in the same layout and workload.

## Task Allocation

Decides which robot takes a task.

- **Nearest Robot**: choose the robot with minimum ETA to pickup.
- **Auction**: each robot bids with ETA plus penalties (queue, battery, congestion).
- **Workload Balanced**: minimize `ETA + lambda * tasks_in_queue`.

## Station Assignment

Chooses where a task should be delivered.

- **Least Queue**: pick station with the smallest predicted queue.
- **Fastest Service**: route tasks to the station with best service capacity.
- **Due Time Priority**: route based on earliest due time.

## Batching

Combines tasks into bundles when it reduces travel and station handling overhead.

- **No Batch**: every order line is an independent task.
- **Station Batch**: bundle tasks to the same station up to a max item count.
- **Zone Batch**: bundle tasks within a zone to reduce travel distance.

Batching must respect payload limits and station constraints.

## Priority Arbitration

If pick, putaway, and replenishment (abbreviated as `replen` in config) compete for the same fleet, a priority policy decides ordering.

- **Strict Priority**: pick > replen > putaway.
- **Weighted Fair**: split capacity by weights across job types.
- **SLA Driven**: prioritize by due time risk.

## Policy Configuration Reference

### Task Allocation Options

```yaml
# Option 1: Nearest Robot (default)
task_allocation:
  type: nearest_robot

# Option 2: Auction-based
task_allocation:
  type: auction
  eta_weight: 1.0
  queue_penalty: 0.5
  battery_penalty: 0.3      # Only used if battery is enabled
  congestion_penalty: 0.2

# Option 3: Workload Balanced
task_allocation:
  type: workload_balanced
  lambda: 0.5               # Weight for tasks_in_queue term
```

### Station Assignment Options

```yaml
# Option 1: Least Queue (default)
station_assignment:
  type: least_queue

# Option 2: Fastest Service
station_assignment:
  type: fastest_service

# Option 3: Due Time Priority
station_assignment:
  type: due_time_priority
  slack_threshold_s: 300    # Route to faster station if slack < threshold
```

### Batching Options

```yaml
# Option 1: No Batching
batching:
  type: none

# Option 2: Station Batch (default)
batching:
  type: station_batch
  max_items: 10
  max_weight_kg: 20         # Optional, must respect robot payload

# Option 3: Zone Batch
batching:
  type: zone_batch
  max_items: 8
  max_weight_kg: 20
  zone_affinity: true       # Prefer tasks in same zone
```

### Priority Arbitration Options

```yaml
# Option 1: Strict Priority
priority:
  type: strict_priority
  order: [pick, replen, putaway]  # Higher priority first

# Option 2: Weighted Fair (default)
priority:
  type: weighted_fair
  weights:
    pick: 0.7
    replen: 0.2
    putaway: 0.1

# Option 3: SLA Driven
priority:
  type: sla_driven
  late_threshold_s: 600     # Consider task late if due within threshold
  late_boost: 2.0           # Priority multiplier for late tasks
```

## Complete Policy Configuration Example

```yaml
policies:
  task_allocation:
    type: nearest_robot
  station_assignment:
    type: least_queue
  batching:
    type: station_batch
    max_items: 10
  priority:
    type: weighted_fair
    weights:
      pick: 0.7
      replen: 0.2
      putaway: 0.1
```
