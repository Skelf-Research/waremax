# waremax demo

Run a quick demo simulation with default settings.

---

## Synopsis

```bash
waremax demo [OPTIONS]
```

---

## Description

The `demo` command runs a quick simulation with sensible defaults. It's useful for:

- Quick testing and exploration
- Verifying installation
- Demonstrations
- Learning the system

No configuration file is required.

---

## Options

| Option | Default | Description |
|--------|---------|-------------|
| `--duration`, `-d` | 60 | Duration in minutes |
| `--robots`, `-r` | 5 | Number of robots |
| `--order-rate` | 4.0 | Order arrival rate per minute |

---

## Examples

### Run with defaults

```bash
waremax demo
```

### Shorter simulation

```bash
waremax demo --duration 10
```

### More robots

```bash
waremax demo --robots 10
```

### Higher order rate

```bash
waremax demo --order-rate 8.0
```

### Custom configuration

```bash
waremax demo --duration 30 --robots 8 --order-rate 6.0
```

---

## Default Configuration

The demo creates:

- **Grid**: 5x5 nodes (3m spacing)
- **Stations**: 1 pick station at node 0
- **Robots**: Configurable (default 5)
  - Speed: 1.5 m/s
  - Payload: 25 kg
- **Traffic**: 2 robots per edge/node
- **Policies**: Default (nearest_robot, least_queue)

---

## Output

```
Running demo simulation...
  Duration: 60 minutes
  Robots: 5
  Order rate: 4.0/min

Distributions:
  Arrivals: Poisson(rate=0.067/s)
  Lines/Order: NegBinomial(mean=2.0, dispersion=1.0)
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
  Completed: 195
  Throughput: 212.7 orders/hr

Cycle Times:
  Average: 38.5s
  P95: 65.2s
  P99: 78.4s

Utilization:
  Robot Fleet: 58.3%
  Stations: 62.1%
```

---

## Use Cases

### Quick Test

```bash
# Verify Waremax is working
waremax demo --duration 1
```

### Explore Scaling

```bash
# Test with few robots
waremax demo --robots 2 --duration 10

# Test with more robots
waremax demo --robots 10 --duration 10

# Test with many robots
waremax demo --robots 20 --duration 10
```

### Demonstrate to Others

```bash
# Run visible demonstration
waremax demo --duration 5 --robots 8 --order-rate 5.0
```

---

## Limitations

The demo command:

- Uses a fixed simple grid layout
- Has only one pick station
- Cannot export detailed results
- Cannot customize policies

For more control, use [generate](generate.md) to create a scenario file.

---

## See Also

- [run](run.md) - Run from scenario file
- [generate](generate.md) - Generate scenario files
- [list-presets](list-presets.md) - View available presets
