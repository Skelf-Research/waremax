# Benchmarking

Measure performance limits and stress test configurations.

---

## Goal

By the end of this tutorial, you will:

- Find maximum throughput capacity
- Identify breaking points
- Stress test configurations
- Create performance baselines

**Time**: 30 minutes

---

## Prerequisites

- Completed [Parameter Sweeps](parameter-sweeps.md)
- Understanding of capacity concepts

---

## Step 1: Basic Benchmark

Run a standardized benchmark:

```bash
waremax benchmark scenario.yaml
```

**Output:**

```
=== Benchmark Results ===

Configuration:
  Scenario: scenario.yaml
  Duration: 3600s
  Runs: 5

Performance:
  Throughput: 1,050 ± 32 tasks/hr
  P50 task time: 38.2s
  P95 task time: 72.5s
  P99 task time: 95.1s

Resource Utilization:
  Robot utilization: 78%
  Station utilization: 82%
  Peak queue length: 8

Stability:
  Variance coefficient: 3.0%
  Steady state reached: Yes (after 180s)

Rating: ████████░░ Good
```

---

## Step 2: Find Maximum Throughput

Increase load until system saturates:

```bash
waremax benchmark scenario.yaml \
  --find-max-throughput \
  --step 100
```

**Output:**

```
Finding Maximum Throughput...

Order Rate  Throughput  Utilization  Stable?
500         500         52%          ✓
600         598         61%          ✓
700         695         70%          ✓
800         788         79%          ✓
900         865         86%          ✓
1000        912         91%          ⚠
1100        935         94%          ✗
1200        928         97%          ✗

Maximum sustainable throughput: ~900 tasks/hr
System saturates at order rate ~1000/hr
```

---

## Step 3: Stress Testing

Push the system to its limits:

```bash
waremax stress-test scenario.yaml \
  --load-multiplier 1.5 \
  --duration 7200
```

**Output:**

```
=== Stress Test Results ===

Load: 150% of normal (1500 orders/hr)
Duration: 2 hours

Performance Under Stress:
  Throughput: 1,020/hr (vs 1,050 normal)
  Task time: 58.3s (vs 42.1s normal)
  Wait time: 22.1s (vs 8.5s normal)

System Behavior:
  Queue buildup: Yes (max 25)
  Deadlocks: 2
  Recovery time: 180s after load reduction

Verdict: System degrades gracefully but does not fail
```

---

## Step 4: Identify Breaking Point

Find where system fails:

```bash
waremax stress-test scenario.yaml \
  --find-breaking-point \
  --metric queue_length \
  --threshold 50
```

**Output:**

```
Finding Breaking Point...

Load    Queue Length  Status
100%    3.2           OK
125%    8.5           OK
150%    18.2          Warning
175%    42.1          Warning
200%    85.3          EXCEEDED

Breaking point: ~175% load
At this point, queue length exceeds acceptable limits.
```

---

## Step 5: Comparative Benchmark

Benchmark multiple configurations:

```bash
waremax benchmark \
  --config config_a.yaml \
  --config config_b.yaml \
  --config config_c.yaml
```

**Output:**

```
=== Comparative Benchmark ===

Configuration    Throughput  P95 Time  Utilization  Score
config_a.yaml    1,050/hr    72s       78%          85
config_b.yaml    1,180/hr    65s       82%          92
config_c.yaml    980/hr      58s       71%          78

Best overall: config_b.yaml
Best latency: config_c.yaml
Best throughput: config_b.yaml
```

---

## Step 6: Long-Running Benchmark

Test for stability over extended periods:

```bash
waremax benchmark scenario.yaml \
  --duration 86400 \  # 24 hours
  --sample-interval 3600
```

**Output:**

```
=== Long-Running Benchmark (24 hours) ===

Hour  Throughput  Queue Avg  Utilization  Notes
1     1,045       3.2        78%          Warmup
2     1,052       3.5        79%
3     1,048       3.3        78%
4     1,051       3.4        78%
...
12    1,049       3.4        78%
...
24    1,047       3.5        78%          Stable

Summary:
  Mean throughput: 1,048 ± 8
  No degradation detected
  Memory stable
  No accumulating queues
```

---

## Step 7: Benchmark Report

Generate a detailed report:

```bash
waremax benchmark scenario.yaml \
  --report benchmark_report.md
```

**Generated report:**

```markdown
# Benchmark Report

## System Configuration
- Robots: 15
- Stations: 2 (concurrency 2 each)
- Map: 5x5 grid

## Performance Summary

| Metric | Value | Rating |
|--------|-------|--------|
| Max throughput | 1,050/hr | Good |
| P95 latency | 72s | Acceptable |
| Utilization | 78% | Optimal |
| Stability | 97% | Excellent |

## Capacity Analysis

| Load Level | Throughput | Status |
|------------|------------|--------|
| 50% | 500/hr | Under-utilized |
| 75% | 750/hr | Good |
| 100% | 1,000/hr | Optimal |
| 125% | 1,100/hr | Near capacity |
| 150% | 1,020/hr | Saturated |

## Recommendations
1. Current configuration handles 1,000 orders/hr
2. Add capacity before exceeding 1,100 orders/hr
3. Monitor queue lengths as early warning
```

---

## Step 8: Benchmark with Presets

Use built-in stress presets:

```bash
# High load test
waremax run --preset high_load -o high_load_results/

# Peak hours simulation
waremax run --preset peak_hours -o peak_results/

# Stress test
waremax run --preset stress_test -o stress_results/
```

Compare results:

```bash
waremax analyze high_load_results/ peak_results/ stress_results/
```

---

## Creating Benchmarks

### Define Success Criteria

```yaml
# benchmark_criteria.yaml
criteria:
  throughput:
    min: 1000
    target: 1200

  p95_task_time:
    max: 90

  utilization:
    min: 70
    max: 85

  queue_length:
    max: 10
```

Run against criteria:

```bash
waremax benchmark scenario.yaml \
  --criteria benchmark_criteria.yaml
```

**Output:**

```
Benchmark vs Criteria:

Metric          Value   Criteria      Status
Throughput      1,050   ≥1000         ✓ Pass
P95 task time   72s     ≤90s          ✓ Pass
Utilization     78%     70-85%        ✓ Pass
Queue length    8       ≤10           ✓ Pass

Overall: PASS (4/4 criteria met)
```

---

## Best Practices

### Warmup Period

Exclude warmup from measurements:

```bash
waremax benchmark scenario.yaml \
  --warmup 300  # Exclude first 5 minutes
```

### Multiple Runs

Account for variance:

```bash
waremax benchmark scenario.yaml \
  --runs 10
```

### Consistent Environment

- Use same machine for comparisons
- Close other applications
- Use fixed random seeds for reproducibility

### Document Conditions

Record benchmark context:

```markdown
Benchmark run on: 2024-01-15
Machine: 8-core, 32GB RAM
Waremax version: 0.5.0
Scenario: production_config.yaml
```

---

## Troubleshooting

### Inconsistent Results

```
Causes:
- Not enough runs
- System not reaching steady state
- External interference

Solutions:
- Increase run count
- Extend warmup period
- Use isolated environment
```

### Lower Than Expected Throughput

```
Causes:
- Bottleneck not identified
- Configuration issues
- Simulation artifacts

Solutions:
- Run bottleneck analysis
- Check configuration
- Verify against known baseline
```

---

## Next Steps

- [Finding Bottlenecks](../analysis/finding-bottlenecks.md): Diagnose limits
- [Capacity Planning](../analysis/capacity-planning.md): Size systems
- [Benchmark Command Reference](../../cli/benchmark.md)
