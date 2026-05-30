# waremax-policies

**Pluggable dispatching policies for [WareMax](../../README.md): task allocation, station assignment, batching, priority, traffic, and deadlock resolution.**

Part of the [WareMax](../../README.md) workspace — a deterministic warehouse-robotics simulator and RL benchmark for Robotic Mobile Fulfillment Systems (RMFS).

## Role

Every operational decision in WareMax is a trait-based policy. Each trait is a clean `Box<dyn ...Policy>` seam: implement, register an arm in `crates/waremax-sim/src/policy_factory.rs`, and you're done. The RL `RlPolicy` (in [`waremax-rl`](../waremax-rl/)) is one such allocation policy — the simulator's event handler does not know it's RL.

## Traits

| Trait | Decision |
|---|---|
| `TaskAllocationPolicy` | Which robot handles a given task |
| `StationAssignmentPolicy` | Which station receives a task |
| `BatchingPolicy` | How to group pending tasks |
| `PriorityPolicy` | Ordering across task types / due dates |
| `TrafficPolicy` / `EdgeTrafficPolicy` | Edge/node entry control, wait/reroute, occupancy callbacks |
| `DeadlockResolver` | What to do on a wait-for cycle (back-up / abort / wait-and-retry) |

The shared `PolicyContext<'a>` exposes `current_time`, `map`, `robots`, `tasks`, `stations`, `orders`, and optionally `attribution` (when the analysis layer is enabled — used by the attribution-shaped RL reward).

## Built-in policies

**Allocation:** `nearest_robot`, `least_busy`, `round_robin`, `auction`, `workload_balanced`, `rl_agent` (placeholder; overwritten by `RlEnv`).
**Station assignment:** `least_queue`, `nearest_station`, `fastest_service`, `due_time_priority`.
**Batching:** `none`, `zone`.
**Priority:** `strict_priority`, `fifo`, `due_time`.
**Traffic:** `wait_at_node`, `reroute_on_wait`, `adaptive`, `coarse`/`continuous` edge tracking.
**Deadlock:** `youngest_backs_up`, `lowest_priority_aborts`, `wait_and_retry`, `tiered`.

All policies use canonical `id.0` tie-breaking for determinism.

## See also

- [`waremax-rl`](../waremax-rl/) — RL implementation of `TaskAllocationPolicy`.
- [`waremax-sim::policy_factory`](../waremax-sim/) — the registration seam.
- [Dispatching policies (docs)](../../docs/dispatching-policies.md).
