# Crate Structure

Organization of the Waremax workspace.

---

## Workspace Overview

Waremax is a Cargo workspace with multiple crates:

```
waremax/
├── Cargo.toml              # Workspace root
├── src/                    # Main binary
│   └── main.rs
└── crates/
    ├── waremax-core/       # DES engine
    ├── waremax-config/     # Configuration
    ├── waremax-map/        # Map/topology
    ├── waremax-storage/    # Inventory
    ├── waremax-entities/   # Robots, stations
    ├── waremax-policies/   # Decision policies
    ├── waremax-metrics/    # Metrics collection
    ├── waremax-sim/        # Simulation orchestration
    ├── waremax-testing/    # Test utilities, presets
    └── waremax-analysis/   # Result analysis
```

---

## Crate Dependencies

```
waremax (binary)
    │
    ├── waremax-sim ─────────────────┐
    │       │                        │
    │       ├── waremax-core         │
    │       ├── waremax-entities ────┤
    │       │       │                │
    │       │       └── waremax-map  │
    │       │                        │
    │       ├── waremax-policies     │
    │       └── waremax-metrics      │
    │                                │
    ├── waremax-config ──────────────┤
    │                                │
    ├── waremax-testing ─────────────┤
    │                                │
    └── waremax-analysis ────────────┘
```

---

## Crate Details

### waremax-core

The discrete event simulation engine.

**Responsibilities:**

- Event scheduling and dispatch
- Time management
- Simulation lifecycle

**Key Types:**

```rust
pub struct Scheduler { ... }
pub struct SimTime(f64);
pub trait Event { ... }
```

**Dependencies:** Minimal (std only)

---

### waremax-config

Configuration parsing and validation.

**Responsibilities:**

- YAML parsing
- Schema validation
- Default values
- Error reporting

**Key Types:**

```rust
pub struct Scenario { ... }
pub struct MapConfig { ... }
pub struct RobotConfig { ... }
```

**Dependencies:** serde, serde_yaml

---

### waremax-map

Warehouse topology and pathfinding.

**Responsibilities:**

- Graph representation
- Shortest path calculation
- Connectivity analysis

**Key Types:**

```rust
pub struct WarehouseMap { ... }
pub struct Node { ... }
pub struct Edge { ... }
pub trait PathFinder { ... }
```

**Dependencies:** petgraph

---

### waremax-storage

Inventory and storage management.

**Responsibilities:**

- Rack and bin modeling
- SKU placement
- Inventory tracking

**Key Types:**

```rust
pub struct Rack { ... }
pub struct Bin { ... }
pub struct Inventory { ... }
```

**Dependencies:** waremax-map

---

### waremax-entities

Simulation entities (robots, stations, tasks).

**Responsibilities:**

- Entity definitions
- State management
- Behavior implementation

**Key Types:**

```rust
pub struct Robot { ... }
pub struct Station { ... }
pub struct Task { ... }
pub struct Order { ... }
```

**Dependencies:** waremax-map, waremax-storage

---

### waremax-policies

Decision-making policies.

**Responsibilities:**

- Policy trait definitions
- Built-in policy implementations
- Policy composition

**Key Types:**

```rust
pub trait TaskAllocationPolicy { ... }
pub trait StationAssignmentPolicy { ... }
pub trait RoutingPolicy { ... }

pub struct NearestIdlePolicy { ... }
pub struct ShortestQueuePolicy { ... }
```

**Dependencies:** waremax-entities

---

### waremax-metrics

Metrics collection and aggregation.

**Responsibilities:**

- Event-driven metric collection
- Time series recording
- Aggregation and statistics

**Key Types:**

```rust
pub struct MetricsCollector { ... }
pub struct TaskMetrics { ... }
pub struct RobotMetrics { ... }
```

**Dependencies:** waremax-entities

---

### waremax-sim

Simulation orchestration.

**Responsibilities:**

- Combine all components
- Run simulation loop
- Handle events

**Key Types:**

```rust
pub struct Simulation { ... }
pub struct SimulationBuilder { ... }
pub struct SimulationResult { ... }
```

**Dependencies:** All other crates

---

### waremax-testing

Test utilities and presets.

**Responsibilities:**

- Preset scenarios
- Test helpers
- Fixture generation

**Key Types:**

```rust
pub enum Preset { ... }
pub fn load_preset(preset: Preset) -> Scenario { ... }
```

**Dependencies:** waremax-config

---

### waremax-analysis

Result analysis and comparison.

**Responsibilities:**

- Statistical analysis
- Result comparison
- Report generation

**Key Types:**

```rust
pub struct AnalysisReport { ... }
pub fn compare(a: &Results, b: &Results) -> Comparison { ... }
```

**Dependencies:** waremax-metrics

---

## Adding a New Crate

### 1. Create Crate

```bash
cd crates
cargo new waremax-mycrate --lib
```

### 2. Update Workspace

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "crates/waremax-mycrate",
    # ...
]
```

### 3. Add Dependencies

```toml
# crates/waremax-mycrate/Cargo.toml
[dependencies]
waremax-core = { path = "../waremax-core" }
```

### 4. Export Public API

```rust
// crates/waremax-mycrate/src/lib.rs
pub mod feature;
pub use feature::MyFeature;
```

---

## Versioning

All crates share the same version:

```toml
# Cargo.toml
[workspace.package]
version = "0.1.0"

# Each crate
[package]
version.workspace = true
```

---

## Testing

Each crate has its own tests:

```bash
# Test single crate
cargo test -p waremax-core

# Test all crates
cargo test --workspace
```

---

## Related

- [Architecture Overview](overview.md)
- [Data Flow](data-flow.md)
