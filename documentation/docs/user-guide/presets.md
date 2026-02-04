# Working with Presets

Presets are predefined scenario configurations for common testing patterns.

---

## Available Presets

| Preset | Description | Robots | Grid | Order Rate | Duration |
|--------|-------------|--------|------|------------|----------|
| `minimal` | Unit tests and debugging | 1 | 3x3 | 10/hr | 1 min |
| `quick` | Fast iteration testing | 3 | 5x5 | 30/hr | 5 min |
| `standard` | General testing | 10 | 10x10 | 60/hr | 30 min |
| `baseline` | Reproducible comparisons | 10 | 10x10 | 60/hr | 60 min |
| `high_load` | Stress testing | 50 | 20x20 | 300/hr | 60 min |
| `peak_hours` | Peak demand simulation | 20 | 15x15 | 200/hr | 120 min |
| `stress_test` | Maximum load testing | 100 | 30x30 | 500/hr | 120 min |
| `battery_test` | Battery/charging focused | 15 | 10x10 | 60/hr | 120 min |
| `maintenance_test` | Reliability testing | 20 | 10x10 | 60/hr | 480 min |

---

## Listing Presets

View all available presets:

```bash
waremax list-presets
```

Output:

```
Available Scenario Presets:
======================================================================

minimal
  Minimal config for unit tests (3x3 grid, 1 robot, 1 station)
  Robots: 1
  Stations: 1
  Order Rate: 10 orders/hr
  Duration: 1 min (warmup: 0 min)

quick
  Quick iteration testing (5x5 grid, 3 robots, 5 min)
  Robots: 3
  Stations: 2
  Order Rate: 30 orders/hr
  Duration: 5 min (warmup: 1 min)

standard
  Standard testing (10x10 grid, 10 robots, 30 min)
  Robots: 10
  Stations: 4
  Order Rate: 60 orders/hr
  Duration: 30 min (warmup: 5 min)

...
```

---

## Generating from Presets

Create a scenario file from a preset:

```bash
waremax generate --preset standard --output my_scenario.yaml
```

### Overriding Parameters

Customize during generation:

```bash
# Change robot count
waremax generate --preset standard --output scenario.yaml --robots 15

# Change order rate
waremax generate --preset standard --output scenario.yaml --order-rate 100

# Change grid size
waremax generate --preset standard --output scenario.yaml --grid 15x15

# Set specific seed
waremax generate --preset standard --output scenario.yaml --seed 12345

# Combine multiple overrides
waremax generate --preset standard --output scenario.yaml \
  --robots 20 \
  --order-rate 120 \
  --grid 12x12 \
  --seed 42
```

---

## Preset Details

### minimal

**Purpose**: Quick unit tests and debugging

```yaml
# Generated configuration
seed: auto
simulation:
  duration_minutes: 1
  warmup_minutes: 0
robots:
  count: 1
stations: 1 pick station
orders:
  rate_per_min: 0.167  # 10/hr
```

**Use when**: Running quick tests, debugging issues, CI/CD pipelines

### quick

**Purpose**: Fast iteration during development

```yaml
simulation:
  duration_minutes: 5
  warmup_minutes: 1
robots:
  count: 3
stations: 2 pick stations
orders:
  rate_per_min: 0.5  # 30/hr
```

**Use when**: Iterating on configuration changes, quick experiments

### standard

**Purpose**: General-purpose testing

```yaml
simulation:
  duration_minutes: 30
  warmup_minutes: 5
robots:
  count: 10
stations: 4 pick stations
orders:
  rate_per_min: 1.0  # 60/hr
```

**Use when**: Standard experiments, policy comparisons, general analysis

### baseline

**Purpose**: Reproducible baseline for comparisons

```yaml
seed: 12345  # Fixed seed!
simulation:
  duration_minutes: 60
  warmup_minutes: 60
robots:
  count: 10
stations: 4 pick stations
```

