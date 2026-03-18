# Architecture Overview

High-level design of Waremax.

---

## Design Philosophy

Waremax is designed around these principles:

- **Modularity**: Independent, composable crates
- **Determinism**: Same inputs produce same outputs
- **Performance**: Efficient discrete event simulation
- **Extensibility**: Easy to add custom components

---

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      CLI Layer                           │
│  ┌─────┐ ┌─────────┐ ┌───────┐ ┌─────────┐ ┌─────────┐ │
│  │ run │ │validate │ │ sweep │ │benchmark│ │ analyze │ │
│  └─────┘ └─────────┘ └───────┘ └─────────┘ └─────────┘ │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────┐
│                  Configuration Layer                     │
│  ┌────────────────┐  ┌────────────────┐                 │
│  │ waremax-config │  │waremax-testing │                 │
│  │   (parsing)    │  │   (presets)    │                 │
│  └────────────────┘  └────────────────┘                 │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────┐
│                   Simulation Layer                       │
│  ┌─────────────────────────────────────────────────────┐│
│  │                   waremax-sim                        ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ ││
│  │  │waremax-core │  │waremax-     │  │waremax-     │ ││
│  │  │ (DES engine)│  │entities     │  │policies     │ ││
│  │  └─────────────┘  └─────────────┘  └─────────────┘ ││
│  └─────────────────────────────────────────────────────┘│
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────┐
│                    Data Layer                            │
│  ┌────────────┐  ┌────────────┐  ┌────────────────────┐ │
│  │waremax-map │  │waremax-    │  │ waremax-metrics    │ │
│  │ (topology) │  │storage     │  │ (collection)       │ │
│  └────────────┘  └────────────┘  └────────────────────┘ │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────┐
│                   Analysis Layer                         │
│  ┌─────────────────────────────────────────────────────┐│
│  │              waremax-analysis                        ││
│  │  (statistics, comparisons, reporting)               ││
│  └─────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
```

---

## Core Concepts

### Discrete Event Simulation (DES)

The simulation is event-driven:

1. Events are scheduled with timestamps
2. Events are processed in time order
3. Processing an event may schedule new events
4. Simulation advances by jumping between events

```rust
// Conceptual event loop
while let Some(event) = scheduler.next_event() {
    simulation_time = event.time;
    event.process(&mut state);
    // Processing may schedule new events
}
```

### Entity-Component Design

Simulation entities (robots, stations) are modular:

```
Robot Entity
├── Position Component
├── Battery Component
├── Task Component
└── Movement Component
```

### Policy Abstraction

Decisions are made by pluggable policies:

```rust
trait TaskAllocationPolicy {
    fn allocate(&self, task: &Task, robots: &[Robot]) -> Option<RobotId>;
}
```

---

## Key Data Structures

### Event

```rust
struct Event {
    time: SimTime,
    priority: u32,
    payload: EventType,
}

enum EventType {
    OrderArrival(Order),
    TaskAssignment(TaskId, RobotId),
    RobotArrival(RobotId, NodeId),
    ServiceComplete(RobotId, StationId),
    // ...
}
```

### Simulation State

```rust
struct SimulationState {
    time: SimTime,
    map: WarehouseMap,
    robots: Vec<Robot>,
    stations: Vec<Station>,
    tasks: TaskQueue,
    metrics: MetricsCollector,
}
```

### Configuration

```rust
struct Scenario {
    simulation: SimulationConfig,
    map: MapConfig,
    robots: RobotConfig,
    stations: Vec<StationConfig>,
    orders: OrderConfig,
    policies: PolicyConfig,
    // ...
}
```

---

## Execution Flow

### Initialization

```
1. Parse configuration (YAML → Scenario)
2. Build warehouse map (MapConfig → WarehouseMap)
3. Create entities (robots, stations)
4. Initialize policies
5. Schedule initial events (order generator)
```

### Main Loop

```
1. Pop next event from scheduler
2. Advance simulation time
3. Process event (update state, create new events)
4. Collect metrics
5. Check termination condition
6. Repeat
```

### Termination

```
- Simulation duration reached
- All tasks completed (optional)
- User interrupt
```

---

## Threading Model

### Single-Threaded Simulation

Each simulation runs single-threaded for determinism:

```
Simulation 1 ──────────────────►
Simulation 2 ──────────────────►
Simulation 3 ──────────────────►
```

### Parallel Runs

Multiple simulations can run in parallel:

```rust
// Sweep runs multiple configurations in parallel
scenarios.par_iter().map(|s| simulate(s)).collect()
```

---

## Memory Model

### Ownership

- Simulation owns all entities
- Events contain references/IDs, not owned data
- Metrics are collected incrementally

### Efficiency

- Events are lightweight
- Entity lookup via IDs (not pointer chasing)
- Pre-allocated collections where possible

---

## Error Handling

### Configuration Errors

Caught early during parsing:

```rust
enum ConfigError {
    InvalidField(String),
    MissingRequired(String),
    ValidationFailed(String),
}
```

### Simulation Errors

Handled during execution:

```rust
enum SimError {
    Deadlock(Vec<RobotId>),
    InvalidState(String),
    ResourceExhausted(String),
}
```

---

## Related

- [Crate Structure](crates.md): Detailed crate breakdown
- [Data Flow](data-flow.md): How data moves
- [Discrete Event Simulation](../../concepts/simulation/discrete-event.md): DES concepts
