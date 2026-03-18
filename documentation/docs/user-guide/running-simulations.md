# Running Simulations

This guide covers all options for running simulations with Waremax.

---

## Basic Usage

Run a simulation from a scenario file:

```bash
waremax run --scenario my_scenario.yaml
```

---

## Command Options

### Required Options

| Option | Description |
|--------|-------------|
| `--scenario`, `-s` | Path to scenario YAML file |

### Output Options

| Option | Description | Default |
|--------|-------------|---------|
| `--output`, `-o` | Output format: `text` or `json` | `text` |
| `--output-dir` | Directory for export files | None |

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
| `--seed` | Override scenario seed |

---

## Output Formats

### Text Output (Default)

Human-readable summary:

```bash
waremax run --scenario my_scenario.yaml --output text
```

Output:

```
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

Machine-readable format:

```bash
waremax run --scenario my_scenario.yaml --output json
```

Output:

```json
{
  "duration_s": 3600.0,
  "warmup_s": 300.0,
  "orders_completed": 245,
  "throughput_per_hour": 267.3,
  "cycle_times": {
    "mean_s": 42.3,
    "p95_s": 78.5,
    "p99_s": 95.2
  },
  "robot_utilization": 0.672,
  "station_utilization": 0.725
}
```

Redirect to file:

```bash
waremax run --scenario my_scenario.yaml --output json > results.json
```

---

## Exporting Data

### Export to Directory

Generate all export files:

```bash
waremax run --scenario my_scenario.yaml \
  --output-dir ./results \
  --per-robot \
  --per-station \
  --heatmap \
  --timeseries \
  --trace
```

Creates:

```
results/
├── report.json         # Full simulation report
├── robots.csv          # Per-robot metrics
├── stations.csv        # Per-station metrics
├── node_congestion.csv # Node congestion scores
├── edge_congestion.csv # Edge congestion scores
├── timeseries.csv      # Time series data
└── trace.csv           # Event trace
```

### Selective Exports

Export only specific data:

```bash
# Only robot and station breakdowns
waremax run --scenario my_scenario.yaml \
  --output-dir ./results \
  --per-robot \
  --per-station

# Only congestion data
waremax run --scenario my_scenario.yaml \
  --output-dir ./results \
  --heatmap
```

---

## Seed Control

### Using Scenario Seed

By default, Waremax uses the seed from the scenario file:

```yaml
seed: 42
```

### Overriding Seed

Override at runtime:

```bash
waremax run --scenario my_scenario.yaml --seed 12345
```

### Running Multiple Seeds

Run same configuration with different seeds:

```bash
for seed in 1 2 3 4 5; do
  waremax run --scenario my_scenario.yaml \
    --seed $seed \
    --output json > results_seed_$seed.json
done
```

---

## Performance Considerations

### Simulation Duration

Longer simulations provide more stable statistics but take more time:

```yaml
simulation:
  duration_minutes: 60   # Production run
  warmup_minutes: 10     # Allow system to reach steady state
```

For quick iteration:

```yaml
simulation:
  duration_minutes: 10   # Quick test
  warmup_minutes: 2
```

### Warmup Period

The warmup period allows the system to reach steady state before collecting metrics:

- **Too short**: Metrics influenced by startup transients
- **Too long**: Wastes simulation time
- **Recommended**: 10-20% of total duration

### Build Mode

Always use release builds for actual simulations:

```bash
# Build release binary
cargo build --release

# Use release binary
./target/release/waremax run --scenario my_scenario.yaml
```

---

## Batch Running

### Using Compare Command

Compare two configurations:

```bash
waremax compare \
  --baseline baseline.yaml \
  --variant variant.yaml \
  --replications 5 \
  --output comparison.json
```

### Using Sweep Command

Run parameter sweep:

```bash
waremax sweep \
  --base baseline.yaml \
  --sweep "robots:5,10,15,20" \
  --replications 3 \
  --output-dir ./sweep_results
```

### Shell Scripts

Run multiple scenarios:

```bash
#!/bin/bash
for scenario in scenarios/*.yaml; do
  name=$(basename "$scenario" .yaml)
  waremax run --scenario "$scenario" \
    --output json > "results/${name}.json"
done
```

---

## Event Tracing

Enable detailed event logging:

```bash
waremax run --scenario my_scenario.yaml --trace --output-dir ./results
```

Trace output includes:

| Event Type | Description |
|------------|-------------|
| `OrderArrival` | New order enters system |
| `TaskAssignment` | Task assigned to robot |
| `RobotDepart` | Robot starts moving |
| `RobotArrive` | Robot arrives at destination |
| `StationServiceStart` | Robot begins service |
| `StationServiceEnd` | Robot completes service |
| `ChargingStart` | Robot begins charging |
| `ChargingEnd` | Robot finishes charging |

---

## Attribution Tracking

Enable delay attribution for root cause analysis:

```bash
waremax run --scenario my_scenario.yaml --attribution --output-dir ./results
```

Attribution data shows:

- Time spent waiting in queues
- Time spent waiting for traffic
- Travel time breakdown
- Service time breakdown

Use with the analyze command:

```bash
waremax analyze --scenario my_scenario.yaml
```

---

## Error Handling

### Validation Errors

If the scenario has errors, Waremax reports them:

```
Error loading scenario: YAML parse error: ...
```

Fix the scenario file and retry.

### Runtime Errors

If simulation fails during execution:

```
Error: Simulation failed - no valid routes found
```

Check:

- Map connectivity
- Station node assignments
- Robot starting positions

---

## Examples

### Basic Run

```bash
waremax run --scenario my_scenario.yaml
```

### Full Export

```bash
waremax run --scenario my_scenario.yaml \
  --output-dir ./results \
  --per-robot \
  --per-station \
  --heatmap \
  --timeseries \
  --trace \
  --attribution
```

### Different Seed

```bash
waremax run --scenario my_scenario.yaml --seed 99999 --output json
```

### JSON to File

```bash
waremax run --scenario my_scenario.yaml --output json > output.json
```

---

## Next Steps

- **[Scenario Files](scenario-files.md)** - Understand scenario configuration
- **[Export Formats](export-formats.md)** - Details on output files
- **[CLI Reference](../cli/run.md)** - Complete command reference
