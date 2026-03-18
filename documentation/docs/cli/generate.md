# waremax generate

Generate a scenario file from a preset.

---

## Synopsis

```bash
waremax generate --preset <NAME> --output <PATH> [OPTIONS]
```

---

## Description

The `generate` command creates a scenario YAML file based on a predefined preset. You can override specific parameters to customize the generated configuration.

---

## Options

### Required

| Option | Description |
|--------|-------------|
| `--preset` | Preset name (see [list-presets](list-presets.md)) |
| `--output`, `-o` | Output file path |

### Overrides

| Option | Description |
|--------|-------------|
| `--robots` | Override number of robots |
| `--order-rate` | Override order rate (orders/hour) |
| `--grid` | Override grid size (e.g., "10x10") |
| `--seed` | Override random seed |

---

## Available Presets

| Preset | Description |
|--------|-------------|
| `minimal` | Minimal config for unit tests |
| `quick` | Quick iteration testing |
| `standard` | Standard testing |
| `baseline` | Reproducible baseline |
| `high_load` | High load stress testing |
| `peak_hours` | Peak demand simulation |
| `stress_test` | Maximum stress testing |
| `battery_test` | Battery/charging focused |
| `maintenance_test` | Maintenance/reliability focused |

---

## Examples

### Generate from preset

```bash
waremax generate --preset standard --output my_scenario.yaml
```

### Override robot count

```bash
waremax generate --preset standard --output scenario.yaml --robots 20
```

### Override order rate

```bash
waremax generate --preset standard --output scenario.yaml --order-rate 120
```

### Override grid size

```bash
waremax generate --preset standard --output scenario.yaml --grid 15x15
```

### Override seed

```bash
waremax generate --preset baseline --output scenario.yaml --seed 99999
```

### Multiple overrides

```bash
waremax generate --preset standard --output scenario.yaml \
  --robots 15 \
  --order-rate 100 \
  --grid 12x12 \
  --seed 42
```

---

## Output

### Success

```
Generating scenario from preset: standard
Scenario generated: my_scenario.yaml
  Preset: standard - Standard testing (10x10 grid, 10 robots, 30 min)
  Robots: 10
  Stations: 4
  Order Rate: 60.0 orders/hr
  Duration: 30.0 min
```

### With Overrides

```
Generating scenario from preset: standard
Scenario generated: my_scenario.yaml
  Preset: standard - Standard testing (10x10 grid, 10 robots, 30 min)
  Robots: 20
  Stations: 4
  Order Rate: 120.0 orders/hr
  Duration: 30.0 min
```

### Unknown Preset

```
Unknown preset: invalid_preset
Available presets:
  minimal - Minimal config for unit tests (3x3 grid, 1 robot, 1 station)
  quick - Quick iteration testing (5x5 grid, 3 robots, 5 min)
  standard - Standard testing (10x10 grid, 10 robots, 30 min)
  ...
```

---

## Generated File

The generated YAML file contains all configuration sections:

```yaml
seed: 42
simulation:
  duration_minutes: 30.0
  warmup_minutes: 5.0
map:
  file: map.json
storage:
  file: storage.yaml
robots:
  count: 10
  max_speed_mps: 1.5
  max_payload_kg: 25.0
stations:
  - id: "S0"
    node: "0"
    type: pick
    concurrency: 1
    service_time_s:
      distribution: constant
      base: 5.0
      per_item: 2.0
# ... additional stations ...
orders:
  arrival_process:
    type: poisson
    rate_per_min: 1.0
  lines_per_order:
    type: negbinomial
    mean: 3.0
    dispersion: 1.0
  sku_popularity:
    type: zipf
    alpha: 1.0
policies:
  task_allocation:
    type: nearest_robot
  station_assignment:
    type: least_queue
  batching:
    type: none
  priority:
    type: strict_priority
traffic:
  policy: wait_at_node
  edge_capacity_default: 1
  node_capacity_default: 1
routing:
  algorithm: dijkstra
  cache_routes: true
metrics:
  sample_interval_s: 60.0
```

---

## Workflow

### Generate and customize

```bash
# Generate base scenario
waremax generate --preset standard --output scenario.yaml

# Edit manually for additional customization
nano scenario.yaml

# Validate
waremax validate --scenario scenario.yaml

# Run
waremax run --scenario scenario.yaml
```

### Generate variants

```bash
# Generate multiple variants
waremax generate --preset standard --output variant_5_robots.yaml --robots 5
waremax generate --preset standard --output variant_10_robots.yaml --robots 10
waremax generate --preset standard --output variant_15_robots.yaml --robots 15

# Compare
waremax compare --baseline variant_10_robots.yaml --variant variant_15_robots.yaml
```

---

## See Also

- [list-presets](list-presets.md) - View available presets
- [run](run.md) - Run simulations
- [Working with Presets](../user-guide/presets.md) - Preset details
