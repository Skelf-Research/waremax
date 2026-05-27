"""Run heuristic dispatching baselines on a scenario across seeds.

Each heuristic is evaluated on identical (scenario, seed) pairs via the Rust
`run_baseline` entry point, so comparisons are apples-to-apples. Only the
allocation policy differs; all other policies are the scenario defaults.

    python crates/waremax-gym/python/baseline.py --preset quick --seeds 5
"""

from __future__ import annotations

import argparse
import json

import waremax_gym

from metrics import aggregate, format_table, metrics_from_report

HEURISTICS = ["nearest_robot", "least_busy", "round_robin", "auction"]


def run(preset, duration, warmup, due_time, seeds):
    results = {}
    for alloc in HEURISTICS:
        rows = []
        for seed in seeds:
            report = json.loads(
                waremax_gym.run_baseline(
                    alloc,
                    preset=preset,
                    seed=seed,
                    duration_minutes=duration,
                    warmup_minutes=warmup,
                    due_time_minutes=due_time,
                )
            )
            rows.append(metrics_from_report(report))
        results[alloc] = aggregate(rows)
    return results


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--preset", default="quick")
    ap.add_argument("--duration", type=float, default=20.0)
    ap.add_argument("--warmup", type=float, default=0.0)
    ap.add_argument("--due-time", type=float, default=None,
                    help="fixed due-time in minutes (tighten to create SLA pressure)")
    ap.add_argument("--seeds", type=int, default=8, help="number of seeds (0..N-1)")
    args = ap.parse_args()

    seeds = list(range(args.seeds))
    results = run(args.preset, args.duration, args.warmup, args.due_time, seeds)
    print(f"\nHeuristic baselines  (preset={args.preset}, {args.duration} min, "
          f"due_time={args.due_time}, {len(seeds)} seeds)\n")
    print(format_table(results))


if __name__ == "__main__":
    main()
