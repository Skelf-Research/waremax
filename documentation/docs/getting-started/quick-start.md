# Quick Start

Get up and running with Waremax in 5 minutes.

---

## Run a Demo

The fastest way to see Waremax in action is the demo command:

```bash
waremax demo
```

This runs a 60-minute simulation with:

- 5x5 grid warehouse
- 5 robots
- 4 orders per minute
- 1 pick station

### Customize the Demo

```bash
# Longer simulation
waremax demo --duration 120

# More robots
waremax demo --robots 10

# Higher order rate
waremax demo --order-rate 8.0

# Combine options
waremax demo --duration 30 --robots 8 --order-rate 6.0
```

---

## List Available Presets

Waremax includes predefined scenario configurations:

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
  Duration: 1 min

quick
  Quick iteration testing (5x5 grid, 3 robots, 5 min)
  Robots: 3
  Stations: 2
  Order Rate: 30 orders/hr
  Duration: 5 min

standard
  Standard testing (10x10 grid, 10 robots, 30 min)
  Robots: 10
  Stations: 4
  Order Rate: 60 orders/hr
  Duration: 30 min
...
```

---

## Generate a Scenario

Create a scenario file from a preset:

```bash
# Generate a standard scenario
waremax generate --preset standard --output my_scenario.yaml
```

This creates `my_scenario.yaml` with the standard configuration.

### Customize During Generation

Override preset defaults:

```bash
# More robots
waremax generate --preset standard --output scenario.yaml --robots 20

# Different order rate
waremax generate --preset standard --output scenario.yaml --order-rate 120

# Larger grid
waremax generate --preset standard --output scenario.yaml --grid 20x20

# Fixed seed for reproducibility
waremax generate --preset standard --output scenario.yaml --seed 12345
```

---

## Run a Simulation

Run your generated scenario:

```bash
waremax run --scenario my_scenario.yaml
```

### Output Formats

```bash
# Text output (default)
waremax run --scenario my_scenario.yaml --output text

# JSON output
waremax run --scenario my_scenario.yaml --output json
```

### Export Data

Export detailed results to files:

```bash
waremax run --scenario my_scenario.yaml \
  --output-dir ./results \
  --per-robot \
  --per-station \
  --heatmap \
  --timeseries
```

This creates:

- `results/report.json` - Full simulation report
- `results/robots.csv` - Per-robot metrics
- `results/stations.csv` - Per-station metrics
- `results/node_congestion.csv` - Node congestion data
- `results/edge_congestion.csv` - Edge congestion data
- `results/timeseries.csv` - Time series data

---

## Validate a Scenario

Check a scenario file for errors before running:

```bash
waremax validate --scenario my_scenario.yaml
```

Output for a valid scenario:

```
Validating scenario: my_scenario.yaml
Scenario valid!
  Seed: 42
  Duration: 30 minutes
  Warmup: 5 minutes
  Robot count: 10
  Stations: 4
```

---

## Quick Command Reference

| Command | Purpose |
|---------|---------|
| `waremax demo` | Run a quick demo simulation |
| `waremax list-presets` | Show available scenario presets |
| `waremax generate` | Create a scenario from a preset |
| `waremax run` | Execute a simulation |
| `waremax validate` | Check scenario for errors |
| `waremax --help` | Show all commands |

---

## Example Workflow

A typical workflow for experimenting with warehouse configurations:

```bash
# 1. Generate a baseline scenario
waremax generate --preset baseline --output baseline.yaml

# 2. Validate the scenario
waremax validate --scenario baseline.yaml

# 3. Run the simulation
waremax run --scenario baseline.yaml --output json > baseline_results.json

# 4. Create a variant with more robots
waremax generate --preset baseline --output variant.yaml --robots 15

# 5. Compare the two configurations
waremax compare --baseline baseline.yaml --variant variant.yaml --replications 5
```

---

## Next Steps

- **[Your First Simulation](first-simulation.md)** - Create a custom scenario from scratch
- **[Understanding Output](understanding-output.md)** - Learn to interpret results
- **[User Guide](../user-guide/index.md)** - Detailed usage instructions
