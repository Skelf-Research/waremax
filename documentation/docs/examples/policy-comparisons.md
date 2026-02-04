# Policy Comparisons

Examples comparing different policy configurations.

---

## Task Allocation Comparison

Compare how different task allocation policies affect performance.

### Setup

```yaml
# base_scenario.yaml (common configuration)
simulation:
  duration_s: 3600

robots:
  count: 10
  speed_m_s: 1.5

stations:
  - { id: S1, node: 30, type: pick, concurrency: 2 }
  - { id: S2, node: 31, type: pick, concurrency: 2 }

orders:
  generation:
    type: poisson
    rate_per_hour: 300
```

### Run Comparison

```bash
waremax compare base_scenario.yaml \
  --param policies.task_allocation=nearest_idle \
  --param policies.task_allocation=least_busy \
  --param policies.task_allocation=round_robin \
  --runs 5
```

### Expected Results

| Policy | Throughput | Utilization Spread | Avg Travel |
|--------|------------|-------------------|------------|
| nearest_idle | 295/hr | 15% | 12.3s |
| least_busy | 280/hr | 5% | 15.8s |
| round_robin | 265/hr | 2% | 18.2s |

### Analysis

- **nearest_idle**: Best throughput, some robots work more than others
- **least_busy**: Balanced workload, slightly longer travel
- **round_robin**: Most balanced but ignores efficiency

---

## Station Assignment Comparison

Compare station selection strategies.

### Setup

```yaml
# Unbalanced stations
stations:
  - { id: S1, node: 30, type: pick, concurrency: 1 }  # Closer
  - { id: S2, node: 31, type: pick, concurrency: 2 }  # Further but larger
```

### Run Comparison

```bash
waremax compare base_scenario.yaml \
  --param policies.station_assignment=nearest \
  --param policies.station_assignment=shortest_queue \
  --param policies.station_assignment=fastest_completion
```

### Expected Results

| Policy | Throughput | S1 Util | S2 Util | Avg Queue |
|--------|------------|---------|---------|-----------|
| nearest | 250/hr | 95% | 45% | 5.2 |
| shortest_queue | 285/hr | 72% | 78% | 2.1 |
| fastest_completion | 295/hr | 68% | 82% | 1.8 |

### Analysis

- **nearest**: Creates imbalance, S1 overloaded
- **shortest_queue**: Better balance
- **fastest_completion**: Best throughput, considers all factors

---

## Routing Policy Comparison

Compare pathfinding strategies under congestion.

### Setup

```yaml
robots:
  count: 20  # Higher density for congestion

traffic:
  congestion_response: reroute
```

### Run Comparison

```bash
waremax compare congestion_scenario.yaml \
  --param routing.policy=shortest_path \
  --param "routing.policy=congestion_aware,routing.congestion_weight=1.0" \
  --param "routing.policy=congestion_aware,routing.congestion_weight=2.0"
```

### Expected Results

| Policy | Throughput | Avg Wait | Avg Travel | Deadlocks |
|--------|------------|----------|------------|-----------|
| shortest_path | 380/hr | 12.5s | 14.2s | 3 |
| congestion_aware (1.0) | 420/hr | 6.8s | 16.1s | 0 |
| congestion_aware (2.0) | 410/hr | 4.2s | 18.5s | 0 |

### Analysis

- **shortest_path**: Direct routes but gets congested
- **congestion_aware (1.0)**: Good balance
- **congestion_aware (2.0)**: Very few waits but longer paths

---

## Batching Comparison

Compare batching vs. individual task execution.

### Setup

```yaml
orders:
  generation:
    type: poisson
    rate_per_hour: 400
```

### Run Comparison

```bash
waremax compare base_scenario.yaml \
  --param policies.batching.enabled=false \
  --param "policies.batching.enabled=true,policies.batching.max_batch_size=3" \
  --param "policies.batching.enabled=true,policies.batching.max_batch_size=5"
```

### Expected Results

| Config | Throughput | Travel/Task | Task Latency |
|--------|------------|-------------|--------------|
| No batching | 380/hr | 15.2s | 42s |
| Batch 3 | 410/hr | 11.8s | 48s |
| Batch 5 | 430/hr | 9.5s | 55s |

### Analysis

- Batching improves throughput but increases individual task latency
- Batch size 3-5 is typically optimal
- Trade-off depends on latency requirements

---

## Combined Policy Optimization

Find the best combination of policies.

### Sweep Configuration

```bash
waremax sweep base_scenario.yaml \
  --param "policies.task_allocation=[nearest_idle,least_busy]" \
  --param "policies.station_assignment=[shortest_queue,fastest_completion]" \
  --param "routing.congestion_weight=[1.0,1.5,2.0]" \
  --runs 3
```

### Top Configurations

| Rank | Allocation | Station | Weight | Throughput |
|------|------------|---------|--------|------------|
| 1 | nearest_idle | fastest_completion | 1.5 | 435/hr |
| 2 | nearest_idle | fastest_completion | 1.0 | 428/hr |
| 3 | nearest_idle | shortest_queue | 1.5 | 420/hr |
| 4 | least_busy | fastest_completion | 1.5 | 415/hr |

### Recommended Configuration

```yaml
policies:
  task_allocation: nearest_idle
  station_assignment: fastest_completion

routing:
  policy: congestion_aware
  congestion_weight: 1.5
```

---

## Running Your Own Comparisons

### Template Script

```bash
#!/bin/bash
# policy_comparison.sh

BASE_SCENARIO="scenario.yaml"
OUTPUT_DIR="comparison_results"

# Run comparisons
waremax compare $BASE_SCENARIO \
  --param policies.task_allocation=nearest_idle \
  --param policies.task_allocation=least_busy \
  --runs 5 \
  -o $OUTPUT_DIR/task_allocation/

# Analyze
waremax analyze $OUTPUT_DIR/task_allocation/

# Generate report
waremax analyze $OUTPUT_DIR/task_allocation/ \
  --export comparison_report.csv
```

---

## Related

- [Tuning Policies](../tutorials/config/tuning-policies.md)
- [A/B Testing](../tutorials/testing/ab-testing.md)
- [Policy Configuration](../configuration/policies.md)
