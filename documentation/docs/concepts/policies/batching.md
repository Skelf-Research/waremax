# Batching

Grouping tasks for improved efficiency.

---

## What is Batching?

Batching combines multiple tasks into a single unit, allowing a robot to complete several tasks in one trip.

### Without Batching

```
Task 1: Pick SKU-A from R1 → Station S1
Task 2: Pick SKU-B from R2 → Station S1
Task 3: Pick SKU-C from R3 → Station S1

Robot trips: 3 (one per task)
```

### With Batching

```
Batch: [Task 1, Task 2, Task 3]

Robot trip: R1 → R2 → R3 → S1

Robot trips: 1 (all tasks together)
```

---

## Why Batch?

### Reduced Travel

```
Without batching:
  Travel = 3 × (rack to station) = 45m

With batching:
  Travel = R1→R2 + R2→R3 + R3→S1 = 25m

Savings: 44%
```

### Higher Throughput

- Fewer trips = more capacity
- Less congestion from robot movements
- Better robot utilization

### Trade-off

Batching delays individual tasks:

```
Without batching:
  Task 1: Complete at t=15

With batching (wait for batch):
  Task 1: Complete at t=30

Individual task delayed, but total work is faster
```

---

## Batching Strategies

### Order-Based Batching

Group tasks from same order:

```yaml
policies:
  batching:
    type: order_based
```

```
Order O1: SKU-A, SKU-B, SKU-C

Batch = [Pick SKU-A, Pick SKU-B, Pick SKU-C]
```

**Pros:**

- Natural grouping
- Completes orders faster

**Cons:**

- Small orders don't benefit
- May wait for large orders

---

### Zone-Based Batching

Group tasks from same zone:

```yaml
policies:
  batching:
    type: zone_based
    zones:
      zone_a: [R1, R2, R3]
      zone_b: [R4, R5, R6]
```

```
Tasks in Zone A:
  Pick from R1, Pick from R2, Pick from R3

Batch = All Zone A picks
```

**Pros:**

- Minimizes travel within batch
- Works across orders

**Cons:**

- Orders may span zones
- Requires zone definition

---

### Proximity-Based Batching

Group nearby tasks:

```yaml
policies:
  batching:
    type: proximity
    max_distance: 10.0
```

```
Task 1: R1 (position 5,5)
Task 2: R2 (position 6,7)
Task 3: R5 (position 50,50)

Batch 1 = [Task 1, Task 2] (close)
Batch 2 = [Task 3] (far from others)
```

**Pros:**

- Optimizes travel dynamically
- No predefined zones needed

**Cons:**

- More complex to compute
- Batch composition varies

---

### Time-Window Batching

Group tasks arriving within time window:

```yaml
policies:
  batching:
    type: time_window
    window_s: 30.0
```

```
t=0:  Task 1 arrives
t=10: Task 2 arrives
t=20: Task 3 arrives
t=30: Window closes

Batch = [Task 1, Task 2, Task 3]
```

**Pros:**

- Simple to implement
- Predictable batching

**Cons:**

- Delays early tasks
- Fixed window may not match demand

---

## Batch Parameters

### Maximum Batch Size

```yaml
policies:
  batching:
    max_batch_size: 5
```

Limits items per batch:

```
Queue: 8 tasks
Max size: 5

Batch 1: 5 tasks
Batch 2: 3 tasks
```

### Maximum Wait Time

```yaml
policies:
  batching:
    max_wait_time_s: 60.0
```

Don't wait too long to fill batch:

```
t=0:  Task 1 arrives (batch started)
t=60: Timer expires (batch released with 2 tasks)
```

### Minimum Batch Size

```yaml
policies:
  batching:
    min_batch_size: 2
    min_wait_time_s: 10.0
```

Wait for minimum before releasing.

---

## Batch Execution

### Sequence Planning

Optimize visit order within batch:

```
Batch: [R3, R1, R5]

Nearest neighbor sequence:
  Start → R1 → R3 → R5 → Station

Better than: Start → R3 → R1 → R5 → Station
```

### Partial Completion

Handle failures gracefully:

```yaml
policies:
  batching:
    partial_completion: true
```

If one pick fails, continue with rest.

---

## Configuration Example

```yaml
policies:
  batching:
    enabled: true
    type: proximity

    # Size limits
    max_batch_size: 6
    min_batch_size: 2

    # Time limits
    max_wait_time_s: 45.0
    min_wait_time_s: 10.0

    # Proximity threshold
    max_distance: 15.0

    # Execution
    sequence_optimization: nearest_neighbor
    partial_completion: true
```

---

## Batching Metrics

### Batch Statistics

| Metric | Description |
|--------|-------------|
| Avg batch size | Mean tasks per batch |
| Batch fill rate | Actual / Max size |
| Wait time for batching | Delay before batch starts |

### Efficiency Metrics

| Metric | Description |
|--------|-------------|
| Travel per task | Total travel / Tasks |
| Throughput | Tasks per hour |
| Travel savings | vs. non-batched |

---

## When to Batch

### Good for Batching

- High task volume
- Tasks with common destinations
- Spatially clustered tasks
- Non-urgent tasks

### Avoid Batching

- Time-critical tasks
- Sparse task arrivals
- Random spatial distribution
- Very low volume

---

## Testing Batching

### Compare With/Without

```bash
waremax compare scenario.yaml \
  --param policies.batching.enabled=false \
  --param policies.batching.enabled=true
```

### Tune Parameters

```bash
waremax sweep scenario.yaml \
  --param "policies.batching.max_batch_size=[3,5,7,10]" \
  --param "policies.batching.max_wait_time_s=[30,60,90]"
```

### Metrics to Watch

- Travel per task (lower = better batching)
- Task latency (watch for excessive delays)
- Throughput (should improve)

---

## Related

- [Task Allocation](task-allocation.md)
- [Policy Configuration](../../configuration/policies.md)
- [Orders](../../configuration/orders.md)
