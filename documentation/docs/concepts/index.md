# Concepts

Deep dive into the core concepts behind Waremax simulation.

---

## Overview

Understanding these concepts helps you design better simulations and interpret results accurately.

| Section | Topics |
|---------|--------|
| [Simulation Model](simulation/index.md) | DES, time model, events, determinism |
| [Warehouse Model](warehouse/index.md) | Maps, nodes, edges, storage, stations |
| [Robot Operations](robots/index.md) | Movement, tasks, battery, maintenance |
| [Traffic Management](traffic/index.md) | Capacity, congestion, deadlocks |
| [Policies](policies/index.md) | Task allocation, station assignment, batching |
| [Metrics & Analysis](metrics/index.md) | KPIs, time series, RCA |

---

## Simulation Model

Waremax uses **Discrete Event Simulation (DES)**, an efficient approach that advances time by jumping between events rather than stepping through fixed time intervals.

Key concepts:

- **Events** - Things that happen (order arrival, robot movement, service completion)
- **Event Queue** - Priority queue of future events
- **Time Advancement** - Clock jumps to next event time
- **Determinism** - Same seed = same results

[Learn more about the Simulation Model →](simulation/index.md)

---

## Warehouse Model

The warehouse is modeled as a **graph** with nodes and edges:

- **Nodes** - Physical locations (aisles, racks, stations)
- **Edges** - Paths connecting nodes
- **Storage** - Racks, bins, and inventory
- **Stations** - Service points for robots

[Learn more about the Warehouse Model →](warehouse/index.md)

---

## Robot Operations

Robots perform tasks by:

- **Moving** along edges between nodes
- **Executing tasks** assigned by policies
- **Managing battery** (if enabled)
- **Receiving maintenance** (if enabled)

[Learn more about Robot Operations →](robots/index.md)

---

## Traffic Management

Traffic is managed through:

- **Capacity constraints** on nodes and edges
- **Congestion handling** (waiting or rerouting)
- **Deadlock detection** and resolution
- **Reservation systems** for proactive control

[Learn more about Traffic Management →](traffic/index.md)

---

## Policies

Policies control decision-making:

- **Task Allocation** - Which robot gets which task
- **Station Assignment** - Which station services a task
- **Batching** - How items are grouped
- **Priority** - How task types are ordered

[Learn more about Policies →](policies/index.md)

---

## Metrics & Analysis

Waremax provides comprehensive metrics:

- **KPIs** - Throughput, cycle time, utilization
- **Time Series** - Metrics over time
- **Congestion Data** - Hotspot identification
- **Root Cause Analysis** - Bottleneck detection

[Learn more about Metrics & Analysis →](metrics/index.md)
