# Data Flow

How data moves through Waremax.

---

## Overview

Data flows through three main phases:

1. **Configuration** → Parsed and validated
2. **Simulation** → Events processed, state updated
3. **Output** → Metrics collected and reported

---

## Configuration Flow

```
┌──────────────┐     ┌───────────────┐     ┌──────────────┐
│  YAML File   │────►│ waremax-config│────►│   Scenario   │
│  (or preset) │     │  (parsing)    │     │   (struct)   │
└──────────────┘     └───────────────┘     └──────────────┘
                                                  │
                     ┌────────────────────────────┤
                     │                            │
              ┌──────▼──────┐              ┌──────▼──────┐
              │  MapConfig  │              │ RobotConfig │
              └──────┬──────┘              └──────┬──────┘
                     │                            │
              ┌──────▼──────┐              ┌──────▼──────┐
              │WarehouseMap │              │  Vec<Robot> │
              └─────────────┘              └─────────────┘
```

### Parsing Steps

1. **Read YAML**: Load file into string
2. **Deserialize**: YAML → Rust structs
3. **Validate**: Check constraints
4. **Transform**: Config → Simulation objects

---

## Initialization Flow

```
Scenario
    │
    ├──► MapConfig ──────────────► WarehouseMap
    │                                   │
    ├──► RobotConfig ───┬──────────────►│
    │                   │               │
    ├──► StationConfig ─┼──────────────►│
    │                   │               │
    ├──► StorageConfig ─┼──────────────►│
    │                   │               │
    └──► PolicyConfig ──┴──► Policies   │
                             │          │
                             ▼          ▼
                        ┌─────────────────────┐
                        │   SimulationState   │
                        └─────────────────────┘
```

---

## Event Flow

### Event Lifecycle

```
┌────────────┐    schedule    ┌────────────┐    dispatch    ┌────────────┐
│  Created   │──────────────►│  Scheduled │──────────────►│  Processed │
└────────────┘               └────────────┘               └────────────┘
                                   │                            │
                                   │                            │
                             in priority queue           updates state,
                             sorted by time              may create new events
```

### Example: Order Processing

```
OrderArrival(Order)
        │
        ▼
┌───────────────────┐
│ Create Task(s)    │
│ from Order items  │
└───────────────────┘
        │
        ▼
TaskCreated(Task)
        │
        ▼
┌───────────────────┐
│ TaskAllocation    │
│ Policy selects    │
│ robot             │
└───────────────────┘
        │
        ▼
TaskAssigned(Task, Robot)
        │
        ▼
┌───────────────────┐
│ Robot starts      │
│ moving            │
└───────────────────┘
        │
        ▼
RobotStartMove(Robot, Path)
        │
        ▼
    ... (movement events)
        │
        ▼
RobotArrival(Robot, Station)
        │
        ▼
┌───────────────────┐
│ Station serves    │
│ robot             │
└───────────────────┘
        │
        ▼
ServiceComplete(Robot, Station)
        │
        ▼
TaskComplete(Task)
```

---

## State Updates

### Robot State Transitions

```
IDLE ──► assign ──► TRAVELING ──► arrive ──► WORKING ──► complete ──► IDLE
  ▲                     │                        │
  │                     │ blocked                │ low battery
  │                     ▼                        ▼
  │                 WAITING                  CHARGING
  │                     │                        │
  └─────────────────────┴────────────────────────┘
```

### State Update Flow

```
Event arrives
      │
      ▼
┌─────────────────┐
│ Read current    │
│ state           │
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ Apply changes   │
│ - Update robot  │
│ - Update station│
│ - Update map    │
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ Schedule new    │
│ events          │
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ Record metrics  │
└─────────────────┘
```

---

## Metrics Flow

### Collection Points

```
Event Processing
       │
       ├──► Task created ──► task_count++
       │
       ├──► Task completed ──► completion_time recorded
       │
       ├──► Robot moves ──► travel_time++
       │
       ├──► Robot waits ──► wait_time++
       │
       └──► Station serves ──► service_time++
```

### Aggregation

```
Raw Events
    │
    ▼
┌─────────────────┐
│ MetricsCollector│
│ - counts        │
│ - sums          │
│ - distributions │
└─────────────────┘
    │
    ├──► Timeseries (periodic snapshots)
    │
    └──► Summary (end of simulation)
```

---

## Output Flow

```
SimulationState
       │
       ▼
┌─────────────────┐
│ MetricsCollector│
└─────────────────┘
       │
       ├──────────────────┬──────────────────┐
       │                  │                  │
       ▼                  ▼                  ▼
┌─────────────┐   ┌─────────────┐   ┌─────────────┐
│ summary.json│   │timeseries.csv│   │events.jsonl │
└─────────────┘   └─────────────┘   └─────────────┘
```

### Output Contents

**summary.json:**
```json
{
  "tasks": { "completed": 1250, "avg_time_s": 42.3 },
  "robots": { "utilization": 0.78 }
}
```

**timeseries.csv:**
```csv
timestamp,throughput,utilization
0,0,0.00
60,82,0.65
120,95,0.78
```

**events.jsonl:**
```json
{"time":0.0,"type":"OrderArrival","order_id":"O1"}
{"time":0.5,"type":"TaskAssigned","task_id":"T1","robot_id":"R1"}
```

---

## Parallel Execution Flow

### Sweep/Compare Operations

```
┌─────────────┐
│ Sweep Config│
└─────────────┘
       │
       ▼
┌─────────────┐
│ Generate    │
│ Variations  │
└─────────────┘
       │
       ▼
┌─────────────────────────────────────────┐
│            Parallel Execution            │
│  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐   │
│  │Sim 1│  │Sim 2│  │Sim 3│  │Sim 4│   │
│  └─────┘  └─────┘  └─────┘  └─────┘   │
└─────────────────────────────────────────┘
       │
       ▼
┌─────────────┐
│ Aggregate   │
│ Results     │
└─────────────┘
       │
       ▼
┌─────────────┐
│ Comparison  │
│ Report      │
└─────────────┘
```

---

## Data Structures in Flight

### During Simulation

```rust
struct ActiveSimulation {
    // Core state
    time: SimTime,
    scheduler: Scheduler,

    // Entities (owned)
    robots: Vec<Robot>,
    stations: Vec<Station>,
    tasks: TaskQueue,

    // References (borrowed)
    map: &WarehouseMap,
    policies: &PolicySet,

    // Output (accumulated)
    metrics: MetricsCollector,
}
```

### Memory Lifecycle

```
Initialization:
  - Config loaded (temporary)
  - Map built (persistent)
  - Entities created (persistent)

Simulation:
  - Events created/destroyed (transient)
  - State updated (persistent)
  - Metrics accumulated (growing)

Completion:
  - Entities dropped
  - Metrics finalized
  - Results written
```

---

## Related

- [Architecture Overview](overview.md)
- [Events](../../concepts/simulation/events.md)
- [Metrics](../../concepts/metrics/index.md)
