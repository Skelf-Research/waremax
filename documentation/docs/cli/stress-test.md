# waremax stress-test

Run stress test with high load parameters.

---

## Synopsis

```bash
waremax stress-test [OPTIONS]
```

---

## Description

The `stress-test` command runs a high-load simulation to test system capacity and identify breaking points. It automatically configures aggressive parameters and runs multiple replications.

---

## Options

| Option | Default | Description |
|--------|---------|-------------|
| `--robots` | 50 | Number of robots |
| `--order-rate` | 300.0 | Order rate per hour |
| `--duration` | 60.0 | Duration in minutes |
| `--grid` | 20x20 | Grid size (e.g., "20x20") |
| `--output` | None | Output file for results (JSON) |

---

## Examples

### Run with defaults

```bash
waremax stress-test
```

### Extreme load

```bash
waremax stress-test \
  --robots 100 \
  --order-rate 500 \
  --duration 120 \
  --grid 30x30
```

### Save results

```bash
waremax stress-test \
  --robots 75 \
  --order-rate 400 \
  --output stress_results.json
```

---

## Configuration

The stress test automatically configures:

- **Traffic policy**: `reroute_on_wait`
- **Routing**: A* with congestion awareness
- **Stations**: Scaled based on robot count (1 per ~5 robots)
- **Station concurrency**: 2
- **Replications**: 3 (with different seeds)

---

## Output

### Console Output

```
Running stress test...
  Robots: 50
  Order rate: 300.0 orders/hr
  Duration: 60.0 min
  Grid: 20x20

Running 3 replications...

Stress Test Results:
==================================================
  Average Throughput: 285.4 orders/hr
  Average P95 Cycle Time: 92.3 s
  Average Robot Utilization: 78.5%
  Avg Run Time: 2.3s

Warning: High robot utilization may indicate capacity constraints
```

### Warnings

| Warning | Meaning |
|---------|---------|
| High robot utilization (>95%) | Robots at capacity limit |
| High P95 cycle time (>300s) | Significant bottlenecks |
| Low throughput vs order rate | System cannot keep up |

---

## Interpreting Results

### Healthy System

- Throughput ≈ order rate
- Robot utilization 60-80%
- P95 cycle time reasonable (< 120s)

### Capacity Limited

- Throughput < order rate
- Robot utilization > 90%
- Growing queues

### Bottlenecked

- High P95 cycle time
- Uneven station utilization
- High congestion scores

---

## Use Cases

### Find Capacity Limits

```bash
# Start with moderate load
waremax stress-test --robots 30 --order-rate 200

# Increase gradually
waremax stress-test --robots 40 --order-rate 250
waremax stress-test --robots 50 --order-rate 300
waremax stress-test --robots 60 --order-rate 350

# Find where system breaks down
```

### Test Specific Configuration

```bash
# Test production-like scenario
waremax stress-test \
  --robots 45 \
  --order-rate 280 \
  --duration 480 \
  --grid 25x25 \
  --output production_stress.json
```

### Compare Under Load

```bash
# Test current config
waremax stress-test --output current_stress.json

# Modify policy/routing
# Test new config
waremax stress-test --output new_stress.json

# Compare results
```

---

## Performance Metrics

| Metric | Description |
|--------|-------------|
| Average Throughput | Orders/hour across replications |
| Average P95 Cycle Time | 95th percentile completion time |
| Average Robot Utilization | Robot busy percentage |
| Run Time | Simulation wall-clock time |

---

## Best Practices

### Incremental Testing

1. Start with moderate parameters
2. Increase load gradually
3. Identify the breaking point
4. Document capacity limits

### Realistic Scenarios

- Match production grid size
- Use expected robot counts
- Test at peak order rates
- Include warmup period

### Analysis

- Compare results across runs
- Look for patterns in failures
- Test mitigation strategies

---

## JSON Output

```json
{
  "config": {
    "robots": 50,
    "order_rate": 300.0,
    "duration_min": 60.0,
    "grid": "20x20"
  },
  "results": {
    "avg_throughput": 285.4,
    "avg_p95_cycle_time": 92.3,
    "avg_robot_utilization": 0.785
  },
  "runs": [
    {
      "label": "stress_seed1",
      "throughput": 282.1,
      "p95_cycle_time": 95.2,
      "robot_utilization": 0.792,
      "duration_ms": 2345
    },
    {
      "label": "stress_seed2",
      "throughput": 288.5,
      "p95_cycle_time": 89.8,
      "robot_utilization": 0.778,
      "duration_ms": 2256
    },
    {
      "label": "stress_seed3",
      "throughput": 285.6,
      "p95_cycle_time": 91.9,
      "robot_utilization": 0.785,
      "duration_ms": 2301
    }
  ]
}
```

---

## See Also

- [benchmark](benchmark.md) - Standard benchmarking
- [sweep](sweep.md) - Systematic parameter exploration
- [analyze](analyze.md) - Root cause analysis
