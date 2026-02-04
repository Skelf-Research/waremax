# Waremax Documentation

**Waremax** is a discrete-event simulation framework for warehouse robot operations. It enables you to model, simulate, and optimize multi-robot warehouse systems before making physical changes or implementing new policies in production.

---

## What is Waremax?

Waremax provides a comprehensive simulation environment for warehouse automation:

- **Discrete Event Simulation (DES)** - Event-driven simulation for accurate modeling without time-stepping overhead
- **Multi-Robot Fleet Management** - Simulate dozens to hundreds of robots with realistic movement and task execution
- **Configurable Policies** - Experiment with different task allocation, station assignment, and batching strategies
- **Traffic Management** - Model congestion, deadlocks, and routing in constrained warehouse environments
- **Comprehensive Metrics** - Track throughput, cycle times, utilization, and identify bottlenecks

---

## Key Features

### Simulation Capabilities

| Feature | Description |
|---------|-------------|
| **Deterministic Simulation** | Same seed + config = reproducible results |
| **Event-Driven Architecture** | Efficient simulation without fixed time steps |
| **Configurable Scenarios** | YAML-based configuration for easy experimentation |
| **Multiple Order Types** | Pick, putaway, replenishment workflows |

### Robot & Station Modeling

| Feature | Description |
|---------|-------------|
| **Fleet Management** | Multiple robots with configurable speed and payload |
| **Battery Simulation** | Optional battery/charging station modeling |
| **Maintenance & Failures** | Scheduled maintenance and random failure simulation |
| **Station Queuing** | Realistic station service times with queue management |

### Analysis & Testing

| Feature | Description |
|---------|-------------|
| **Parameter Sweeps** | Systematic exploration of configuration space |
| **A/B Testing** | Statistical comparison between configurations |
| **Root Cause Analysis** | Automatic bottleneck detection and recommendations |
| **Benchmark Suites** | Performance regression detection |

---

## Quick Start

Get started with Waremax in minutes:

```bash
# Install from source
cargo install --path .

# Run a quick demo
waremax demo --duration 60 --robots 5

# Generate a scenario from a preset
waremax generate --preset standard --output my_scenario.yaml

# Run a simulation
waremax run --scenario my_scenario.yaml --output json
```

For detailed installation instructions, see the [Installation Guide](getting-started/installation.md).

---

## Documentation Overview

### For Users

- **[Getting Started](getting-started/index.md)** - Installation, quick start, and first simulation
- **[User Guide](user-guide/index.md)** - Running simulations, configuring scenarios, working with presets
- **[CLI Reference](cli/index.md)** - Complete command-line interface documentation
- **[Configuration Reference](configuration/index.md)** - All configuration options explained
- **[Tutorials](tutorials/index.md)** - Step-by-step guides for common tasks

### For Understanding

- **[Concepts](concepts/index.md)** - Deep dive into simulation model, policies, and metrics

### For Developers

- **[Developer Guide](developer/index.md)** - Architecture, extending Waremax, contributing

---

## Use Cases

Waremax is designed for:

- **Capacity Planning** - Determine optimal robot counts and station configurations
- **Policy Optimization** - Compare task allocation and batching strategies
- **What-If Analysis** - Evaluate layout changes before physical implementation
- **Performance Benchmarking** - Track simulation performance over time
- **Training & Education** - Learn warehouse automation concepts

---

## Example Output

```
Simulation Complete
==================
Duration: 60.0 minutes (warmup: 5.0 minutes)

Orders:
  Completed: 245
  Throughput: 245.0 orders/hr

Cycle Times:
  Average: 42.3s
  P95: 78.5s
  P99: 95.2s

Utilization:
  Robot Fleet: 67.2%
  Stations: 72.5%
```

---

## Next Steps

1. **[Install Waremax](getting-started/installation.md)** - Set up Waremax on your system
2. **[Run Your First Simulation](getting-started/first-simulation.md)** - Step-by-step tutorial
3. **[Explore Presets](user-guide/presets.md)** - Use built-in scenario configurations
4. **[Learn the Concepts](concepts/index.md)** - Understand how the simulation works

---

## Technology

Waremax is built with:

- **Rust** - Memory-safe, high-performance systems language
- **rkyv** - Zero-copy serialization for fast state management
- **sled** - Embedded ACID-compliant database
- **mimalloc** - High-performance memory allocator

---

## License

Waremax is released under the MIT License.
