# Analyzing Results

Interpret simulation output and extract insights.

---

## Goal

By the end of this tutorial, you will:

- Read and interpret simulation output files
- Use the analyze command effectively
- Identify performance issues from metrics
- Create actionable recommendations

**Time**: 30 minutes

---

## Prerequisites

- Completed [Your First Simulation](first-simulation.md)
- Results from a simulation run

---

## Step 1: Generate Results

Run a simulation with output:

```bash
waremax run --preset standard -o results/
```

---

## Step 2: Explore Output Files

List the results directory:

```bash
ls results/
```

```
config.yaml       # Configuration used
events.jsonl      # All simulation events
summary.json      # Key metrics
timeseries.csv    # Metrics over time
```

---

## Step 3: Read the Summary

View key metrics:

```bash
cat results/summary.json | jq .
```

```json
{
  "simulation": {
    "duration_s": 3600,
    "seed": 12345,
    "completed": true
  },
  "tasks": {
    "created": 1280,
    "completed": 1250,
    "throughput_per_hour": 1250,
    "avg_time_s": 42.3,
    "median_time_s": 38.0,
    "p95_time_s": 72.5
  },
  "robots": {
    "count": 10,
    "utilization": 0.78,
    "avg_travel_time_s": 15.2,
    "avg_wait_time_s": 8.1
  },
  "stations": {
    "avg_utilization": 0.75,
    "avg_queue_length": 2.3
  }
}
```

### Key Questions to Answer

| Metric | Question |
|--------|----------|
| `throughput_per_hour` | Meeting target? |
| `avg_time_s` vs `p95_time_s` | Consistent performance? |
| `utilization` | Robots busy enough? |
| `avg_wait_time_s` | Congestion issues? |

---

## Step 4: Analyze Time Series

Look at how metrics change over time:

```bash
head -20 results/timeseries.csv
```

```csv
timestamp,throughput,robot_utilization,avg_queue_length,wait_time
0,0,0.00,0.0,0.0
60,82,0.65,1.2,2.1
120,95,0.78,2.5,4.3
180,91,0.80,2.8,5.1
240,88,0.82,3.2,6.2
```

### What to Look For

**Warmup period:**
```
t=0-180s: Metrics ramping up
t=180s+: Steady state
```

**Patterns:**
```
Consistent: Values stable after warmup ✓
Trending: Values increasing/decreasing (investigate)
Oscillating: Periodic behavior (check order patterns)
```

---

## Step 5: Use the Analyze Command

Get automated analysis:

```bash
waremax analyze results/
```

**Output:**

```
=== Simulation Analysis ===

📊 Performance Summary
  Duration: 3600s (1 hour)
  Tasks completed: 1,250
  Throughput: 1,250 tasks/hour

📈 Task Metrics
  Average time: 42.3s
  Median time: 38.0s
  P95 time: 72.5s
  Standard deviation: 18.2s

🤖 Robot Metrics
  Fleet size: 10
  Utilization: 78%
  Travel time: 36% of task time
  Wait time: 19% of task time

🏭 Station Metrics
  Station S1: 85% utilization, avg queue 3.2
  Station S2: 65% utilization, avg queue 1.4

⚠️ Potential Issues
  - Station S1 utilization is high (>80%)
  - P95 time is 1.7× the average (some tasks delayed)

💡 Recommendations
  - Consider adding capacity to Station S1
  - Investigate causes of task time variance
```

---

## Step 6: Dive into Specifics

### Task Time Breakdown

```bash
waremax analyze results/ --breakdown task-time
```

```
Task Time Breakdown:
  Queue wait:     5.2s (12%)
  Travel time:   18.3s (43%)
  Traffic wait:   6.7s (16%)
  Station queue:  7.5s (18%)
  Service time:   4.6s (11%)
  ─────────────────────────
  Total:         42.3s (100%)

Largest contributor: Travel time (43%)
```

### Station Analysis

```bash
waremax analyze results/ --focus stations
```

```
Station Analysis:

Station S1:
  Utilization: 85%
  Throughput: 720 tasks/hour
  Avg queue: 3.2 robots
  Max queue: 8 robots
  Avg service: 5.0s

Station S2:
  Utilization: 65%
  Throughput: 530 tasks/hour
  Avg queue: 1.4 robots
  Max queue: 4 robots
  Avg service: 5.2s

Imbalance detected: S1 has 2.3× the queue of S2
```

### Robot Analysis

```bash
waremax analyze results/ --focus robots
```

```
Robot Analysis:

Fleet Utilization Distribution:
  R1:  82% ████████░░
  R2:  79% ████████░░
  R3:  78% ████████░░
  R4:  77% ████████░░
  R5:  76% ████████░░
  R6:  75% ████████░░
  R7:  75% ████████░░
  R8:  74% ███████░░░
  R9:  73% ███████░░░
  R10: 71% ███████░░░

Utilization is fairly balanced (71-82%)
```

---

## Step 7: Compare Results

Compare two configurations:

```bash
# Run baseline
waremax run scenario.yaml -o baseline/

# Run with more robots
waremax run scenario.yaml --param robots.count=15 -o more_robots/

# Compare
waremax analyze baseline/ more_robots/
```

```
=== Comparison: baseline vs more_robots ===

                    baseline    more_robots    change
─────────────────────────────────────────────────────
Throughput/hr       1,250       1,380         +10.4%
Avg task time       42.3s       38.1s         -9.9%
Robot utilization   78%         65%           -13%
Avg wait time       8.1s        5.2s          -35.8%

Summary: Adding robots improved throughput 10% with
         35% less wait time, but lower utilization.
```

---

## Step 8: Export for Further Analysis

### Export to CSV

```bash
waremax analyze results/ --export analysis.csv
```

### Export to JSON

```bash
waremax analyze results/ --format json > analysis.json
```

---

## Interpretation Guide

### Healthy Metrics

```
✓ Throughput meeting target
✓ Robot utilization 70-85%
✓ P95 time < 2× average
✓ Station utilization balanced
✓ Wait time < 15% of task time
```

### Warning Signs

```
⚠️ Station utilization >90% = bottleneck
⚠️ P95 time >2× average = inconsistent
⚠️ Wait time >25% = congestion
⚠️ Robot utilization >90% = overloaded
⚠️ Robot utilization <60% = underutilized
```

### Root Cause Hints

| Symptom | Likely Cause |
|---------|--------------|
| High wait, high util | Congestion |
| Low throughput, low util | Not enough demand |
| Low throughput, high util | Bottleneck |
| High variance | Unbalanced load |

---

## Next Steps

- [Finding Bottlenecks](../analysis/finding-bottlenecks.md): Deeper diagnosis
- [Root Cause Analysis](../analysis/root-cause-analysis.md): Systematic troubleshooting
- [Parameter Sweeps](../testing/parameter-sweeps.md): Systematic optimization
