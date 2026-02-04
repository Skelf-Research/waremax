# waremax list-presets

List available scenario presets.

---

## Synopsis

```bash
waremax list-presets
```

---

## Description

The `list-presets` command displays all available predefined scenario configurations. Each preset shows its purpose, key parameters, and use case.

---

## Options

This command has no options.

---

## Examples

### List all presets

```bash
waremax list-presets
```

---

## Output

```
Available Scenario Presets:
======================================================================

minimal
  Minimal config for unit tests (3x3 grid, 1 robot, 1 station)
  Grid: inferred from stations
  Robots: 1
  Stations: 1
  Order Rate: 10 orders/hr
  Duration: 1 min (warmup: 0 min)

quick
  Quick iteration testing (5x5 grid, 3 robots, 5 min)
  Grid: inferred from stations
  Robots: 3
  Stations: 2
  Order Rate: 30 orders/hr
  Duration: 5 min (warmup: 1 min)

standard
  Standard testing (10x10 grid, 10 robots, 30 min)
  Grid: inferred from stations
  Robots: 10
  Stations: 4
  Order Rate: 60 orders/hr
  Duration: 30 min (warmup: 5 min)

baseline
  Reproducible baseline (10x10, fixed seed, 60 min warmup)
  Grid: inferred from stations
  Robots: 10
  Stations: 4
  Order Rate: 60 orders/hr
  Duration: 60 min (warmup: 60 min)

high_load
  High load stress test (20x20, 50 robots, 300 orders/hr)
  Grid: inferred from stations
  Robots: 50
  Stations: 10
  Order Rate: 300 orders/hr
  Duration: 60 min (warmup: 10 min)

peak_hours
  Peak hours simulation (15x15, 20 robots, 200 orders/hr)
  Grid: inferred from stations
  Robots: 20
  Stations: 6
  Order Rate: 200 orders/hr
  Duration: 120 min (warmup: 15 min)

stress_test
  Maximum stress test (30x30, 100 robots, 500 orders/hr)
  Grid: inferred from stations
  Robots: 100
  Stations: 20
  Order Rate: 500 orders/hr
  Duration: 120 min (warmup: 20 min)

battery_test
  Battery/charging focused (15 robots, 4 chargers)
  Grid: inferred from stations
  Robots: 15
  Stations: 4
  Order Rate: 60 orders/hr
  Duration: 120 min (warmup: 10 min)
  Battery: enabled (400 Wh)

maintenance_test
  Maintenance/reliability focused (8 hour sim)
  Grid: inferred from stations
  Robots: 20
  Stations: 4
  Order Rate: 60 orders/hr
  Duration: 480 min (warmup: 30 min)
  Maintenance: enabled (8.0 hr interval)
```

---

## Preset Summary

| Preset | Robots | Order Rate | Duration | Special |
|--------|--------|------------|----------|---------|
| `minimal` | 1 | 10/hr | 1 min | Fastest |
| `quick` | 3 | 30/hr | 5 min | Quick iteration |
| `standard` | 10 | 60/hr | 30 min | General use |
| `baseline` | 10 | 60/hr | 60 min | Fixed seed |
| `high_load` | 50 | 300/hr | 60 min | Stress test |
| `peak_hours` | 20 | 200/hr | 120 min | Peak simulation |
| `stress_test` | 100 | 500/hr | 120 min | Maximum load |
| `battery_test` | 15 | 60/hr | 120 min | Battery enabled |
| `maintenance_test` | 20 | 60/hr | 480 min | Maintenance enabled |

---

## Using Presets

### Generate from preset

```bash
waremax generate --preset standard --output my_scenario.yaml
```

### Override parameters

```bash
waremax generate --preset standard --output my_scenario.yaml \
  --robots 20 \
  --order-rate 100
```

---

## Choosing a Preset

### By Development Stage

| Stage | Preset |
|-------|--------|
| Unit testing | `minimal` |
| Development | `quick` |
| Integration testing | `standard` |
| Release testing | `baseline` |

### By Analysis Goal

| Goal | Preset |
|------|--------|
| Capacity planning | `high_load` |
| Peak hour analysis | `peak_hours` |
| System limits | `stress_test` |
| Battery analysis | `battery_test` |
| Reliability analysis | `maintenance_test` |

---

## See Also

- [generate](generate.md) - Generate scenarios from presets
- [Working with Presets](../user-guide/presets.md) - Detailed preset guide
- [run](run.md) - Run simulations
