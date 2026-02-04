# Installation

This guide covers how to install Waremax on your system.

---

## System Requirements

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| **Rust** | 1.70+ | Latest stable |
| **Memory** | 4 GB | 8+ GB |
| **Disk** | 500 MB | 1+ GB |
| **OS** | Linux, macOS, Windows | Linux |

---

## Installation Methods

### From Source (Recommended)

Clone the repository and build with Cargo:

```bash
# Clone the repository
git clone https://github.com/waremax/waremax.git
cd waremax

# Build in release mode
cargo build --release

# Install to your path
cargo install --path .
```

### Verify Installation

After installation, verify Waremax is working:

```bash
# Check version
waremax --version

# Should output: waremax 0.1.0

# Run a quick demo
waremax demo --duration 1
```

---

## Build from Source Details

### Debug Build

For development and debugging:

```bash
cargo build
./target/debug/waremax --help
```

### Release Build

For production performance:

```bash
cargo build --release
./target/release/waremax --help
```

!!! tip "Performance"
    Always use release builds for actual simulations. Debug builds can be 10-50x slower.

### Running Tests

Verify the build with tests:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

---

## Project Structure

After cloning, the repository structure is:

```
waremax/
├── Cargo.toml              # Workspace configuration
├── src/                    # Main CLI binary
│   └── main.rs
├── crates/                 # Library crates
│   ├── waremax-core/       # Core types and DES kernel
│   ├── waremax-map/        # Warehouse topology
│   ├── waremax-storage/    # Storage and inventory
│   ├── waremax-entities/   # Robots, stations, tasks
│   ├── waremax-policies/   # Dispatching policies
│   ├── waremax-config/     # Configuration parsing
│   ├── waremax-metrics/    # KPI collection
│   ├── waremax-sim/        # Simulation runner
│   ├── waremax-testing/    # Testing utilities
│   └── waremax-analysis/   # Root cause analysis
├── docs/                   # Design documentation
├── examples/               # Example scenarios
└── documentation/          # User documentation (this site)
```

---

## Troubleshooting

### Common Issues

#### Rust Not Found

```
error: rustc not found
```

**Solution**: Install Rust via rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Build Fails with Memory Error

```
error: could not compile - memory allocation failed
```

**Solution**: Close other applications and retry, or increase swap space.

#### Permission Denied

```
error: permission denied
```

**Solution**: Don't use `sudo` with cargo. Ensure `~/.cargo/bin` is in your PATH.

---

## Updating

To update to the latest version:

```bash
cd waremax
git pull origin main
cargo build --release
cargo install --path . --force
```

---

## Uninstalling

To remove Waremax:

```bash
# Remove the binary
cargo uninstall waremax

# Optionally remove the source directory
rm -rf /path/to/waremax
```

---

## Next Steps

Once installed, proceed to the [Quick Start](quick-start.md) guide to run your first simulation.
