# waremax-rl

**The reinforcement-learning control seam for the [WareMax](../../README.md) warehouse-robotics simulator. Exposes the task-allocation decision as a deterministic, Gym-style environment with attribution-shaped and per-decision routed reward modes.**

Part of the [WareMax](../../README.md) workspace — a deterministic warehouse-robotics simulator and RL benchmark for Robotic Mobile Fulfillment Systems (RMFS).

## Role

`waremax-rl` inverts control at the task-allocation seam without restructuring the event handler: an `RlPolicy` (implementing `waremax_policies::TaskAllocationPolicy`) blocks at each decision waiting for an action from an external agent over a strict crossbeam ping-pong channel. The simulation runs on a worker thread. Because exactly one side runs at a time, the agent-driven run is **byte-identical** for a given `(seed, action sequence)`.

This is the Rust-side control loop; the Python Gymnasium wrapper is in [`waremax-gym`](../waremax-gym/).

## Key types

| Item | Purpose |
|---|---|
| `RlEnv` | Owns the worker thread + channels; `reset(seed) -> Observation`, `step(action) -> StepResult`. |
| `RlPolicy` | Implements `TaskAllocationPolicy`; blocks at each `allocate()` for an action from the agent. |
| `Observation` | Fixed-shape, fully-owned per-decision state: candidate robot features, task features, action mask. |
| `RewardConfig` / `RewardMode` | `Sparse`, `Dense`, `Attribution`, `AttributionFull`, `Routed`. |
| `RewardSnapshot` / `delta` / `snapshot_from` | Reward computation from world aggregates + (optional) per-task delay attribution. |

## Reward modes

- **`Sparse`** — throughput minus an SLA-miss penalty only.
- **`Dense`** — hand-designed shaping (throughput, lateness, backlog).
- **`Attribution`** — penalize the simulator-attributed delay of completed tasks, restricted to *controllable* components (assignment wait, travel-to-pickup).
- **`AttributionFull`** — ablation: as above but additionally penalize *uncontrollable* delay (congestion, station queue). Used to evidence the controllability principle.
- **`Routed`** — per-decision routed credit: each assignment's controllable cost (estimated travel + chosen robot's backlog) is charged to the exact action that incurred it.

## Determinism

`tests/determinism.rs` asserts that `(seed, action sequence)` reproduces exactly across two freshly spawned worker threads, for multiple scenario presets, and for both the standard and attribution-mode reward paths. The same property holds end-to-end through the Python boundary (see [`waremax-gym`](../waremax-gym/)).

## Example (Rust)

```rust
use waremax_rl::{RlEnv, RewardConfig, RewardMode, ActionMsg};
use waremax_testing::ScenarioPreset;

let mut scenario = ScenarioPreset::Quick.config();
scenario.simulation.duration_minutes = 10.0;
let cfg = RewardConfig { mode: RewardMode::Routed, ..Default::default() };
let mut env = RlEnv::new(scenario, cfg);
let _first_obs = env.reset(42);
loop {
    let res = env.step(ActionMsg::Choose(0));   // your agent here
    if res.done { break; }
}
let report = env.last_report().unwrap();
```

## Used by

- [`waremax-gym`](../waremax-gym/) — PyO3 bindings + Python wrapper.

## See also

- [WareMax README](../../README.md) — architecture, research findings, citation.
- [Concepts](../../README.md#concepts) — RMFS, SMDP, attribution, controllability.
