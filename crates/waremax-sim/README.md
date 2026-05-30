# waremax-sim

**Simulation orchestrator for [WareMax](../../README.md): the `World` container, `SimulationRunner`, event handlers, and policy factory.**

Part of the [WareMax](../../README.md) workspace — a deterministic warehouse-robotics simulator and RL benchmark for Robotic Mobile Fulfillment Systems (RMFS).

## Role

Glues the engine ([`waremax-core`](../waremax-core/)), entities, map, storage, and policies into a runnable simulation. `World` owns all per-scenario state; `SimulationRunner` drives the event loop; `EventHandler` translates each `SimEvent` into state transitions, kernel reschedules, metric updates, and (when attribution is enabled) per-task delay recording.

## Key types

| Item | Purpose |
|---|---|
| `World` | Container of all simulation state: `map`, `router`, `traffic`, `robots`, `stations`, `orders`, `tasks`, `policies`, RNG, attribution collector, scenario flags. `policy_context(t)` builds a `PolicyContext` for policies. |
| `PolicySet` | The active `Box<dyn ...Policy>` set inside `World`; swappable per-decision-type. |
| `SimulationRunner` | `new(world, duration, warmup)`, `run() -> SimulationReport`, `world()`, `world_mut()`, `metrics()`, `generate_full_report(...)`. |
| `EventHandler` | Dispatches `SimEvent`s; implements order arrival → dispatch → assignment → movement → service → completion. |
| `create_policies_with_traffic(&PolicyConfig, &TrafficConfig)` | Policy factory used at world build. |
| `DistributionSet`, `create_distributions(&OrderConfig)` | Stochastic input models (Poisson arrivals, negative-binomial lines, Zipf SKUs, lognormal service times). |

## Determinism flags

- `world.congestion_routing: bool` — when true, task routing uses `Router::find_route_with_traffic` (occupancy-weighted) at both pickup and station legs.
- `world.smart_bin_selection: bool` — at task assignment, re-pick the in-stock replica bin minimizing `dist(robot→bin) + dist(bin→station)` for the chosen robot.

## See also

- [`waremax-testing::runner::build_world_from_config`](../waremax-testing/) — the library-level world builder.
- [`waremax-rl`](../waremax-rl/) — overrides `world.policies.task_allocation` with an `RlPolicy`.
