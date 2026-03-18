# waremax benchmark

Run benchmark suite for performance testing.

---

## Synopsis

```bash
waremax benchmark [OPTIONS]
```

---

## Description

The `benchmark` command runs a suite of predefined scenarios to measure performance. It can track results over time to detect performance regressions.

---

## Options

| Option | Default | Description |
|--------|---------|-------------|
| `--suite` | None | Custom benchmark suite file (JSON) |
| `--replications` | 3 | Replications per benchmark |
| `--history` | None | History file for regression tracking |
| `--regression-threshold` | 5.0 | Regression threshold percentage |
| `--output` | None | Output file for results (JSON) |

---

## Examples

### Run default benchmarks

```bash
waremax benchmark
```

### More replications

```bash
waremax benchmark --replications 5
```

### Track history

```bash
waremax benchmark \
  --history ./benchmark_history.json \
  --regression-threshold 5.0
```

### Save results

```bash
waremax benchmark \
  --replications 5 \
  --output benchmark_results.json
```

---

## Default Suite

Without `--suite`, benchmarks run using default presets:

- `minimal` - Quick validation
- `quick` - Fast test
- `standard` - Standard workload

---

## Output

### Console Output

```
Running benchmark suite...
  Replications: 3
  Regression threshold: 5.0%

Running benchmarks...

Benchmark Results
=================

Benchmark       Throughput      P95 Cycle       Robot Util      Run Time
                (orders/hr)     Time (s)        (%)             (ms)
--------------------------------------------------------------------------------
minimal         10.2 ± 0.5      25.3 ± 2.1      15.2% ± 1.8%    45
quick           32.5 ± 2.1      42.8 ± 3.5      35.4% ± 2.5%    125
standard        58.7 ± 4.2      68.5 ± 5.2      52.3% ± 3.1%    892

No regressions detected vs. history
History updated: 15 entries
```

### With Regressions

```
2 REGRESSION(S) DETECTED vs. history:
  quick - throughput: 35.2 → 32.5 (-7.7%)
  standard - p95_cycle_time: 62.3 → 68.5 (+9.9%)
```

---

## History Tracking

### Enabling History

```bash
waremax benchmark --history ./benchmark_history.json
```

### How It Works

1. After each run, results are added to history
2. New results are compared against historical average
3. Deviations beyond threshold are flagged as regressions

### History File Format

```json
{
  "entries": [
    {
      "timestamp": "2026-01-15T10:30:00Z",
      "benchmarks": {
        "minimal": {
          "throughput": 10.5,
          "p95_cycle_time": 24.8,
          "robot_utilization": 0.148
        },
        "quick": {
          "throughput": 35.2,
          "p95_cycle_time": 41.5,
          "robot_utilization": 0.362
        }
      }
    }
  ],
  "max_entries": 100
}
```

---

## Custom Suite

Create a custom benchmark suite:

```json
{
  "name": "production_benchmarks",
  "benchmarks": [
    {
      "name": "low_load",
      "scenario_path": "scenarios/low_load.yaml"
    },
    {
      "name": "medium_load",
      "scenario_path": "scenarios/medium_load.yaml"
    },
    {
      "name": "high_load",
      "scenario_path": "scenarios/high_load.yaml"
    }
  ]
}
```

Run with custom suite:

```bash
waremax benchmark --suite custom_suite.json
```

---

## Use Cases

### CI/CD Integration

```bash
#!/bin/bash
# Run benchmarks and fail on regression
waremax benchmark \
  --history ./ci/benchmark_history.json \
  --regression-threshold 5.0 \
  --output ./ci/latest_results.json

if [ $? -ne 0 ]; then
  echo "Performance regression detected!"
  exit 1
fi
```

### Regular Performance Tracking

```bash
# Weekly benchmark run
waremax benchmark \
  --replications 10 \
  --history ./perf/history.json \
  --output "./perf/results_$(date +%Y%m%d).json"
```

### Before Release

```bash
# Thorough benchmark before release
waremax benchmark \
  --replications 20 \
  --history ./release/benchmark_history.json \
  --regression-threshold 2.0
```

---

## Regression Detection

### Threshold

The `--regression-threshold` specifies the percentage deviation that triggers a regression alert:

- 5.0% = 5% worse than historical average
- Lower values are more sensitive
- Higher values reduce false positives

### Metrics Tracked

| Metric | Regression if... |
|--------|------------------|
| Throughput | Decreases by threshold |
| P95 Cycle Time | Increases by threshold |
| Robot Utilization | Significantly changes |

---

## Best Practices

### Consistent Environment

- Run benchmarks on consistent hardware
- Minimize background processes
- Use release builds

### Appropriate Replications

- CI/CD: 3-5 replications (fast)
- Regular tracking: 5-10 replications
- Release verification: 10-20 replications

### History Management

- Keep history files in version control
- Periodically clean old entries
- Reset after significant code changes

---

## See Also

- [stress-test](stress-test.md) - High-load testing
- [sweep](sweep.md) - Parameter exploration
- [ab-test](ab-test.md) - Statistical comparison
