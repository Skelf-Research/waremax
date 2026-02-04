<div align="center">

# Waremax

**High-fidelity discrete event simulation for warehouse robotics**

[![Build Status](https://img.shields.io/github/actions/workflow/status/example/waremax/ci.yml?branch=main&style=flat-square)](https://github.com/example/waremax/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg?style=flat-square)](https://www.rust-lang.org/)
[![Documentation](https://img.shields.io/badge/docs-latest-brightgreen.svg?style=flat-square)](https://example.github.io/waremax)

[Getting Started](#getting-started) •
[Documentation](https://example.github.io/waremax) •
[Examples](#examples) •
[Architecture](#architecture)

</div>

---

Waremax is a discrete event simulation framework for modeling autonomous mobile robot (AMR) operations in warehouse environments. It enables engineers and architects to evaluate throughput, identify bottlenecks, and optimize fleet sizing and policies—before deploying changes to production.

```bash
# Run a 1-hour simulation with 10 robots
waremax run --preset standard

# Compare task allocation policies
waremax compare scenario.yaml \
  --param policies.task_allocation=nearest_idle \
  --param policies.task_allocation=least_busy

# Find optimal fleet size
waremax sweep scenario.yaml --param "robots.count=[5,10,15,20,25]"
```

## Why Waremax?

| Challenge | How Waremax Helps |
|-----------|-------------------|
| **Fleet sizing** | Simulate different robot counts to find the throughput sweet spot |
| **Policy evaluation** | Compare allocation, routing, and batching strategies with statistical rigor |
| **Capacity planning** | Model peak loads and growth scenarios before scaling infrastructure |
| **Layout optimization** | Test map changes and station placements in simulation |
| **What-if analysis** | Evaluate impact of failures, maintenance windows, and demand spikes |

## Key Features

- **Deterministic simulation** — Same seed, same results. Debug and reproduce any scenario
- **Configurable policies** — Pluggable task allocation, routing, and batching strategies
- **Rich metrics** — Throughput, utilization, queue lengths, travel times, and more
- **Statistical tools** — Parameter sweeps, A/B testing, and benchmarking built-in
- **YAML configuration** — Human-readable scenarios with validation
- **High performance** — Event-driven core handles 100+ robots efficiently

## Getting Started

### Installation

```bash
# Clone and build
git clone https://github.com/example/waremax.git
cd waremax
cargo install --path .

# Verify installation
waremax --version
```

### Quick Demo

```bash
# Run a demo simulation
waremax demo

# Run with a preset
waremax run --preset standard -o results/

# Analyze results
waremax analyze results/
```

### Your First Scenario

Create `my_scenario.yaml`:

```yaml
simulation:
  duration_s: 3600
  seed: 12345

robots:
  count: 10
  speed_m_s: 1.5

stations:
  - id: "S1"
    node: 30
    type: pick
    concurrency: 2

orders:
  generation:
    type: poisson
    rate_per_hour: 300

policies:
  task_allocation: nearest_idle
  station_assignment: shortest_queue
```

```bash
waremax run my_scenario.yaml -o results/
```

## Examples

### Compare Policies

```bash
waremax compare scenario.yaml \
  --param policies.task_allocation=nearest_idle \
  --param policies.task_allocation=least_busy \
  --param policies.task_allocation=round_robin \
  --runs 10
```

### Parameter Sweep

```bash
waremax sweep scenario.yaml \
  --param "robots.count=[10,15,20,25,30]" \
  --param "policies.batching.max_batch_size=[3,5,7]"
```

### Stress Test

```bash
waremax stress-test scenario.yaml --load-multiplier 1.5
```

## Architecture

Waremax is organized as a Cargo workspace with focused crates:

```
waremax/
├── src/                    # CLI binary
└── crates/
    ├── waremax-core        # DES engine, event scheduling
    ├── waremax-config      # YAML parsing, validation
    ├── waremax-map         # Graph topology, pathfinding
    ├── waremax-entities    # Robots, stations, tasks
    ├── waremax-policies    # Allocation, routing, batching
    ├── waremax-metrics     # Collection and aggregation
    ├── waremax-sim         # Simulation orchestration
    ├── waremax-testing     # Presets, test utilities
    └── waremax-analysis    # Statistics, comparison
```

### Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Core | Rust | Memory-safe, high-performance simulation |
| Serialization | serde + YAML | Configuration and results |
| Graphs | petgraph | Map topology and pathfinding |
| RNG | rand + ChaCha | Reproducible randomness |

## Configuration Reference

### Simulation

```yaml
simulation:
  duration_s: 3600      # Simulation length
  seed: 12345           # Random seed for reproducibility
  warmup_s: 300         # Warmup period (excluded from metrics)
```

### Robots

```yaml
robots:
  count: 10             # Fleet size
  speed_m_s: 1.5        # Travel speed
```

### Policies

```yaml
policies:
  task_allocation: nearest_idle | least_busy | round_robin
  station_assignment: nearest | shortest_queue | fastest_completion

routing:
  policy: shortest_path | congestion_aware
  congestion_weight: 1.5
```

See [full configuration reference](https://example.github.io/waremax/configuration/) for all options.

## CLI Reference

| Command | Description |
|---------|-------------|
| `waremax run` | Execute a simulation |
| `waremax validate` | Check configuration |
| `waremax demo` | Run demo simulation |
| `waremax sweep` | Parameter exploration |
| `waremax compare` | Compare configurations |
| `waremax ab-test` | Statistical comparison |
| `waremax benchmark` | Performance testing |
| `waremax analyze` | Analyze results |
| `waremax list-presets` | Show available presets |

## Documentation

- [Getting Started Guide](https://example.github.io/waremax/getting-started/)
- [Configuration Reference](https://example.github.io/waremax/configuration/)
- [Concepts](https://example.github.io/waremax/concepts/)
- [Tutorials](https://example.github.io/waremax/tutorials/)
- [Developer Guide](https://example.github.io/waremax/developer/)
- [API Documentation](https://docs.rs/waremax)

## Contributing

Contributions are welcome! Please read our [Contributing Guide](https://example.github.io/waremax/developer/contributing/code-style/) before submitting a pull request.

```bash
# Development setup
git clone https://github.com/example/waremax.git
cd waremax
cargo build
cargo test
```

## License

Waremax is licensed under the [MIT License](LICENSE).

---

<div align="center">

**[Documentation](https://example.github.io/waremax)** •
**[Issues](https://github.com/example/waremax/issues)** •
**[Discussions](https://github.com/example/waremax/discussions)**

</div>
