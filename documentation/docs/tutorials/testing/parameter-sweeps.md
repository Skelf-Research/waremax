# Parameter Sweeps

Systematically explore configuration parameter space.

---

## Goal

By the end of this tutorial, you will:

- Use the sweep command effectively
- Design sweep experiments
- Analyze sweep results
- Find optimal configurations

**Time**: 45 minutes

---

## Prerequisites

- Completed [Analyzing Results](../basic/analyzing-results.md)
- Understanding of scenario configuration

---

## Step 1: Basic Sweep

Sweep a single parameter:

```bash
waremax sweep scenario.yaml \
  --param "robots.count=[5,10,15,20,25]"
```

**Output:**

```
Sweep Configuration:
  Parameter: robots.count
  Values: [5, 10, 15, 20, 25]
  Total runs: 5

Running sweeps...
  [1/5] robots.count=5  ━━━━━━━━━━ 100%
  [2/5] robots.count=10 ━━━━━━━━━━ 100%
  [3/5] robots.count=15 ━━━━━━━━━━ 100%
  [4/5] robots.count=20 ━━━━━━━━━━ 100%
  [5/5] robots.count=25 ━━━━━━━━━━ 100%

Results:
  robots.count  Throughput  Avg Time  Utilization
  5             420         52.3s     89%
  10            780         45.1s     82%
  15            980         42.0s     74%
  20            1,050       43.5s     65%
  25            1,020       48.2s     55%

Best throughput: robots.count=20 (1,050/hr)
```

---

## Step 2: Multi-Parameter Sweep

Sweep multiple parameters together:

```bash
waremax sweep scenario.yaml \
  --param "robots.count=[10,15,20]" \
  --param "robots.speed_m_s=[1.0,1.5,2.0]"
```

This creates a grid: 3 × 3 = 9 combinations.

**Output:**

```
Sweep Grid:
  robots.count: [10, 15, 20]
  robots.speed_m_s: [1.0, 1.5, 2.0]
  Total combinations: 9

Results Matrix:
                  speed=1.0  speed=1.5  speed=2.0
  count=10        620        780        890
  count=15        750        980        1,100
  count=20        820        1,050      1,150
```

---

## Step 3: Range Syntax

Use ranges for numeric parameters:

```bash
# Explicit list
--param "robots.count=[5,10,15,20,25]"

# Range: start..end..step
--param "robots.count=5..25..5"

# Logarithmic scale
--param "order_rate=[100,200,400,800,1600]"
```

---

## Step 4: Multiple Runs per Configuration

Account for randomness with multiple seeds:

```bash
waremax sweep scenario.yaml \
  --param "robots.count=[10,15,20]" \
  --runs 5
```

**Output includes statistics:**

```
robots.count  Throughput (mean ± std)
10            780 ± 25
15            975 ± 32
20            1,048 ± 28
```

---

## Step 5: Save Detailed Results

Save all results for later analysis:

```bash
waremax sweep scenario.yaml \
  --param "robots.count=5..25..5" \
  -o sweep_results/
```

Creates:

```
sweep_results/
├── summary.json
├── config_0/          # robots.count=5
│   ├── summary.json
│   └── events.jsonl
├── config_1/          # robots.count=10
│   └── ...
└── ...
```

---

## Step 6: Analyze Sweep Results

Use analyze on sweep output:

```bash
waremax analyze sweep_results/ --sweep
```

**Output:**

```
=== Sweep Analysis ===

Parameter: robots.count
Range: 5 to 25

Metric Trends:
  Throughput:
    5→10:  +86% ↑
    10→15: +26% ↑
    15→20: +7%  ↑
    20→25: -3%  ↓  ← Diminishing returns

  Utilization:
    5→10:  -8%
    10→15: -10%
    15→20: -12%
    20→25: -15%

Optimal Points:
  Max throughput: robots.count=20 (1,050/hr)
  Best efficiency: robots.count=15 (65 tasks/robot/hr)

Recommendation:
  Use 15-20 robots for best balance of throughput and utilization.
```

---

## Step 7: Multi-Objective Sweep

Find Pareto-optimal configurations:

```bash
waremax sweep scenario.yaml \
  --param "robots.count=5..30..5" \
  --param "stations[0].concurrency=[1,2,3]" \
  --objectives throughput:max utilization:target:0.75
```

**Output:**

```
Pareto-Optimal Configurations:
                      Throughput  Utilization
1. count=15, conc=2   1,020       76%
2. count=20, conc=2   1,150       68%
3. count=15, conc=3   1,080       72%

Closest to target (util=75%):
  robots.count=15, stations[0].concurrency=2
```

---

## Step 8: Conditional Sweeps

Sweep one parameter based on another:

```bash
# Station concurrency should scale with robot count
waremax sweep scenario.yaml \
  --param "robots.count=[10,20,30]" \
  --param-expr "stations[0].concurrency=robots.count/10"
```

---

## Designing Good Sweeps

### Start Coarse, Refine

```bash
# Step 1: Coarse sweep to find region
waremax sweep scenario.yaml \
  --param "robots.count=[5,15,25,35,45]"

# Output: Best around 25

# Step 2: Fine sweep around best
waremax sweep scenario.yaml \
  --param "robots.count=20..30..2"
```

### Cover Key Scenarios

```yaml
# Sweep plan
1. Fleet sizing: robots.count
2. Station capacity: stations[*].concurrency
3. Traffic management: routing.congestion_weight
4. Order rate stress: orders.rate_per_hour
```

### Resource Constraints

Large sweeps take time:

```
Combinations = Π(parameter values)
Time = Combinations × Single run time × Runs per config

Example:
  3 parameters × 5 values each = 125 combinations
  5 minutes per run × 3 runs = 15 minutes per config
  Total: 125 × 15 = 1,875 minutes (31+ hours)
```

Use parallel execution:

```bash
waremax sweep scenario.yaml \
  --param "robots.count=[10,15,20]" \
  --parallel 4
```

---

## Common Sweep Patterns

### Capacity Planning

```bash
waremax sweep scenario.yaml \
  --param "robots.count=5..50..5" \
  --param "orders.rate_per_hour=[500,750,1000,1250]"
```

### Policy Comparison

```bash
waremax sweep scenario.yaml \
  --param "policies.task_allocation=[nearest_idle,least_busy,round_robin]" \
  --param "policies.station_assignment=[nearest,shortest_queue,fastest_completion]"
```

### Sensitivity Analysis

```bash
waremax sweep scenario.yaml \
  --param "robots.speed_m_s=1.0..2.0..0.1" \
  --metric throughput \
  --analysis sensitivity
```

---

## Example: Finding Optimal Fleet Size

```bash
# Step 1: Wide range
waremax sweep scenario.yaml \
  --param "robots.count=5..50..5" \
  --runs 3

# Results show optimal around 25-30

# Step 2: Narrow range
waremax sweep scenario.yaml \
  --param "robots.count=22..32..2" \
  --runs 5

# Results: optimal at 28

# Step 3: Verify with longer simulation
waremax run scenario.yaml \
  --param robots.count=28 \
  --duration 14400 \
  --runs 10
```

---

## Next Steps

- [A/B Testing](ab-testing.md): Statistical comparison
- [Benchmarking](benchmarking.md): Performance limits
- [Sweep Command Reference](../../cli/sweep.md)
