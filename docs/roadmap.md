# Roadmap

This roadmap outlines a phased delivery plan for Waremax, from minimal viable simulator to advanced distributed capabilities.

## Technology Foundation

All versions build on the core technology stack:

| Component | Technology | Introduced |
|-----------|------------|------------|
| Language | Rust | v0 |
| Serialization | rkyv | v0 |
| Storage | sled | v0 |
| Allocator | mimalloc | v0 |
| Messaging | async-nng | v3 |

---

## v0 (Minimum Viable Simulator)

**Goal**: End-to-end simulation of pick operations with basic policies.

### Core Infrastructure
- [ ] Project scaffolding with Rust, rkyv, sled, mimalloc
- [ ] Scenario file parser (YAML/JSON) with schema validation
- [ ] DES kernel with priority event queue
- [ ] Event logging to sled with rkyv serialization

### Map & Storage
- [ ] Graph-based map (nodes, edges, lengths)
- [ ] Edge capacity enforcement
- [ ] Rack and bin inventory model

### Stations (v0 scope: pick only)
- [ ] Pick station with queue and service time (base + per_item model)
- [ ] Concurrency limits per station

### Robots
- [ ] Robot fleet initialization
- [ ] Basic motion: edge traversal at constant speed
- [ ] Task queue per robot

### Policies (v0 baseline)
- [ ] Task allocation: `nearest_robot`
- [ ] Station assignment: `least_queue`
- [ ] Batching: `none` (single-task)
- [ ] Priority: `strict_priority` (pick only, so trivial)

### Traffic (v0 baseline)
- [ ] Policy: `wait_at_node`
- [ ] Edge and node capacity checks

### Routing (v0 baseline)
- [ ] Algorithm: `dijkstra`
- [ ] Route caching between frequent node pairs

### Metrics & Output
- [ ] KPIs: throughput, order cycle time (avg, p95), robot utilization
- [ ] Event log export (JSON via rkyv)

---

## v1 (Multi-Flow & Policy Options)

**Goal**: Full warehouse flows with configurable policies.

### Additional Station Types
- [ ] Drop stations
- [ ] Inbound stations
- [ ] Outbound stations

### Putaway & Replenishment
- [ ] Putaway flow with arrival process
- [ ] Replenishment triggers on low inventory
- [ ] Destination policy: `nearest_empty_bin`

### Additional Policies
- [ ] Task allocation: `auction`, `workload_balanced`
- [ ] Station assignment: `fastest_service`, `due_time_priority`
- [ ] Batching: `station_batch` with max_items and max_weight
- [ ] Priority: `weighted_fair`

### Traffic Enhancements
- [ ] Policy: `reroute_on_wait` with configurable threshold

### Routing Enhancements
- [ ] Algorithm: `astar` option
- [ ] Congestion-aware routing with configurable weight

### Service Time
- [ ] Distribution-based service times (lognormal, exponential, etc.)

### Battery & Charging
- [ ] Battery state tracking (SOC, capacity)
- [ ] Charging stations and charge rate
- [ ] Low-battery task deferral

### Metrics Expansion
- [ ] Station utilization and queue length over time
- [ ] Congestion hotspot ranking
- [ ] SLA miss rate

---

## v2 (Advanced Simulation)

**Goal**: Sophisticated traffic control, storage models, and experimentation.

### Traffic Control
- [ ] Policy: `reservation` with configurable horizon
- [ ] Deadlock detection and resolution

### Advanced Policies
- [ ] Batching: `zone_batch` with zone affinity
- [ ] Priority: `sla_driven` with late-task boosting

### Multi-Level Storage
- [ ] Level-specific access times
- [ ] Vertical movement constraints
- [ ] Level-aware bin selection

### Layout Experiments
- [ ] One-way aisles
- [ ] Fast lanes / express paths
- [ ] Dynamic blocked edges/nodes

### Scenario Management
- [ ] Scenario sweep tooling (parameter grid search)
- [ ] Comparative run reports
- [ ] Seed management for reproducibility

---

## v3 (Distributed & Visualization)

**Goal**: Scale out simulation and enable visual analysis.

### Distributed Simulation
- [ ] async-nng integration for inter-process messaging
- [ ] Partitioned simulation across cores
- [ ] Multi-machine coordination (pub/sub for events)

### Visualization & Replay
- [ ] Trace data export (robot positions, queue lengths over time)
- [ ] Replay engine for event log playback
- [ ] Real-time dashboard integration hooks

### Analysis Tooling
- [ ] Automated bottleneck detection
- [ ] What-if scenario comparisons
- [ ] Report generation (PDF/HTML)

---

## Version Summary

| Version | Focus | Key Deliverables |
|---------|-------|------------------|
| v0 | MVP | Pick flow, basic policies, DES kernel, metrics |
| v1 | Full flows | All station types, putaway/replen, policy options, battery |
| v2 | Advanced | Reservation traffic, zone batching, multi-level storage, experiments |
| v3 | Scale | Distributed runs, visualization, replay, analysis |

## Dependencies

```
v0 ─────► v1 ─────► v2 ─────► v3
                      │
                      └─► (v2 features can be selectively implemented)
```

v0 is required for all subsequent versions. v1 and v2 features can be prioritized based on user needs.
