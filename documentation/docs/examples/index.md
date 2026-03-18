# Examples

Ready-to-use scenario examples and case studies.

---

## Overview

This section provides complete examples you can use as starting points for your own simulations.

| Example | Description |
|---------|-------------|
| [Scenario Examples](scenarios.md) | Complete scenario configurations |
| [Policy Comparisons](policy-comparisons.md) | Comparing different policies |
| [Capacity Studies](capacity-studies.md) | Fleet and station sizing |

---

## Quick Examples

### Minimal Simulation

```bash
waremax run --preset minimal
```

```yaml
# Equivalent configuration
simulation:
  duration_s: 60

map:
  nodes:
    - { id: 0, x: 0, y: 0, type: aisle }
    - { id: 1, x: 3, y: 0, type: rack }
    - { id: 2, x: 6, y: 0, type: station_pick }
  edges:
    - { from: 0, to: 1, bidirectional: true }
    - { from: 1, to: 2, bidirectional: true }

robots:
  count: 1
  speed_m_s: 1.5

stations:
  - id: S1
    node: 2
    type: pick

orders:
  generation:
    type: constant
    rate_per_hour: 60
```

### Standard Warehouse

```bash
waremax run --preset standard
```

10 robots, 2 stations, 1-hour simulation.

### High Load Test

```bash
waremax run --preset high_load
```

25 robots, high order rate for stress testing.

---

## Example Files

Download example files:

```bash
# Generate example scenarios
waremax generate scenario --type minimal > minimal.yaml
waremax generate scenario --type standard > standard.yaml
waremax generate scenario --type warehouse > warehouse.yaml
```

---

## Running Examples

### Basic Run

```bash
waremax run example.yaml
```

### With Output

```bash
waremax run example.yaml -o results/
```

### With Modifications

```bash
waremax run example.yaml --param robots.count=20
```

---

## Example Categories

### By Complexity

| Level | Examples |
|-------|----------|
| Beginner | Minimal, Quick |
| Intermediate | Standard, Custom Map |
| Advanced | Multi-zone, High Load |

### By Use Case

| Use Case | Example |
|----------|---------|
| Learning | Minimal, Standard |
| Testing | High Load, Stress Test |
| Planning | Capacity Study |
| Comparison | Policy Comparison |

---

## Contributing Examples

Have a useful example? Contribute it:

1. Create a complete, working scenario
2. Add comments explaining key choices
3. Test that it runs successfully
4. Submit a pull request

---

## Related

- [Quick Start](../getting-started/quick-start.md)
- [Creating Scenarios](../tutorials/basic/creating-scenarios.md)
- [Presets](../user-guide/presets.md)
