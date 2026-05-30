# waremax-core

**Discrete-event simulation kernel for [WareMax](../../README.md): typed IDs, simulation time, ChaCha-seeded RNG, and the event-priority queue.**

Part of the [WareMax](../../README.md) workspace — a deterministic warehouse-robotics simulator and RL benchmark for Robotic Mobile Fulfillment Systems (RMFS).

## Role

The lowest-level crate every other WareMax crate depends on. It defines the time abstraction, the typed entity identifiers, the random-number generator (deterministic ChaCha8), and the priority-queue–based event kernel that drives the whole simulator forward.

## Key types

| Item | Purpose |
|---|---|
| `Kernel` | Min-heap event queue; `schedule_now/_after`, `pop_next`, `peek_next`, `now`, `events_processed`. |
| `SimEvent` | Enum of all simulator events (order arrival, robot movement, station service, deadlock, metrics tick, …). |
| `SimTime` | Floating-point seconds with explicit constructors and arithmetic; `ZERO`, `from_seconds`, `from_minutes`. |
| `SimRng` | Wrapper over `ChaCha8Rng`, seeded from `u64`; deterministic across platforms. |
| `RobotId` / `TaskId` / `OrderId` / `StationId` / `NodeId` / `EdgeId` / `SkuId` / `RackId` | Newtype IDs over `u32` (`Copy + Hash + Eq`); the inner `.0` enables canonical sorting for determinism. |
| `IdGenerator<T>` | Monotonic ID minter used for orders, tasks, etc. |

## Determinism contract

ChaCha8 is the only RNG used by the simulator. Tie-breaking elsewhere is on `id.0`. The kernel pops events by `(time, event_id)`, breaking ties by insertion order, so trajectories are deterministic for a given seed.

## See also

- [WareMax README — Reproducibility](../../README.md#reproducibility)
- [`waremax-sim`](../waremax-sim/) — the runner that uses this kernel.
