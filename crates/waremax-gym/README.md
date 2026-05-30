# waremax-gym

**Python (PyO3) bindings exposing the [WareMax](../../README.md) warehouse-dispatching environment as a [Gymnasium](https://gymnasium.farama.org/) env, with a permutation-equivariant MaskablePPO policy and a resumable experiment-grid harness.**

Part of the [WareMax](../../README.md) workspace — a deterministic warehouse-robotics simulator and reinforcement-learning benchmark for Robotic Mobile Fulfillment Systems (RMFS).

## Role

This crate is the bridge that makes WareMax's RL control seam ([`waremax-rl`](../waremax-rl/)) usable from Python: a `cdylib` extension module exposes `WaremaxEnv` (the raw env) and `run_baseline` (heuristic-on-identical-seed evaluator). A thin Python package wraps the env as `gymnasium.Env`, ships a candidate-scoring `MaskableActorCriticPolicy`, and includes training/baseline/experiment scripts.

## Build

```bash
python -m venv .venv && . .venv/bin/activate
pip install -r python/requirements.txt
pip install torch --index-url https://download.pytorch.org/whl/cpu  # for training
maturin develop -m Cargo.toml --release
```

## Layout

```
crates/waremax-gym/
├── Cargo.toml                 # pyo3 cdylib (waremax_gym)
├── pyproject.toml             # maturin build backend
├── src/lib.rs                 # PyO3: WaremaxEnv + run_baseline
└── python/
    ├── waremax_alloc_env.py   # gymnasium.Env wrapper
    ├── candidate_policy.py    # MaskableActorCriticPolicy (per-candidate scorer)
    ├── train_ppo.py           # MaskablePPO training + held-out comparison
    ├── baseline.py            # heuristic baseline runner
    ├── experiments.py         # resumable multi-scenario grid
    ├── learning_curves.py     # candidate-vs-MLP sample-efficiency curves
    ├── metrics.py             # metric extraction + CI table rendering
    └── results/               # persisted CSVs, tables, figures
```

## Quickstart

### Heuristic baselines

```bash
PYTHONPATH=python python python/baseline.py --preset standard --duration 15 --due-time 2 --seeds 8
```

### Train MaskablePPO and compare reward modes

```bash
PYTHONPATH=python python python/train_ppo.py \
    --preset standard --duration 15 --due-time 2 \
    --timesteps 40000 --eval-seeds 8 \
    --reward-modes sparse,dense,attribution,routed \
    --policy candidate
```

### Use the env directly

```python
from waremax_alloc_env import WaremaxAllocEnv
env = WaremaxAllocEnv(preset="standard", duration_minutes=15,
                      due_time_minutes=2, reward_mode="routed")
obs, _ = env.reset(seed=42)
# observation_space = Dict({"robots": (64, 8), "task": (6,), "action_mask": (64,)})
# action_space      = Discrete(64), use sb3-contrib MaskablePPO with env.action_masks()
```

### Run the full experiment grid

```bash
PYTHONPATH=python python python/experiments.py --train-seeds 3 --eval-seeds 8 --timesteps 40000
# writes results/runs.csv (resumable), results/summary.csv, results/tables.md
```

## Scenario knobs (forwarded as kwargs)

`preset`, `scenario_path`, `duration_minutes`, `warmup_minutes`, `due_time_minutes`, `n_robots`, `order_rate`, `node_capacity`, `edge_capacity`, `congestion_weight`, `smart_bins`, `inventory_skus`. See [main README — Configuration](../../README.md#configuration).

## Determinism across the FFI boundary

The Python `--check-determinism` mode asserts that `(seed, action script)` produces identical returns and event counts across runs, confirming that the Rust↔Python handshake preserves reproducibility.

## Used by

- The WareMax research grid (results in [`python/results/`](python/results/)).

## See also

- [`waremax-rl`](../waremax-rl/) — the underlying Rust control seam.
- [WareMax README — Reinforcement Learning Quickstart](../../README.md#reinforcement-learning-quickstart).
