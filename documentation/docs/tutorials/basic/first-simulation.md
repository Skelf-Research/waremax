# Your First Simulation

Run a complete simulation and understand the output.

---

## Goal

By the end of this tutorial, you will:

- Run a simulation using a built-in preset
- Understand the output format
- Know where results are stored

**Time**: 15-20 minutes

---

## Prerequisites

- Waremax installed
- Terminal access

Verify installation:

```bash
waremax --version
```

---

## Step 1: Run a Demo Simulation

The easiest way to start is with the `demo` command:

```bash
waremax demo
```

This runs a pre-configured simulation and displays results.

**Expected output:**

```
🚀 Running demo simulation...

Warehouse: 5x5 grid, 25 nodes
Robots: 5
Stations: 2 pick stations
Duration: 300s (5 minutes)

⏳ Simulating... ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100%

=== Simulation Results ===
Duration: 300.0s
Tasks completed: 127
Throughput: 1,524 tasks/hour
Average task time: 23.4s
Robot utilization: 72.3%
```

---

## Step 2: Run with a Preset

Presets are built-in configurations for common scenarios:

```bash
waremax run --preset standard
```

View available presets:

```bash
waremax list-presets
```

**Common presets:**

| Preset | Description |
|--------|-------------|
| `minimal` | Smallest possible simulation |
| `quick` | Fast 60-second simulation |
| `standard` | Balanced 1-hour simulation |
| `high_load` | Stress testing with many robots |

---

## Step 3: Save Results

Save simulation output to a directory:

```bash
waremax run --preset standard -o my_results/
```

This creates:

```
my_results/
├── summary.json      # Key metrics
├── events.jsonl      # All simulation events
├── timeseries.csv    # Metrics over time
└── config.yaml       # Configuration used
```

---

## Step 4: View Results

### Summary

View the summary:

```bash
cat my_results/summary.json
```

```json
{
  "simulation": {
    "duration_s": 3600,
    "seed": 12345
  },
  "tasks": {
    "completed": 1250,
    "throughput_per_hour": 1250,
    "avg_time_s": 42.3
  },
  "robots": {
    "count": 10,
    "utilization": 0.78
  }
}
```

### Time Series

View metrics over time:

```bash
head my_results/timeseries.csv
```

```csv
timestamp,throughput,utilization,queue_length
0,0,0.00,0
60,82,0.65,2
120,95,0.78,3
180,91,0.80,3
```

---

## Step 5: Analyze Results

Use the analyze command for detailed analysis:

```bash
waremax analyze my_results/
```

**Output:**

```
=== Analysis Report ===

Performance Summary:
  Throughput: 1,250 tasks/hour
  Avg task time: 42.3s
  Robot utilization: 78%

Bottleneck Analysis:
  Primary constraint: Station S1 (95% utilization)
  Secondary: Edge E15 (82% occupancy)

Recommendations:
  - Consider adding capacity to Station S1
  - Review traffic flow near E15
```

---

## Understanding the Output

### Key Metrics

| Metric | Meaning | Good Value |
|--------|---------|------------|
| Throughput | Tasks completed per hour | Higher is better |
| Avg task time | Average task duration | Lower is better |
| Utilization | Robot busy percentage | 70-85% |

### What to Look For

**Good simulation:**
```
✓ Throughput meets target
✓ Utilization 70-85%
✓ No deadlocks
✓ Low wait times
```

**Signs of problems:**
```
✗ Low throughput + high utilization = congestion
✗ Low throughput + low utilization = not enough tasks
✗ High wait times = bottlenecks
✗ Deadlocks > 0 = traffic issues
```

---

## Next Steps

Now that you can run simulations:

1. **Create custom scenarios**: [Creating Scenarios](creating-scenarios.md)
2. **Analyze in depth**: [Analyzing Results](analyzing-results.md)
3. **Build custom maps**: [Custom Maps](../config/custom-maps.md)

---

## Troubleshooting

### "Command not found"

Waremax not in PATH. Re-run installation or add to PATH.

### Simulation runs slowly

Try a shorter duration:

```bash
waremax run --preset quick  # 60-second simulation
```

### Results directory exists

Use a different name or add `--force`:

```bash
waremax run --preset standard -o my_results/ --force
```
