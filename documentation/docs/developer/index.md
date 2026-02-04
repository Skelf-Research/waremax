# Developer Guide

Building and extending Waremax.

---

## Overview

This guide covers:

- Understanding the codebase architecture
- Extending Waremax with custom components
- Using the crate APIs
- Contributing to the project

---

## Guide Sections

### Architecture

Understand how Waremax is built.

| Topic | Description |
|-------|-------------|
| [Overview](architecture/overview.md) | High-level architecture |
| [Crate Structure](architecture/crates.md) | Workspace organization |
| [Data Flow](architecture/data-flow.md) | How data moves through the system |

### Extending Waremax

Add custom functionality.

| Topic | Description |
|-------|-------------|
| [Custom Policies](extending/custom-policies.md) | Implement new decision policies |
| [Custom Distributions](extending/custom-distributions.md) | Add statistical distributions |
| [Custom Entities](extending/custom-entities.md) | Create new simulation entities |

### API Reference

Crate documentation.

| Crate | Description |
|-------|-------------|
| [waremax-core](api/core.md) | Core simulation engine |
| [waremax-config](api/config.md) | Configuration parsing |
| [waremax-entities](api/entities.md) | Simulation entities |
| [waremax-policies](api/policies.md) | Decision policies |
| [waremax-metrics](api/metrics.md) | Metrics collection |

### Contributing

Help improve Waremax.

| Topic | Description |
|-------|-------------|
| [Code Style](contributing/code-style.md) | Coding guidelines |
| [Testing](contributing/testing.md) | Test requirements |
| [Documentation](contributing/documentation.md) | Doc standards |

---

## Quick Links

### For Users Wanting to Extend

1. Read [Architecture Overview](architecture/overview.md)
2. Choose extension type:
   - [Custom Policies](extending/custom-policies.md)
   - [Custom Distributions](extending/custom-distributions.md)
3. Follow implementation guide

### For Contributors

1. Read [Contributing Guide](contributing/code-style.md)
2. Understand [Testing Requirements](contributing/testing.md)
3. Follow [Pull Request Process](contributing/code-style.md#pull-requests)

---

## Development Setup

### Prerequisites

```bash
# Rust toolchain (1.70+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/example/waremax.git
cd waremax
```

### Build

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test
```

### Development Workflow

```bash
# Format code
cargo fmt

# Run lints
cargo clippy

# Build docs
cargo doc --open
```

---

## Crate Dependencies

```
waremax (binary)
├── waremax-sim
│   ├── waremax-core
│   ├── waremax-entities
│   ├── waremax-policies
│   └── waremax-metrics
├── waremax-config
├── waremax-testing
├── waremax-analysis
├── waremax-map
└── waremax-storage
```

---

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/example/waremax/issues)
- **Discussions**: [GitHub Discussions](https://github.com/example/waremax/discussions)
- **API Docs**: `cargo doc --open`
