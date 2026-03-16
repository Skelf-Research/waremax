# Waremax

Waremax is a warehouse robot simulation and optimization workspace. It focuses on modeling how robots, storage, stations, and order flows interact so you can evaluate throughput, congestion, and utilization before changing real layouts or policies.

This repo is intentionally documentation-first. It defines a clear simulation model, configuration schema, and metrics so implementation can be built in a consistent, testable way.

## What Waremax Models

- Warehouse topology (graph-based map with nodes and edges)
- Storage layout (racks, levels, bins)
- Items and inventory placement
- Order arrival and composition
- Robot fleets and station capacities
- Dispatch and batching policies
- Traffic constraints and congestion effects

## Outputs You Can Expect

- Throughput over time
- Order cycle time statistics (average and tail)
- Robot utilization (moving, waiting, charging, servicing)
- Station utilization and queue lengths
- Congestion hotspots
- Event logs and trace data suitable for replay/visualization

## Technology Stack

Waremax is built with performance and reliability in mind:

- **Rust** - Memory-safe, high-performance core
- **rkyv** - Zero-copy serialization for fast state snapshots
- **sled** - Embedded database for scenarios and metrics
- **mimalloc** - Low-latency memory allocation
- **async-nng** - Scalable async messaging for distributed runs

See `docs/system-architecture.md` for details on technology choices.

## Core Concepts

- **Job**: a unit of work (pick, putaway, replenishment)
- **Action**: atomic step within a job (pickup, move, drop, station service)
- **Policy**: a pluggable strategy for allocation or scheduling
- **Scenario**: a full simulation configuration with a map, storage, items, and workload

## Repository Layout

- `README.md` - this overview
- `docs/overview.md` - goals, scope, and non-goals
- `docs/system-architecture.md` - component model and data flow
- `docs/scenario-config.md` - configuration files and schema
- `docs/simulation-model.md` - time model, routing, and traffic rules
- `docs/dispatching-policies.md` - allocation, batching, and prioritization
- `docs/metrics-and-logs.md` - KPIs, logs, and report formats
- `docs/roadmap.md` - phased delivery plan

## Status

Documentation is ready for review and refinement. Implementation can proceed once the core model, configuration schema, and baseline policies are agreed.

## Next Steps (If You Want to Build)

1. Confirm the configuration schema in `docs/scenario-config.md`.
2. Lock the baseline policies in `docs/dispatching-policies.md`.
3. Implement the DES kernel and routing from `docs/simulation-model.md`.
4. Add metrics and log exporters from `docs/metrics-and-logs.md`.

If you want, I can also draft a minimal code scaffold based on these documents.
