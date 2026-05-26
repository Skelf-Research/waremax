"""Train MaskablePPO dispatchers and compare reward modes to heuristics.

Trains one MaskablePPO policy per reward mode (sparse / dense / attribution /
routed) and evaluates them, plus the heuristic baselines, on a held-out seed set
under identical conditions. A scenario is a dict of overrides forwarded to the
env / baseline (preset, horizon, due-time, fleet size, order rate, traffic
capacity), so congestion/overload regimes are first-class.

    python crates/waremax-gym/python/train_ppo.py \
        --preset standard --duration 15 --due-time 3 \
        --n-robots 10 --order-rate 4 --node-cap 1 --edge-cap 1 \
        --timesteps 40000 --reward-modes dense,attribution,routed

    python crates/waremax-gym/python/train_ppo.py --check-determinism
"""

from __future__ import annotations

import argparse
import json

import numpy as np
import waremax_gym
from sb3_contrib import MaskablePPO
from sb3_contrib.common.maskable.utils import get_action_masks
from sb3_contrib.common.wrappers import ActionMasker

from candidate_policy import CandidateScoringPolicy
from metrics import aggregate, format_table, metrics_from_info, metrics_from_report
from waremax_alloc_env import WaremaxAllocEnv

HEURISTICS = ["nearest_robot", "least_busy", "round_robin", "auction"]


def mask_fn(env):
    return env.action_masks()


def make_env(scn, base_seed, reward_mode=None):
    env = WaremaxAllocEnv(base_seed=base_seed, reward_mode=reward_mode, **scn)
    return ActionMasker(env, mask_fn)


def evaluate_policy(model, scn, seeds):
    rows = []
    for s in seeds:
        env = make_env(scn, base_seed=s)
        obs, _ = env.reset(seed=s)
        info, done = {}, False
        while not done:
            masks = get_action_masks(env)
            action, _ = model.predict(obs, action_masks=masks, deterministic=True)
            obs, _r, term, trunc, info = env.step(int(action))
            done = term or trunc
        rows.append(metrics_from_info(info))
    return aggregate(rows)


def evaluate_heuristic(alloc, scn, seeds):
    rows = []
    for s in seeds:
        report = json.loads(waremax_gym.run_baseline(alloc, seed=s, **scn))
        rows.append(metrics_from_report(report))
    return aggregate(rows)


def train_one(reward_mode, scn, timesteps, policy="candidate", seed=0):
    env = make_env(scn, base_seed=seed, reward_mode=reward_mode)
    if policy == "candidate":
        policy_cls, policy_kwargs = CandidateScoringPolicy, {"hidden": 128}
    else:
        policy_cls, policy_kwargs = "MultiInputPolicy", None
    model = MaskablePPO(
        policy_cls, env, seed=seed, n_steps=512, batch_size=128, verbose=0,
        policy_kwargs=policy_kwargs,
    )
    print(f"  training reward_mode={reward_mode} policy={policy} seed={seed} "
          f"for {timesteps} timesteps...")
    model.learn(total_timesteps=timesteps, progress_bar=False)
    return model


def check_determinism(scn):
    def run():
        env = WaremaxAllocEnv(reward_mode="attribution", **scn)
        obs, _ = env.reset(seed=42)
        ret, done = 0.0, False
        while not done:
            valid = np.flatnonzero(obs["action_mask"].astype(bool))
            obs, r, term, trunc, _ = env.step(int(valid[0]))
            ret += r
            done = term or trunc
        return round(ret, 4), env.last_report()["events_processed"]

    a, b = run(), run()
    ok = a == b
    print(f"[determinism] run1={a} run2={b} reproducible={ok}")
    return ok


def scenario_from_args(args):
    scn = {"preset": args.preset, "duration_minutes": args.duration,
           "warmup_minutes": args.warmup}
    if args.due_time is not None:
        scn["due_time_minutes"] = args.due_time
    if args.n_robots is not None:
        scn["n_robots"] = args.n_robots
    if args.order_rate is not None:
        scn["order_rate"] = args.order_rate
    if args.node_cap is not None:
        scn["node_capacity"] = args.node_cap
    if args.edge_cap is not None:
        scn["edge_capacity"] = args.edge_cap
    return scn


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--timesteps", type=int, default=40000)
    ap.add_argument("--preset", default="standard")
    ap.add_argument("--duration", type=float, default=15.0)
    ap.add_argument("--warmup", type=float, default=0.0)
    ap.add_argument("--due-time", type=float, default=None)
    ap.add_argument("--n-robots", type=int, default=None)
    ap.add_argument("--order-rate", type=float, default=None)
    ap.add_argument("--node-cap", type=int, default=None)
    ap.add_argument("--edge-cap", type=int, default=None)
    ap.add_argument("--eval-seeds", type=int, default=8)
    ap.add_argument("--reward-modes", default="sparse,dense,attribution,routed")
    ap.add_argument("--policy", default="candidate", choices=["candidate", "mlp"])
    ap.add_argument("--check-determinism", action="store_true")
    args = ap.parse_args()

    scn = scenario_from_args(args)

    if args.check_determinism:
        raise SystemExit(0 if check_determinism(scn) else 1)

    eval_seeds = list(range(1000, 1000 + args.eval_seeds))  # held-out
    results = {alloc: evaluate_heuristic(alloc, scn, eval_seeds) for alloc in HEURISTICS}

    print(f"Training MaskablePPO  scn={scn}")
    for mode in [m.strip() for m in args.reward_modes.split(",") if m.strip()]:
        model = train_one(mode, scn, args.timesteps, policy=args.policy)
        results[f"ppo_{mode}"] = evaluate_policy(model, scn, eval_seeds)

    print(f"\nDispatching policies  scn={scn}, {len(eval_seeds)} held-out seeds\n")
    print(format_table(results))


if __name__ == "__main__":
    main()
