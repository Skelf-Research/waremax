"""Full experiment grid for the WareMax-RL paper.

Runs heuristics and RL reward modes (across training seeds and policy
architectures) over a grid of scenarios, persists one row per evaluated
configuration to a CSV (resumable — re-running skips completed rows, keyed
including the policy architecture), and renders summary tables with mean +/- 95%
CI (over training seeds for RL; over eval seeds for the deterministic heuristics).

    python crates/waremax-gym/python/experiments.py --quick            # smoke
    python crates/waremax-gym/python/experiments.py --scenarios standard_tight
    python crates/waremax-gym/python/experiments.py --summarize-only

A scenario is a dict of overrides forwarded to the env / baseline (preset,
horizon, due-time, fleet size, order rate, traffic capacity, ...).
"""

from __future__ import annotations

import argparse
import csv
import math
import statistics
from pathlib import Path

from train_ppo import HEURISTICS, evaluate_heuristic, evaluate_policy, train_one

# Scenario grid (keys are forwarded as env/baseline kwargs). Tight due-times
# create SLA pressure; tight node/edge capacity induces congestion.
SCENARIOS = {
    "standard_tight": dict(preset="standard", duration_minutes=15.0,
                           warmup_minutes=0.0, due_time_minutes=2.0),
    "standard_mod": dict(preset="standard", duration_minutes=15.0,
                         warmup_minutes=0.0, due_time_minutes=3.0),
    "peak_tight": dict(preset="peak_hours", duration_minutes=12.0,
                       warmup_minutes=0.0, due_time_minutes=2.0),
    # Congestion regime for the controllability ablation.
    "congested": dict(preset="standard", duration_minutes=15.0, warmup_minutes=0.0,
                      due_time_minutes=4.0, n_robots=12, order_rate=5.0,
                      node_capacity=1, edge_capacity=2),
}

METRICS = ["on_time_pct", "p95_lateness_s", "throughput_per_hour"]
RESULTS_DIR = Path(__file__).parent / "results"
RUNS_CSV = RESULTS_DIR / "runs.csv"

FIELDS = (
    ["scenario", "preset", "duration", "due_time", "policy", "reward_mode",
     "arch", "train_seed", "timesteps", "n_eval"]
    + [f"{m}_mean" for m in METRICS]
    + [f"{m}_std" for m in METRICS]
)

T95 = {1: 12.71, 2: 4.30, 3: 3.18, 4: 2.78, 5: 2.57, 6: 2.45,
       7: 2.36, 8: 2.31, 9: 2.26, 10: 2.23}


def ci95_from_values(values):
    n = len(values)
    if n == 0:
        return float("nan"), 0.0
    mean = statistics.fmean(values)
    if n == 1:
        return mean, 0.0
    sd = statistics.stdev(values)
    return mean, T95.get(n - 1, 1.96) * sd / math.sqrt(n)


def ci95_from_std(mean, std, n):
    if n <= 1:
        return mean, 0.0
    return mean, T95.get(n - 1, 1.96) * std / math.sqrt(n)


def load_done(path):
    """Completed configurations, keyed by (scenario, policy, reward_mode, arch, train_seed)."""
    done = set()
    if path.exists():
        with open(path) as f:
            for row in csv.DictReader(f):
                done.add((row["scenario"], row["policy"], row["reward_mode"],
                          row["arch"], row["train_seed"]))
    return done


def append_row(path, row):
    is_new = not path.exists()
    with open(path, "a", newline="") as f:
        w = csv.DictWriter(f, fieldnames=FIELDS)
        if is_new:
            w.writeheader()
        w.writerow(row)


def make_row(scn_name, scn, policy, reward_mode, arch, train_seed, timesteps, agg, n_eval):
    row = dict(
        scenario=scn_name, preset=scn.get("preset", ""),
        duration=scn.get("duration_minutes", ""), due_time=scn.get("due_time_minutes", ""),
        policy=policy, reward_mode=reward_mode, arch=arch, train_seed=train_seed,
        timesteps=timesteps, n_eval=n_eval,
    )
    for m in METRICS:
        mean, std = agg[m]
        row[f"{m}_mean"] = round(mean, 4)
        row[f"{m}_std"] = round(std, 4)
    return row


