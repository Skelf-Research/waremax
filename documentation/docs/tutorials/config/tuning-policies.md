# Tuning Policies

Optimize system behavior through policy configuration.

---

## Goal

By the end of this tutorial, you will:

- Compare different policy options
- Systematically find optimal policies
- Balance trade-offs between metrics
- Document your tuning decisions

**Time**: 45-60 minutes

---

## Prerequisites

- Completed [Creating Scenarios](../basic/creating-scenarios.md)
- Completed [Analyzing Results](../basic/analyzing-results.md)

---

## Step 1: Establish Baseline

Start with default policies:

```yaml
# baseline.yaml
policies:
  task_allocation: nearest_idle
  station_assignment: nearest
```

Run and save results:

```bash
waremax run baseline.yaml -o baseline_results/
waremax analyze baseline_results/
```

Record baseline metrics:

```
Baseline:
  Throughput: 850 tasks/hour
  Avg task time: 48.2s
  Robot utilization: 72%
  Station S1 queue: 4.2 avg
```

---

## Step 2: Compare Task Allocation Policies

Test different allocation strategies:

```bash
waremax compare baseline.yaml \
  --param policies.task_allocation=nearest_idle \
  --param policies.task_allocation=least_busy \
  --param policies.task_allocation=round_robin
```

**Results:**

```
                    nearest_idle  least_busy  round_robin
─────────────────────────────────────────────────────────
Throughput/hr       850           820         780
Avg task time       48.2s         52.1s       58.3s
Robot utilization   72%           75%         76%
Utilization spread  12%           5%          2%
```

**Analysis:**

- `nearest_idle`: Highest throughput, some robot imbalance
- `least_busy`: More balanced, slightly lower throughput
- `round_robin`: Most balanced, lowest throughput

**Decision:** Keep `nearest_idle` for throughput focus.

---

## Step 3: Compare Station Assignment

Test station assignment policies:

```bash
waremax compare baseline.yaml \
  --param policies.station_assignment=nearest \
  --param policies.station_assignment=shortest_queue \
  --param policies.station_assignment=fastest_completion
```

**Results:**

```
                    nearest    shortest_queue  fastest_completion
────────────────────────────────────────────────────────────────
Throughput/hr       850        890             905
Avg task time       48.2s      45.1s           43.8s
S1 utilization      92%        78%             75%
S2 utilization      58%        74%             77%
```

**Analysis:**

- `nearest`: Creates imbalanced station load
- `shortest_queue`: Better balance
- `fastest_completion`: Best throughput and balance

**Decision:** Switch to `fastest_completion`.

---

## Step 4: Update Configuration

Apply improvements:

```yaml
# optimized.yaml
policies:
  task_allocation: nearest_idle
  station_assignment: fastest_completion
```

Verify improvement:

```bash
waremax run optimized.yaml -o optimized_results/
waremax analyze baseline_results/ optimized_results/
```

```
Improvement:
  Throughput: 850 → 905 (+6.5%)
  Avg task time: 48.2s → 43.8s (-9.1%)
```

---

## Step 5: Tune Routing

Compare routing policies:

```bash
waremax compare optimized.yaml \
  --param routing.policy=shortest_path \
  --param routing.policy=congestion_aware
```

For congestion-aware, also tune the weight:

```bash
waremax sweep optimized.yaml \
  --param routing.policy=congestion_aware \
  --param "routing.congestion_weight=[0.5,1.0,1.5,2.0,2.5]"
```

**Results:**

```
congestion_weight  Throughput  Avg Wait Time  Avg Travel
0.5                905         8.2s           15.1s
1.0                918         6.8s           16.2s
1.5                925         5.9s           17.5s
2.0                920         5.5s           19.1s
2.5                908         5.3s           21.2s
```

**Analysis:**

- Weight 1.5 gives best throughput
- Higher weights reduce wait but increase travel

**Decision:** Use `congestion_weight: 1.5`.

---

## Step 6: Tune Traffic Policies

Configure congestion response:

```yaml
traffic:
  congestion_response: hybrid
  wait_threshold_s: 3.0
  reroute_threshold_s: 10.0
```

Test different thresholds:

```bash
waremax sweep optimized.yaml \
  --param "traffic.reroute_threshold_s=[5,10,15,20]"
```

---

## Step 7: Tune Batching

If using batching, optimize parameters:

```yaml
policies:
  batching:
    enabled: true
    max_batch_size: 5
    max_wait_time_s: 30.0
```

Sweep batch parameters:

```bash
waremax sweep optimized.yaml \
  --param "policies.batching.max_batch_size=[3,5,7,10]" \
  --param "policies.batching.max_wait_time_s=[15,30,45,60]"
```

---

## Step 8: Multi-Factor Sweep

Optimize multiple parameters together:

```bash
waremax sweep optimized.yaml \
  --param "policies.task_allocation=[nearest_idle,least_busy]" \
  --param "policies.station_assignment=[shortest_queue,fastest_completion]" \
  --param "routing.congestion_weight=[1.0,1.5,2.0]" \
  --runs 3
```

This tests 2 × 2 × 3 = 12 combinations.

**Output:**

```
Rank  Allocation    Station         Weight  Throughput
1     nearest_idle  fastest_comp    1.5     928
2     nearest_idle  fastest_comp    1.0     920
3     nearest_idle  shortest_queue  1.5     915
4     least_busy    fastest_comp    1.5     912
...
```

---

## Step 9: Validate Final Configuration

Create final configuration:

```yaml
# final_optimized.yaml
policies:
  task_allocation: nearest_idle
  station_assignment: fastest_completion
  batching:
    enabled: true
    max_batch_size: 5
    max_wait_time_s: 30.0

routing:
  policy: congestion_aware
  congestion_weight: 1.5

traffic:
  congestion_response: hybrid
  wait_threshold_s: 3.0
  reroute_threshold_s: 10.0
```

Run comprehensive test:

```bash
waremax run final_optimized.yaml -o final_results/ --runs 5
```

---

## Step 10: Document Decisions

Create a tuning log:

```markdown
# Policy Tuning Log

## Baseline
- Throughput: 850/hr
- Task time: 48.2s

## Task Allocation
Tested: nearest_idle, least_busy, round_robin
Selected: nearest_idle (best throughput)

## Station Assignment
Tested: nearest, shortest_queue, fastest_completion
Selected: fastest_completion (+6.5% throughput)

## Routing
Tested: congestion_weight 0.5-2.5
Selected: 1.5 (best balance)

## Final Results
- Throughput: 928/hr (+9.2%)
- Task time: 41.5s (-13.9%)
```

---

## Trade-off Guidelines

### Throughput vs. Balance

```
Higher throughput often means some imbalance
If balance matters: Use least_busy + shortest_queue
If throughput matters: Use nearest_idle + fastest_completion
```

### Speed vs. Congestion

```
Aggressive routing (low congestion_weight):
  - Shorter paths
  - More congestion risk

Conservative routing (high congestion_weight):
  - Longer paths
  - Less congestion
```

### Batching Trade-offs

```
Large batches:
  + More efficient travel
  - Longer individual task times

Small batches:
  + Faster individual tasks
  - Less travel efficiency
```

---

## Next Steps

- [A/B Testing](../testing/ab-testing.md): Statistically compare policies
- [Parameter Sweeps](../testing/parameter-sweeps.md): Systematic exploration
- [Policy Configuration](../../configuration/policies.md): Full reference
