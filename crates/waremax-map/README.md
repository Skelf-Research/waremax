# waremax-map

**Graph-based warehouse map, shortest-path and congestion-aware routing, and traffic management for [WareMax](../../README.md).**

Part of the [WareMax](../../README.md) workspace — a deterministic warehouse-robotics simulator and RL benchmark for Robotic Mobile Fulfillment Systems (RMFS).

## Role

Models the warehouse as a directed graph of nodes (aisles, pick stations, racks, charging bays, …) and edges (corridors with length, capacity, and direction). Provides routing algorithms (Dijkstra, A\*, congestion-aware Dijkstra with occupancy-weighted edges), an edge/node capacity-aware traffic manager, edge reservations, and deadlock detection / wait-for-graph cycle finding.

## Key types

| Item | Purpose |
|---|---|
| `WarehouseMap` | Graph of `Node`s and `Edge`s; `add_node`, `add_edge`, `get_node`, `get_edge`, `euclidean_distance`. |
| `Node` / `Edge` | Geometry (`x`, `y`, `length_m`), type, capacity, speed multiplier. |
| `Router` | `find_route` (shortest), `find_route_with_traffic` (occupancy-weighted), `find_route_avoiding`; `set_congestion_weight`. |
| `Route` | `{ path: Vec<NodeId>, total_distance: f64 }`. |
| `TrafficManager` | Edge/node occupancy, capacity checks, `get_edge_occupancy`, `record_edge_wait`, deadlock cycle detection. |
| `ReservationManager` | Proactive edge reservations over a time horizon. |

## Algorithms

- **Dijkstra** with caching for the shortest-path default.
- **A\*** with Euclidean heuristic for large sparse graphs.
- **Congestion-aware Dijkstra/A\***: edge cost = `length × speed × (1 + w · occupancy)`.
- **Wait-for graph deadlock detection** over robot blocking relationships.

## See also

- [`waremax-sim`](../waremax-sim/) for routing call sites in event handlers.
- [WareMax README — Configuration](../../README.md#configuration) for `traffic.congestion_weight`.
