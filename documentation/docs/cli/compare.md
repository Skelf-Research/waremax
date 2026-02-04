# waremax compare

Compare two scenario configurations.

---

## Synopsis

```bash
waremax compare --baseline <PATH> --variant <PATH> [OPTIONS]
```

---

## Description

The `compare` command runs both a baseline and variant configuration multiple times, then compares their performance metrics. It provides a quick way to understand the difference between two configurations.

---

## Options

### Required

| Option | Description |
|--------|-------------|
| `--baseline` | Path to baseline scenario file |
| `--variant` | Path to variant scenario file |

### Optional

| Option | Default | Description |
|--------|---------|-------------|
| `--replications` | 5 | Number of replications per configuration |
| `--output` | None | Output file for results (JSON) |

---

## Examples

### Basic comparison

```bash
waremax compare \
  --baseline baseline.yaml \
  --variant variant.yaml
```

### More replications

```bash
waremax compare \
  --baseline baseline.yaml \
  --variant variant.yaml \
  --replications 10
```

### Save results

```bash
waremax compare \
  --baseline baseline.yaml \
  --variant variant.yaml \
  --replications 5 \
  --output comparison.json
```

---

## Output

### Console Output

```
Comparing configurations...
  Baseline: baseline.yaml
  Variant: variant.yaml
  Replications: 5

Running 5 baseline simulations...
Running 5 variant simulations...

Comparison Results
==================

                    Baseline        Variant         Diff
Throughput (ord/hr) 198.5 ± 12.3    267.8 ± 8.7     +34.9%
P95 Cycle Time (s)  78.5 ± 5.2      65.3 ± 4.1      -16.8%
Robot Utilization   67.2% ± 3.1%    58.4% ± 2.8%    -13.1%
Station Utilization 72.5% ± 4.5%    68.2% ± 3.2%    -5.9%
```

### JSON Output

```json
{
  "baseline": {
    "name": "baseline",
    "replications": 5,
    "throughput": {
      "mean": 198.5,
      "std_dev": 12.3,
      "min": 182.1,
      "max": 215.4
    },
    "p95_cycle_time": {
      "mean": 78.5,
      "std_dev": 5.2
    },
    "robot_utilization": {
      "mean": 0.672,
      "std_dev": 0.031
    },
    "station_utilization": {
      "mean": 0.725,
      "std_dev": 0.045
    }
  },
  "variant": {
    "name": "variant",
    "replications": 5,
    "throughput": {
      "mean": 267.8,
      "std_dev": 8.7,
      "min": 255.2,
      "max": 278.9
    }
  },
  "comparison": {
    "throughput_diff_pct": 34.9,
    "p95_cycle_time_diff_pct": -16.8,
    "robot_utilization_diff_pct": -13.1,
    "station_utilization_diff_pct": -5.9
  }
}
```

---

## Metrics Compared

| Metric | Description |
|--------|-------------|
| Throughput | Orders completed per hour |
| P95 Cycle Time | 95th percentile order completion time |
| Robot Utilization | Average robot busy percentage |
| Station Utilization | Average station capacity usage |

---

## Interpreting Results

### Throughput

- Positive diff: Variant completes more orders
- Negative diff: Variant completes fewer orders

### Cycle Time

- Negative diff: Variant is faster (better)
- Positive diff: Variant is slower (worse)

### Utilization

- Lower utilization with same/higher throughput = more efficient
- Higher utilization may indicate approaching capacity limits

---

## Use Cases

### Policy Comparison

```bash
# Compare task allocation policies
# baseline.yaml: nearest_robot
# variant.yaml: workload_balanced

waremax compare \
  --baseline policies/nearest.yaml \
  --variant policies/workload.yaml
```

### Capacity Analysis

```bash
# Compare with more robots
# baseline.yaml: 10 robots
# variant.yaml: 15 robots

waremax compare \
  --baseline capacity/10_robots.yaml \
  --variant capacity/15_robots.yaml
```

### Before/After

```bash
# Compare before and after a change
waremax compare \
  --baseline scenarios/before_change.yaml \
  --variant scenarios/after_change.yaml \
  --replications 10 \
  --output before_after_comparison.json
```

---

## Limitations

- Does not perform statistical significance testing
- For formal A/B testing, use [ab-test](ab-test.md)
- For multiple configurations, use [sweep](sweep.md)

---

## See Also

- [ab-test](ab-test.md) - Statistical A/B testing
- [sweep](sweep.md) - Parameter sweeps
- [benchmark](benchmark.md) - Benchmark suites
