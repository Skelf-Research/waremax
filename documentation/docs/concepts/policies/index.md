# Policies

Decision-making rules that control system behavior.

---

## Overview

Policies define how the system makes decisions. They're configurable rules that determine behavior without changing code.

| Topic | Description |
|-------|-------------|
| [Task Allocation](task-allocation.md) | Assigning tasks to robots |
| [Station Assignment](station-assignment.md) | Selecting stations for tasks |
| [Batching](batching.md) | Grouping tasks for efficiency |
| [Priority](priority.md) | Task and robot priority rules |
| [Traffic Policies](traffic-policies.md) | Congestion and routing decisions |

---

## Why Policies?

### Flexibility

Same system, different behaviors:

```yaml
# Configuration A: Speed-focused
policies:
  task_allocation: nearest_idle

# Configuration B: Balance-focused
policies:
  task_allocation: least_busy
```

### Experimentation

Compare approaches easily:

```bash
waremax compare scenario.yaml \
  --param policies.task_allocation=nearest_idle \
  --param policies.task_allocation=least_busy
```

### Optimization

Find best policy for your warehouse:

```bash
waremax sweep scenario.yaml \
  --param "policies.task_allocation=[nearest_idle,least_busy,round_robin]"
```

---

## Policy Categories

### Operational Policies

Control day-to-day operations:

| Policy | Decisions |
|--------|-----------|
| Task allocation | Which robot gets which task |
| Station assignment | Which station serves which task |
| Batching | How to group tasks |

### Traffic Policies

Control robot movement:

| Policy | Decisions |
|--------|-----------|
| Routing | Which path to take |
| Congestion response | Wait or reroute |
| Deadlock resolution | How to break deadlocks |

### Resource Policies

Control resource usage:

| Policy | Decisions |
|--------|-----------|
| Charging | When to charge |
| Maintenance | When to maintain |
| Station selection | Which charging/maintenance station |

---

## Policy Structure

### Configuration Format

```yaml
policies:
  category:
    policy_name: value
    # or
    policy_name:
      type: complex_type
      param1: value1
      param2: value2
```

### Example

```yaml
policies:
  task_allocation:
    type: nearest_idle
  station_assignment:
    type: shortest_queue
  batching:
    enabled: true
    max_batch_size: 5
```

---

## Policy Selection

### Factors to Consider

| Factor | Consideration |
|--------|---------------|
| Throughput | Which maximizes tasks/hour |
| Latency | Which minimizes task time |
| Fairness | Which balances robot utilization |
| Robustness | Which handles variability |

### Trade-offs

```
          Throughput
              ↑
              │     ●A
              │
              │ ●B
              │        ●C
              │
              └──────────────→ Fairness

A: High throughput, low fairness
B: Balanced
C: High fairness, lower throughput
```

---

## Policy Comparison

### Using Compare Command

```bash
waremax compare base.yaml \
  --param policies.task_allocation=nearest_idle \
  --param policies.task_allocation=least_busy
```

### Key Metrics

- Tasks completed
- Average task time
- Robot utilization spread
- Wait time

---

## Configuration Reference

### Task Allocation

```yaml
policies:
  task_allocation:
    type: nearest_idle | least_busy | round_robin
```

### Station Assignment

```yaml
policies:
  station_assignment:
    type: nearest | shortest_queue | fastest_completion
```

### Routing

```yaml
policies:
  routing:
    type: shortest_path | congestion_aware
    congestion_weight: 1.5
```

### Batching

```yaml
policies:
  batching:
    enabled: true
    max_batch_size: 5
    max_wait_time_s: 30.0
```

---

## Best Practices

### Start Simple

Begin with default policies:

```yaml
policies:
  task_allocation: nearest_idle
  station_assignment: nearest
```

### Measure Impact

Always compare before and after:

```bash
# Baseline
waremax run baseline.yaml -o baseline_results/

# With new policy
waremax run new_policy.yaml -o new_policy_results/

# Compare
waremax analyze baseline_results/ new_policy_results/
```

### Consider Interactions

Policies interact with each other:

- Task allocation affects traffic patterns
- Traffic patterns affect routing effectiveness
- Routing affects task completion times

---

## Related

- [Policy Configuration](../../configuration/policies.md)
- [A/B Testing](../../cli/ab-test.md)
- [Parameter Sweeps](../../cli/sweep.md)
