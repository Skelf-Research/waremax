# waremax sweep

Run a parameter sweep across multiple configurations.

---

## Synopsis

```bash
waremax sweep --base <PATH> --sweep <SPEC> --output-dir <PATH> [OPTIONS]
```

---

## Description

The `sweep` command systematically varies a parameter across specified values, running multiple simulations to understand how the parameter affects performance. Results are aggregated and ranked.

---

## Options

### Required

| Option | Description |
|--------|-------------|
| `--base` | Base scenario file |
| `--sweep` | Sweep specification (see format below) |
| `--output-dir` | Directory for results |

### Optional

| Option | Default | Description |
|--------|---------|-------------|
| `--replications` | 3 | Replications per configuration |

---

## Sweep Specification

Format: `parameter:value1,value2,value3,...`

### Supported Parameters

| Parameter | Description |
|-----------|-------------|
| `robots` | Number of robots |
| `order_rate` | Order arrival rate (orders/hour) |
| `stations` | Number of stations |

### Examples

```bash
# Sweep robot count
--sweep "robots:5,10,15,20"

# Sweep order rate
--sweep "order_rate:30,60,90,120"

# Sweep station count
--sweep "stations:2,4,6,8"
```

---

## Examples

### Robot count sweep

```bash
waremax sweep \
  --base baseline.yaml \
  --sweep "robots:5,10,15,20,25" \
  --replications 5 \
  --output-dir ./robot_sweep
```

### Order rate sweep

```bash
waremax sweep \
  --base baseline.yaml \
  --sweep "order_rate:60,90,120,150" \
  --replications 3 \
  --output-dir ./rate_sweep
```

### Station count sweep

```bash
waremax sweep \
  --base baseline.yaml \
  --sweep "stations:2,4,6" \
  --replications 3 \
  --output-dir ./station_sweep
```

---

## Output

### Console Output

```
Running parameter sweep...
  Base scenario: baseline.yaml
  Sweep: robots:5,10,15,20
  Replications: 3
Generated 12 scenarios

Results by Throughput:
------------------------------------------------------------
 1. robots=20                      285.4 ± 12.3 orders/hr
 2. robots=15                      267.8 ± 8.7 orders/hr
 3. robots=10                      198.5 ± 15.2 orders/hr
 4. robots=5                       112.3 ± 6.1 orders/hr

Results saved to: ./robot_sweep/sweep_results.json
```

### Results File

`sweep_results.json`:

```json
[
  {
    "label": "robots=5_seed=1000",
    "seed": 1000,
    "throughput": 108.5,
    "p95_cycle_time": 72.3,
    "robot_utilization": 0.85,
    "station_utilization": 0.45,
    "duration_ms": 1234
  },
  {
    "label": "robots=5_seed=1001",
    "seed": 1001,
    "throughput": 115.2,
    "p95_cycle_time": 68.9,
    "robot_utilization": 0.82,
    "station_utilization": 0.48,
    "duration_ms": 1198
  }
]
```

---

## Understanding Results

### Ranking

Results are ranked by throughput, showing:

- Mean throughput across replications
- Standard deviation

### Statistical Significance

- Higher replications provide more reliable means
- Large standard deviation indicates high variability
- Consider using [ab-test](ab-test.md) for statistical comparison

---

## Use Cases

### Capacity Planning

```bash
# Find optimal robot count for target throughput
waremax sweep \
  --base baseline.yaml \
  --sweep "robots:5,10,15,20,25,30" \
  --replications 5 \
  --output-dir ./capacity_study
```

### Sensitivity Analysis

```bash
# Understand impact of order rate changes
waremax sweep \
  --base baseline.yaml \
  --sweep "order_rate:50,75,100,125,150,175,200" \
  --replications 3 \
  --output-dir ./sensitivity
```

### Scaling Study

```bash
# Test different warehouse sizes
for size in 5 10 15 20; do
  waremax generate --preset standard --output "size_${size}.yaml" --grid "${size}x${size}"
done

# Sweep robots for each size
for size in 5 10 15 20; do
  waremax sweep \
    --base "size_${size}.yaml" \
    --sweep "robots:5,10,15,20" \
    --output-dir "./scaling_${size}"
done
```

---

## Best Practices

### Replications

- Use at least 3 replications for general testing
- Use 5+ replications for important decisions
- Use 10+ replications for publication-quality results

### Parameter Ranges

- Start with wide ranges
- Narrow down to interesting regions
- Include current/baseline values

### Analysis

- Look for diminishing returns (e.g., more robots not helping)
- Identify knee points where behavior changes
- Consider interactions between parameters

---

## See Also

- [compare](compare.md) - Compare two specific configurations
- [ab-test](ab-test.md) - Statistical A/B testing
- [benchmark](benchmark.md) - Benchmark suites
