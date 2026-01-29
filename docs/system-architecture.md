# System Architecture

This document describes the architecture of Waremax, a warehouse robot simulation system built in Rust.

## Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Language | **Rust** | Memory safety, performance, fearless concurrency |
| Serialization | **rkyv** | Zero-copy deserialization for fast state snapshots and event logs |
| Storage | **sled** | Embedded database for scenario state, metrics, and replay data |
| Allocator | **mimalloc** | High-performance memory allocator for reduced latency |
| Messaging | **async-nng** | Async nanomsg-next-gen for inter-process communication and distributed runs |

### Why These Choices

- **Rust**: Ensures memory safety without garbage collection pauses, critical for deterministic simulation timing.
- **rkyv**: Enables zero-copy access to serialized data, minimizing overhead when checkpointing simulation state or replaying event logs.
- **sled**: Provides ACID-compliant embedded storage without external dependencies, ideal for storing scenario configs, simulation snapshots, and metrics.
- **mimalloc**: Reduces allocation latency and fragmentation in allocation-heavy workloads like discrete-event simulation.
- **async-nng**: Supports scalable messaging patterns (pub/sub, req/rep) for distributing simulation across cores or machines.

## Layers

1. **Scenario Input Layer**
   - Parses configuration files (map, storage, SKUs, orders, policies).
   - Validates schema, defaults, and constraints.

2. **Simulation Kernel (DES)**
   - Executes a priority queue of events.
   - Maintains global time, entity states, and event scheduling.

3. **Execution Models**
   - **Traffic Model**: edge and node capacity, waiting rules.
   - **Routing Model**: shortest-path or congestion-aware routing.
   - **Station Model**: queues and service-time distributions.
   - **Inventory Model**: per-bin quantities and reservations.

4. **Policy Layer**
   - Task allocation (which robot does which task)
   - Station assignment (which station receives a task)
   - Batching (how tasks are grouped)
   - Priority arbitration (pick vs putaway vs replen)

5. **Output Layer**
   - Metrics aggregation and KPI reporting.
   - Structured logs for events and traces.

## Core Components

- **Map**
  - Graph: nodes with types, edges with lengths and direction.
  - Optional zones for policy constraints or batching.

- **Robots**
  - State: location, load, task queue, battery (optional).
  - Capabilities: speed, payload, charging behavior (optional).

- **Stations**
  - Types: pick, drop, inbound, outbound.
  - Capacity: concurrency, queue size, service distribution.

- **Storage**
  - Racks: access nodes, levels, bin counts.
  - Bins: addressable locations with quantities.

- **Orders and Jobs**
  - Orders: arrival process, line item distributions, due times.
  - Jobs: pick, putaway, replenishment.
  - Actions: pickup, move, drop, station service.

## Data Flow

1. Scenario files are parsed and validated.
2. Initial state is constructed (inventory, robots, stations).
3. Order arrivals and work generation are scheduled.
4. Policies assign tasks and create routes.
5. Traffic model governs movement and waiting.
6. Metrics and logs are emitted continuously.

## Interfaces (Conceptual)

- **Task Allocation**
  - Input: robot states, task queue, map distances
  - Output: robot-task assignment

- **Batching**
  - Input: tasks, station constraints, capacity limits
  - Output: bundles of tasks

- **Routing**
  - Input: map, source, destination, congestion
  - Output: path and travel time estimate

- **Traffic Manager**
  - Input: edge or node request at time t
  - Output: grant or wait time
