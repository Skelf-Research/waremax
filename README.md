<div align="center">

# WareMax

**An open, deterministic warehouse-robotics simulator and reinforcement-learning benchmark for task allocation in Robotic Mobile Fulfillment Systems (RMFS).**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg?style=flat-square)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.10%2B-3776ab.svg?style=flat-square)](https://www.python.org/)
[![Gymnasium](https://img.shields.io/badge/gymnasium-env-9cf.svg?style=flat-square)](https://gymnasium.farama.org/)
[![Status](https://img.shields.io/badge/status-research-yellow.svg?style=flat-square)](#research)

[Getting Started](#getting-started) ·
[RL Quickstart](#reinforcement-learning-quickstart) ·
[Research](#research) ·
[Concepts](#concepts) ·
[FAQ](#faq) ·
[Cite](#citation)

</div>

---

## What is WareMax?

**WareMax** is a high-performance discrete-event simulator (DES) for **Robotic Mobile Fulfillment Systems** (RMFS) — fleets of autonomous mobile robots (AMRs) that move inventory to pick stations, in the style of Kiva / Amazon Robotics. It ships with:

- A **deterministic** simulator core in Rust: identical seed and action sequence produce **byte-identical** trajectories.
- A **Gymnasium**-style reinforcement-learning environment exposing the task-allocation decision as a semi-Markov decision process (SMDP), with PyO3 bindings.
- **Instrumented causal delay attribution** — per-task decomposition of cycle time into assignment wait, travel, station queue, congestion, and service.
- Four built-in **reward modes** for dispatching (sparse, dense, attribution, per-decision routed) and a **permutation-equivariant candidate-scoring policy** for use with [MaskablePPO](https://sb3-contrib.readthedocs.io/en/master/modules/ppo_mask.html).
- A **resumable experiment grid** that produces multi-seed mean ± CI tables across scenarios.
- Heuristic baselines (nearest-robot, least-busy, round-robin, auction, workload-balanced) for apples-to-apples comparison.

It is built for two audiences: **operations engineers** sizing fleets and comparing policies before deployment, and **researchers** running reproducible RL experiments on warehouse dispatching.

## Highlights

- 🔒 **Reproducible.** A property the project actively tests — including a fix for several latent HashMap-iteration determinism bugs in the simulator core (so prior seeded results on the unfixed simulator were silently irreproducible).
- 🧪 **Research-grade benchmark.** Determinism + a Gym interface + delay attribution + a multi-scenario grid + persisted CSV results make experiments cleanly comparable.
- 🦀 **Fast.** Pure-Rust event-driven core with mimalloc; trains thousands of episodes on a laptop CPU.
- 🐍 **Pythonic.** PyO3 + maturin → `pip install`-style workflow once built; usable from `stable-baselines3` and `sb3-contrib`.
- 🧩 **Configurable structure.** Tunable load, fleet size, traffic capacity, congestion-aware routing, smart pickup-bin selection, and inventory SKU count make it easy to study *when* dispatching intelligence has leverage.
- 📈 **Statistical tooling.** Parameter sweeps, A/B tests with Welch's t-test, benchmarking with regression detection — built in.

## Why WareMax?

| You want to… | WareMax gives you |
|---|---|
| Pick a fleet size before procuring robots | Deterministic sweeps over `robots.count` with CIs |
| Compare dispatching policies under stress | Heuristic baselines + RL agents on identical seeds |
| Train an RL dispatcher | Gymnasium env, MaskablePPO, candidate-scoring policy |
| Decompose where time is lost in a warehouse | Per-task delay attribution + root-cause reporter |
| Study *when* learning helps vs. heuristics | Tunable scenario structure (load, congestion, replicas) |
| Run reproducible RMFS research | Determinism guarantees + a resumable experiment grid |

## Research

WareMax was built to support a research program characterizing when learned policies beat heuristics for warehouse dispatching. The accompanying paper is *“When Does Learning to Dispatch Help? A Deterministic Benchmark and a Controllability Principle for Reward Design in Warehouse Robotics.”* Key findings, each backed by multi-seed committed data under [`crates/waremax-gym/python/results/`](crates/waremax-gym/python/results):

1. **Representation × reward interaction.** A permutation-equivariant *candidate-scoring* policy paired with a reward targeting the delay the decision controls reaches strong-heuristic SLA attainment (~97% on-time); a flattened MLP — or a naive dense/sparse reward — plateaus near the weakest heuristic (~82–85%). Neither ingredient suffices alone.
2. **Controllability principle for reward design.** Restricting attribution to delay the agent can *control* (assignment wait, travel-to-pickup) is directionally better than additionally penalizing the *uncontrollable* delay (congestion, station queue). Reported with significance test (Welch's t).
3. **Bounded leverage.** Across four control levers (allocation, congestion-aware routing, reward design, pickup-bin choice) and a range of regimes, learned dispatching matches but does not beat simple heuristics here, because the system is capacity- and destination-contention-bound; state-blind round-robin is near-optimal.

A learning-curve figure (candidate vs. flattened MLP) is at [`crates/waremax-gym/python/results/learning_curves.pdf`](crates/waremax-gym/python/results/learning_curves.pdf); a CSV grid at [`runs.csv`](crates/waremax-gym/python/results/runs.csv).

## Getting Started

### Install (Rust CLI)

```bash
git clone https://github.com/Skelf-Research/waremax.git
cd waremax
cargo install --path .
waremax --version
```

### Run a deterministic simulation

```bash
# Built-in preset
waremax run --preset standard -o results/

# Your own scenario
waremax run my_scenario.yaml -o results/

# Compare policies on identical seeds
waremax compare scenario.yaml \
    --param policies.task_allocation=nearest_robot \
    --param policies.task_allocation=least_busy \
    --param policies.task_allocation=round_robin
```

### Build the Python RL extension

```bash
python -m venv .venv && . .venv/bin/activate
pip install -r crates/waremax-gym/python/requirements.txt
pip install torch --index-url https://download.pytorch.org/whl/cpu   # for training
maturin develop -m crates/waremax-gym/Cargo.toml --release
```

## Reinforcement Learning Quickstart

### Heuristic baselines on a held-out seed set

```bash
PYTHONPATH=crates/waremax-gym/python python crates/waremax-gym/python/baseline.py \
    --preset standard --duration 15 --due-time 2 --seeds 8
```

### Train MaskablePPO and compare reward modes

```bash
PYTHONPATH=crates/waremax-gym/python python crates/waremax-gym/python/train_ppo.py \
    --preset standard --duration 15 --due-time 2 \
    --timesteps 40000 --eval-seeds 8 \
    --reward-modes sparse,dense,attribution,routed \
    --policy candidate
```

### Run the full reproducible experiment grid

```bash
# resumable; appends to results/runs.csv and rebuilds results/tables.md
PYTHONPATH=crates/waremax-gym/python python crates/waremax-gym/python/experiments.py \
    --train-seeds 3 --eval-seeds 8 --timesteps 40000
```

### Use the env directly from Python

```python
from waremax_alloc_env import WaremaxAllocEnv
env = WaremaxAllocEnv(preset="standard", duration_minutes=15,
                      due_time_minutes=2, reward_mode="routed")
obs, _ = env.reset(seed=42)
# obs is a Dict({"robots": (64, 8), "task": (6,), "action_mask": (64,)})
# action: index into masked candidates; use sb3-contrib MaskablePPO.
```

## Architecture

WareMax is a Cargo workspace of focused crates. Determinism is enforced uniformly across them: every randomized data structure is canonically iterated, every RNG is seeded.

```
waremax/
├── src/                        # CLI binary (`waremax` command)
└── crates/
    ├── waremax-core            # DES kernel, event queue, IDs, SimTime, ChaCha-seeded RNG
    ├── waremax-map             # Graph topology, shortest-path & congestion-aware routing, traffic
    ├── waremax-storage         # Racks, bins, SKUs, inventory replicas
    ├── waremax-entities        # Robot, Order, Task, Station, ChargingStation
    ├── waremax-policies        # Allocation, station assignment, batching, priority, traffic policies
    ├── waremax-config          # YAML/JSON scenario parsing + schema validation
    ├── waremax-metrics         # Event log, time-series, CSV/JSON export, HTML/PDF reports
    ├── waremax-sim             # SimulationRunner, World, EventHandler, policy factory
    ├── waremax-testing         # Presets, ScenarioBuilder, BatchRunner, A/B testing, benchmarking
    ├── waremax-analysis        # Delay attribution, critical-path analysis, bottlenecks, RCA
    ├── waremax-statemachine    # Generic state-machine primitives
    ├── waremax-api             # Axum-based REST/WebSocket API
    ├── waremax-api-server      # API server binary
    ├── waremax-rl              # 🆕 RL control seam (Gym-style env, attribution/routed rewards)
    └── waremax-gym             # 🆕 PyO3 bindings + Python wrapper + training scripts
```

Each crate has its own `README.md` with API entry points; see [Concepts](#concepts) for the cross-cutting ideas.

## Concepts

| Term | Definition |
|---|---|
| **RMFS** | *Robotic Mobile Fulfillment System.* Warehouse operation where robots transport pods/items to pick stations rather than humans walking aisles. |
| **AMR** | *Autonomous Mobile Robot.* The mobile units the simulator dispatches. |
| **DES** | *Discrete-Event Simulation.* Event-driven time advancement; WareMax's core. |
| **SMDP** | *Semi-Markov Decision Process.* Time between agent decisions varies; the natural formulation for event-driven control. |
| **Task allocation / dispatching** | The decision *which robot handles which pick task*. The primary lever WareMax studies. |
| **Delay attribution** | Per-task decomposition of cycle time into causal categories (assignment, travel, station queue, congestion, service). |
| **Controllability principle** | A reward-design guideline: shape with delay the decision actually controls; including uncontrollable delay degrades learning. |
| **Candidate-scoring policy** | A permutation-equivariant actor that scores each candidate robot with a shared network and selects via masked softmax — the right inductive bias for variable action sets. |
| **MaskablePPO** | Action-masked PPO from sb3-contrib, used so the agent never proposes an invalid candidate. |
| **Determinism** | Same seed + same action sequence ⇒ identical trajectory. Verified by tests. |

## Reproducibility

WareMax is engineered for exact reproducibility. The core simulator is **single-threaded** (per scenario), uses a **ChaCha8** RNG seeded from `u64`, and applies canonical (id-based) tie-breaking throughout — in inventory placement, station/charging selection, and all heuristic policies. The RL control loop wraps the simulator with a **strict crossbeam ping-pong handshake** between a worker thread (the simulation) and the agent, so exactly one side runs at a time and `(seed, action sequence) ⇒ trajectory`. Reproducibility is enforced by the `waremax-rl` determinism tests.

## Configuration

Scenarios are YAML, parsed by `waremax-config`. A minimal example:

```yaml
seed: 12345
simulation:
  duration_minutes: 60
  warmup_minutes: 5

robots:
  count: 10
  max_speed_mps: 1.5

stations:
  - id: S1
    node: "30"
    type: pick
    concurrency: 2
    service_time_s:
      distribution: lognormal
      base: 12.0
      per_item: 3.0

orders:
  arrival_process:
    type: poisson
    rate_per_min: 1.0
  lines_per_order:
    type: negative_binomial
    mean: 2.0
  sku_popularity:
    type: zipf
    alpha: 1.1
  due_times:
    type: fixed
    minutes: 30

policies:
  task_allocation: { type: routed }       # or nearest_robot, least_busy, round_robin, auction, rl_agent
  station_assignment: { type: least_queue }
  batching: { type: none }
  priority: { type: strict_priority }
  smart_bins: false                        # re-pick the in-stock replica nearest to the chosen robot
  inventory_skus: 100                      # smaller => more spread replicas per SKU

traffic:
  policy: wait_at_node
  node_capacity_default: 4
  edge_capacity_default: 4
  congestion_weight: 0.0                   # > 0 enables occupancy-weighted routing
```

## CLI reference

| Command | Description |
|---|---|
| `waremax run <scenario.yaml>` | Execute a simulation |
| `waremax validate <scenario.yaml>` | Validate configuration |
| `waremax demo` | Run a demo scenario |
| `waremax sweep` | Parameter exploration |
| `waremax compare` | Compare configurations |
| `waremax ab-test` | Statistical comparison (Welch's t) |
| `waremax benchmark` | Performance regression detection |
| `waremax analyze` | Analyze report directories |
| `waremax list-presets` | Show built-in scenario presets |

## FAQ

**How does WareMax compare to RAWSim-O / other RMFS simulators?**
WareMax targets three properties prior simulators do not jointly provide: *exact* determinism (byte-identical replay), a first-class RL interface (Gymnasium + PyO3), and instrumented delay attribution usable as a reward signal. The Rust core also lets a single laptop run experiment grids that would take much longer in interpreted simulators.

**Is the simulation actually reproducible?**
Yes. We expose this as a property, test it, and fixed several core bugs (notably `HashMap`-iteration-dependent inventory placement and heuristic tie-breaking) that had previously made seeded results silently irreproducible. See [`waremax-rl/tests/determinism.rs`](crates/waremax-rl/tests/determinism.rs).

**Can I plug in my own dispatching policy?**
Yes. Implement `TaskAllocationPolicy` (Rust) — a one-method trait — and add a single arm to `crates/waremax-sim/src/policy_factory.rs`. The RL `RlPolicy` is one such implementation. The same pattern applies to station assignment, batching, priority, and traffic policies.

**Can I train RL without writing Rust?**
Yes. Build the extension once with `maturin develop`, then everything is Python: `WaremaxAllocEnv` is a standard `gymnasium.Env`; pair it with `sb3_contrib.MaskablePPO` and the `CandidateScoringPolicy`. See [`crates/waremax-gym/python/train_ppo.py`](crates/waremax-gym/python/train_ppo.py).

**Do the RL agents beat the heuristics?**
On the built-in scenarios, *no* — they match round-robin / nearest-robot but do not surpass them, because the system is capacity- and destination-contention-bound. This is itself a [finding](#research) of the paper. WareMax's tunable structure lets you study regimes where dispatching has genuine leverage.

**What reward should I use?**
Start with `routed` (per-decision controllable cost). The `attribution` mode is also strong, and exposes the simulator's causal delay decomposition. `dense` and `sparse` are baselines, not recommendations.

**Is the project actively maintained?**
Yes — it backs an ongoing research effort. See [`docs/`](docs/) for design notes and [open issues](https://github.com/Skelf-Research/waremax/issues) for the roadmap.

## Documentation

- [System architecture](docs/system-architecture.md)
- [Simulation model](docs/simulation-model.md)
- [Dispatching policies](docs/dispatching-policies.md)
- [Roadmap](docs/roadmap.md)
- [Per-crate API docs](https://docs.rs/waremax) (publish-pending)

## Citation

If you use WareMax in your research, please cite:

```bibtex
@unpublished{waremax2026,
  title  = {When Does Learning to Dispatch Help? A Deterministic Benchmark
            and a Controllability Principle for Reward Design in Warehouse Robotics},
  author = {Sarkar, Dipankar},
  note   = {WareMax benchmark; \url{https://github.com/Skelf-Research/waremax}},
  year   = {2026}
}
```

## Contributing

Contributions are welcome — bug reports, scenario contributions, new heuristic baselines, new reward modes, alternative RL algorithms, or paper-ready ablations.

```bash
git clone https://github.com/Skelf-Research/waremax.git
cd waremax
cargo test --workspace        # Rust unit + determinism tests
```

Please open an issue before substantial changes so we can align on direction.

## License

MIT. See [LICENSE](LICENSE).

---

<sub>Keywords: warehouse robotics, robotic mobile fulfillment system, RMFS, AMR, autonomous mobile robot, discrete-event simulation, DES, reinforcement learning benchmark, Gymnasium environment, MaskablePPO, attribution-shaped reward, dispatching, task allocation, multi-agent path finding, MAPF, Kiva, fleet sizing, throughput, p95 lateness, Rust, PyO3.</sub>
