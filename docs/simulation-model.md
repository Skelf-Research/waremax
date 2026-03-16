# Simulation Model

Waremax uses a discrete-event simulation (DES) kernel. The goal is to scale to many robots without stepping time at millisecond resolution.

## Time Model

- Global clock advances to the next scheduled event.
- Each event updates state and schedules downstream events.
- A small motion tick is optional for models that require periodic updates, but the default is fully event-driven.

## Event Types (Baseline)

- Order arrival
- Putaway or replenishment arrival
- Task assignment
- Robot departs node
- Robot arrives node
- Station service start
- Station service end
- Inventory decrement/increment
- Charging start/end (if enabled)

## Robot Motion

Robots traverse edges in the map graph.

- Travel time = `edge_length / robot_speed`.
- If an edge is at capacity, the robot waits at the current node.
- Waiting may trigger rerouting based on a policy threshold.

## Traffic Model

Each edge and node has a capacity. The Traffic Manager decides whether a robot may enter an edge at a given time.

Supported traffic policies (configured via `traffic.policy` in scenario config):

- `wait_at_node`: Robot waits at current node until edge is available. Simplest policy with no rerouting.
- `reroute_on_wait`: If waiting exceeds `reroute_after_s` threshold, recompute path to avoid congestion.
- `reservation`: Reserve edges over a short time horizon (`reservation_horizon_s`). Prevents conflicts proactively.

## Routing

Routing returns a path and estimated travel time. Configuration is via `routing` in scenario config.

**Algorithm options** (`routing.algorithm`):
- `dijkstra`: Standard shortest-path. Lower overhead, suitable for most cases.
- `astar`: A* with Euclidean heuristic. Faster for large sparse graphs.

**Congestion awareness** (`routing.congestion_aware`):
- When enabled, edge costs include a congestion penalty based on current occupancy.
- `congestion_weight` controls penalty strength (0.0 = ignore, 1.0 = heavy penalty).

**Route caching** (`routing.cache_routes`):
- When enabled, caches routes between frequently used node pairs.
- Cache is invalidated when map constraints change.

## Stations

Stations are modeled as queues with service times.

- Queue capacity can be finite or unbounded.
- Service time can be modeled as `base + per_item` or as a distribution.
- Concurrency defines the number of parallel service slots.

## Inventory

Inventory is represented as counts in bins.

- Picks decrement quantities at the source bin.
- Putaway increments quantities at the destination bin.
- If a pick is not available, the task can backorder or trigger replenishment based on policy.

## Determinism

- Random processes must be seeded from the scenario config.
- Identical inputs and seeds should produce identical outputs.