def run_grid(args):
    RESULTS_DIR.mkdir(exist_ok=True)
    done = load_done(RUNS_CSV)
    eval_seeds = list(range(1000, 1000 + args.eval_seeds))
    train_seeds = list(range(args.train_seeds))
    scen_names = args.scenarios or list(SCENARIOS)

    for scn_name in scen_names:
        scn = SCENARIOS[scn_name]
        print(f"\n=== scenario {scn_name}: {scn} ===")

        for h in HEURISTICS:
            if (scn_name, h, "na", "na", "-1") in done:
                continue
            agg = evaluate_heuristic(h, scn, eval_seeds)
            append_row(RUNS_CSV, make_row(scn_name, scn, h, "na", "na", -1, 0, agg, len(eval_seeds)))
            print(f"[done] {scn_name} {h}: on_time={agg['on_time_pct'][0]:.1f}%")

        for mode in args.reward_modes:
            for ts in train_seeds:
                if (scn_name, "ppo", mode, args.policy, str(ts)) in done:
                    continue
                model = train_one(mode, scn, args.timesteps, policy=args.policy, seed=ts)
                agg = evaluate_policy(model, scn, eval_seeds)
                append_row(RUNS_CSV, make_row(scn_name, scn, "ppo", mode, args.policy, ts,
                                              args.timesteps, agg, len(eval_seeds)))
                print(f"[done] {scn_name} ppo/{mode}/{args.policy} seed={ts}: "
                      f"on_time={agg['on_time_pct'][0]:.1f}%")


def summarize():
    if not RUNS_CSV.exists():
        print("no runs.csv yet")
        return
    rows = list(csv.DictReader(open(RUNS_CSV)))
    by_scn: dict[str, list] = {}
    for r in rows:
        by_scn.setdefault(r["scenario"], []).append(r)

    summary_rows = []
    md = ["# WareMax-RL results (mean +/- 95% CI)\n"]
    for scn, rs in by_scn.items():
        md.append(f"\n## {scn}\n")
        header = f"{'policy':<26}{'SLA on-time %':>16}{'p95 lateness s':>18}{'throughput/h':>16}"
        md += ["```", header, "-" * len(header)]

        entries = []  # (label, {metric: (mean, ci)}, n)
        # Heuristics: CI over eval seeds (from stored std + n_eval).
        for r in rs:
            if r["policy"] == "ppo":
                continue
            d, n = {}, int(r["n_eval"])
            for m in METRICS:
                d[m] = ci95_from_std(float(r[f"{m}_mean"]), float(r[f"{m}_std"]), n)
            entries.append((r["policy"], d, n))
        # RL: CI over training seeds, grouped by (reward_mode, arch).
        groups: dict[tuple, list] = {}
        for r in rs:
            if r["policy"] == "ppo":
                groups.setdefault((r["reward_mode"], r["arch"]), []).append(r)
        for (mode, arch), grs in sorted(groups.items()):
            d = {}
            for m in METRICS:
                d[m] = ci95_from_values([float(r[f"{m}_mean"]) for r in grs])
            entries.append((f"ppo/{mode}/{arch}", d, len(grs)))

        for label, d, n in entries:
            ot, p95, thr = d["on_time_pct"], d["p95_lateness_s"], d["throughput_per_hour"]
            md.append(
                f"{label + f' (n={n})':<26}"
                f"{f'{ot[0]:.1f}+-{ot[1]:.1f}':>16}"
                f"{f'{p95[0]:.1f}+-{p95[1]:.1f}':>18}"
                f"{f'{thr[0]:.1f}+-{thr[1]:.1f}':>16}"
            )
            summary_rows.append(dict(
                scenario=scn, policy=label, n=n,
                on_time_pct=round(ot[0], 2), on_time_ci=round(ot[1], 2),
                p95_lateness_s=round(p95[0], 2), p95_lateness_ci=round(p95[1], 2),
                throughput_per_hour=round(thr[0], 2), throughput_ci=round(thr[1], 2),
            ))
        md.append("```")

    text = "\n".join(md)
    print(text)
    (RESULTS_DIR / "tables.md").write_text(text + "\n")
    with open(RESULTS_DIR / "summary.csv", "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(summary_rows[0].keys()))
        w.writeheader()
        w.writerows(summary_rows)
    print(f"\nWrote {RESULTS_DIR/'tables.md'} and {RESULTS_DIR/'summary.csv'}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--scenarios", nargs="*", default=None,
                    help=f"subset of {list(SCENARIOS)} (default: all)")
    ap.add_argument("--reward-modes", default="sparse,dense,attribution,routed")
    ap.add_argument("--train-seeds", type=int, default=3)
    ap.add_argument("--eval-seeds", type=int, default=8)
    ap.add_argument("--timesteps", type=int, default=40000)
    ap.add_argument("--policy", default="candidate", choices=["candidate", "mlp"])
    ap.add_argument("--quick", action="store_true")
    ap.add_argument("--summarize-only", action="store_true")
    args = ap.parse_args()

    if args.quick:
        args.scenarios = ["standard_tight"]
        args.reward_modes = "dense,routed"
        args.train_seeds = 1
        args.eval_seeds = 3
        args.timesteps = 4000

    args.reward_modes = [m.strip() for m in args.reward_modes.split(",") if m.strip()]

    if not args.summarize_only:
        run_grid(args)
    summarize()


if __name__ == "__main__":
    main()