**Use when**: Creating reproducible benchmarks, A/B testing baselines

### high_load

**Purpose**: High-volume stress testing

```yaml
simulation:
  duration_minutes: 60
  warmup_minutes: 10
robots:
  count: 50
stations: 10 pick stations (concurrency: 2)
orders:
  rate_per_min: 5.0  # 300/hr
traffic:
  policy: reroute_on_wait
routing:
  congestion_aware: true
```

**Use when**: Testing system under heavy load, capacity planning

### peak_hours

**Purpose**: Simulating peak demand periods

```yaml
simulation:
  duration_minutes: 120
  warmup_minutes: 15
robots:
  count: 20
stations: 6 pick stations (concurrency: 2)
orders:
  rate_per_min: 3.33  # 200/hr
service_time: lognormal distribution
```

**Use when**: Planning for peak periods, testing surge capacity

### stress_test

**Purpose**: Maximum stress testing

```yaml
simulation:
  duration_minutes: 120
  warmup_minutes: 20
robots:
  count: 100
stations: 20 pick stations (concurrency: 3)
orders:
  rate_per_min: 8.33  # 500/hr
  items_per_order: 5
sku_count: 500
traffic:
  policy: reroute_on_wait
routing:
  algorithm: astar
  congestion_aware: true
```

**Use when**: Finding system limits, extreme capacity testing

### battery_test

**Purpose**: Testing battery and charging systems

```yaml
simulation:
  duration_minutes: 120
  warmup_minutes: 10
robots:
  count: 15
  battery:
    enabled: true
    capacity_wh: 400
    min_soc: 0.15
charging_stations:
  count: 4
  bays_each: 2
  charge_rate_w: 200
```

**Use when**: Evaluating charging station placement, battery life analysis

### maintenance_test

**Purpose**: Testing maintenance and reliability

```yaml
simulation:
  duration_minutes: 480  # 8 hours
  warmup_minutes: 30
robots:
  count: 20
  maintenance:
    enabled: true
    interval_hours: 8.0
  failure:
    enabled: true
    mtbf_hours: 50.0
maintenance_stations:
  count: 2
  bays_each: 2
```

**Use when**: Testing maintenance scheduling, failure recovery, uptime analysis

---

## Choosing a Preset

### For Development

| Stage | Preset |
|-------|--------|
| Initial debugging | `minimal` |
| Quick iteration | `quick` |
| Full testing | `standard` |

### For Analysis

| Goal | Preset |
|------|--------|
| General experiments | `standard` |
| Reproducible comparison | `baseline` |
| Capacity planning | `high_load` |
| Peak planning | `peak_hours` |

### For Specialized Testing

| Focus | Preset |
|-------|--------|
| Battery/charging | `battery_test` |
| Maintenance/reliability | `maintenance_test` |
| System limits | `stress_test` |

---

## Customizing Presets

After generating, edit the YAML file for further customization:

```bash
# Generate base scenario
waremax generate --preset standard --output scenario.yaml

# Edit for specific needs
# - Add custom policies
# - Change service time distributions
# - Add maintenance stations
# - Etc.

# Run customized scenario
waremax run --scenario scenario.yaml
```

---

## Creating Your Own Presets

While Waremax doesn't support user-defined presets, you can:

1. Generate from closest preset
2. Customize the YAML
3. Save as template
4. Copy and modify for variations

```bash
# Create template
waremax generate --preset standard --output templates/my_warehouse.yaml

# Create variations
cp templates/my_warehouse.yaml scenarios/variation_a.yaml
cp templates/my_warehouse.yaml scenarios/variation_b.yaml

# Edit each variation
```

---

## Next Steps

- **[Export Formats](export-formats.md)** - Understanding output files
- **[CLI Reference](../cli/generate.md)** - Complete generate command reference
- **[Tutorials](../tutorials/index.md)** - Step-by-step guides
