# waremax run

Run a simulation from a scenario file.

---

## Synopsis

```bash
waremax run --scenario <PATH> [OPTIONS]
```

---

## Description

The `run` command executes a discrete-event simulation using the configuration specified in a scenario file. It processes orders, simulates robot movement and task execution, and outputs performance metrics.

---

## Options

### Required

| Option | Description |
|--------|-------------|
| `--scenario`, `-s` | Path to the scenario YAML file |

### Output Control

| Option | Default | Description |
|--------|---------|-------------|
| `--output`, `-o` | `text` | Output format: `text` or `json` |
| `--output-dir` | None | Directory for export files |

### Export Flags

| Flag | Description |
|------|-------------|
| `--per-robot` | Generate per-robot breakdown CSV |
| `--per-station` | Generate per-station breakdown CSV |
| `--heatmap` | Generate congestion heatmap CSVs |
| `--timeseries` | Generate time-series CSV |
| `--trace` | Enable event tracing |
| `--attribution` | Enable delay attribution for RCA |

### Simulation Control

| Option | Description |
|--------|-------------|
| `--seed` | Override the scenario seed |

---

## Examples

### Basic run

```bash
waremax run --scenario my_scenario.yaml
```

### JSON output

```bash
waremax run --scenario my_scenario.yaml --output json
```

### Save to file

```bash
waremax run --scenario my_scenario.yaml --output json > results.json
```

### Export all data

```bash
waremax run --scenario my_scenario.yaml \
  --output-dir ./results \
  --per-robot \
  --per-station \
  --heatmap \
  --timeseries \
  --trace
```

### Override seed

```bash
waremax run --scenario my_scenario.yaml --seed 12345
```

### Enable attribution tracking

```bash
waremax run --scenario my_scenario.yaml --attribution
```

---

## Output

### Text Output

```
Loading scenario from: my_scenario.yaml
Running simulation with seed: 42
Duration: 60 minutes (warmup: 5 minutes)
Distributions:
  Arrivals: Poisson(rate=0.017/s)
  Lines/Order: NegBinomial(mean=3.0, dispersion=1.0)
  SKU Selection: Zipf(alpha=1.0)
Policies:
  Task Allocation: nearest_robot
  Station Assignment: least_queue
  Batching: none
  Priority: strict_priority

Simulation Complete
==================
Duration: 60.0 minutes (warmup: 5.0 minutes)

Orders:
  Completed: 245
  Throughput: 267.3 orders/hr

Cycle Times:
  Average: 42.3s
  P95: 78.5s
  P99: 95.2s

Utilization:
  Robot Fleet: 67.2%
  Stations: 72.5%
```

### JSON Output

```json
{
  "duration_s": 3600.0,
  "warmup_s": 300.0,
  "orders_completed": 245,
  "throughput_per_hour": 267.3,
  "cycle_times": {
    "mean_s": 42.3,
    "median_s": 38.5,
    "p95_s": 78.5,
    "p99_s": 95.2
  },
  "robot_utilization": 0.672,
  "station_utilization": 0.725
}
```

---

## Export Files

When using `--output-dir`:

| File | Contents | Flag Required |
|------|----------|---------------|
| `report.json` | Full simulation report | Always |
| `robots.csv` | Per-robot metrics | `--per-robot` |
| `stations.csv` | Per-station metrics | `--per-station` |
| `node_congestion.csv` | Node congestion | `--heatmap` |
| `edge_congestion.csv` | Edge congestion | `--heatmap` |
| `timeseries.csv` | Time series data | `--timeseries` |
| `trace.csv` | Event trace | `--trace` |

---

## See Also

- [validate](validate.md) - Validate scenarios without running
- [demo](demo.md) - Quick demo simulations
- [Scenario Files](../user-guide/scenario-files.md) - Configuration reference
