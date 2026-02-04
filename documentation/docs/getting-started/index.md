# Getting Started

Welcome to Waremax! This section will help you get up and running with warehouse robot simulation.

---

## Overview

Getting started with Waremax involves four steps:

1. **[Installation](installation.md)** - Install Waremax on your system
2. **[Quick Start](quick-start.md)** - Run your first commands in 5 minutes
3. **[Your First Simulation](first-simulation.md)** - Create and run a custom scenario
4. **[Understanding Output](understanding-output.md)** - Interpret simulation results

---

## Prerequisites

Before installing Waremax, ensure you have:

- **Rust 1.70+** - Waremax is built with Rust
- **Cargo** - Rust's package manager (included with Rust)
- **Git** - For cloning the repository

### Installing Rust

If you don't have Rust installed:

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

---

## Quick Overview

Waremax provides a CLI for running warehouse simulations:

```bash
# Run a quick demo
waremax demo

# List available presets
waremax list-presets

# Generate a scenario
waremax generate --preset standard --output scenario.yaml

# Run a simulation
waremax run --scenario scenario.yaml

# Validate a scenario
waremax validate --scenario scenario.yaml
```

---

## What You'll Learn

By the end of this section, you'll be able to:

- Install and verify Waremax installation
- Run demo simulations
- Create scenario configuration files
- Execute simulations and interpret results
- Export simulation data for analysis

---

## Next Steps

Start with the [Installation Guide](installation.md) to set up Waremax on your system.
