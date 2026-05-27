"""Learning curves: SLA on-time vs training timesteps, candidate vs MLP policy.

Makes the representation finding vivid: the candidate policy climbs while the
flattened MLP plateaus. Periodically evaluates the in-training policy on held-out
seeds and records (timesteps, on-time %). Writes a CSV and a PDF figure.

    python crates/waremax-gym/python/learning_curves.py \
        --reward-mode routed --timesteps 40000 --seeds 3

Run after heavy background jobs finish (training is CPU-bound).
"""

from __future__ import annotations

import argparse
import csv
import statistics
from collections import defaultdict
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
from sb3_contrib import MaskablePPO
from sb3_contrib.common.maskable.utils import get_action_masks
from stable_baselines3.common.callbacks import BaseCallback

from candidate_policy import CandidateScoringPolicy
from metrics import metrics_from_info
from train_ppo import make_env

RESULTS = Path(__file__).parent / "results"


def eval_on_time(model, scn, seeds):
    ots = []
    for s in seeds:
        env = make_env(scn, base_seed=s)
        obs, _ = env.reset(seed=s)
        info, done = {}, False
        while not done:
            masks = get_action_masks(env)
            action, _ = model.predict(obs, action_masks=masks, deterministic=True)
            obs, _r, term, trunc, info = env.step(int(action))
            done = term or trunc
        ots.append(metrics_from_info(info)["on_time_pct"])
    return float(np.mean(ots))


class CurveCallback(BaseCallback):
    def __init__(self, scn, eval_seeds, eval_freq):
        super().__init__()
        self.scn, self.eval_seeds, self.eval_freq = scn, eval_seeds, eval_freq
        self.curve = []
        self._next = eval_freq

    def _on_step(self) -> bool:
        if self.num_timesteps >= self._next:
            self.curve.append((self.num_timesteps, eval_on_time(self.model, self.scn, self.eval_seeds)))
            self._next += self.eval_freq
        return True


def train_curve(arch, scn, reward_mode, timesteps, eval_seeds, eval_freq, seed):
    env = make_env(scn, base_seed=seed, reward_mode=reward_mode)
    cls, kw = (CandidateScoringPolicy, {"hidden": 128}) if arch == "candidate" else ("MultiInputPolicy", None)
    model = MaskablePPO(cls, env, seed=seed, n_steps=512, batch_size=128, verbose=0, policy_kwargs=kw)
    cb = CurveCallback(scn, eval_seeds, eval_freq)
    print(f"  {arch} seed={seed}...")
    model.learn(total_timesteps=timesteps, callback=cb, progress_bar=False)
    return cb.curve


def aggregate(curves):
    """curves: list of [(t, ot), ...] across seeds. Returns sorted (t, mean, ci)."""
    by_t = defaultdict(list)
    for c in curves:
        for t, ot in c:
            by_t[t].append(ot)
    out = []
    for t in sorted(by_t):
        vals = by_t[t]
        m = statistics.fmean(vals)
        sd = statistics.pstdev(vals) if len(vals) > 1 else 0.0
        out.append((t, m, 1.96 * sd / max(len(vals), 1) ** 0.5))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--reward-mode", default="routed")
    ap.add_argument("--preset", default="standard")
    ap.add_argument("--duration", type=float, default=15.0)
    ap.add_argument("--due-time", type=float, default=2.0)
    ap.add_argument("--timesteps", type=int, default=40000)
    ap.add_argument("--eval-freq", type=int, default=4000)
    ap.add_argument("--eval-seeds", type=int, default=4)
    ap.add_argument("--seeds", type=int, default=3)
    args = ap.parse_args()

    scn = {"preset": args.preset, "duration_minutes": args.duration,
           "warmup_minutes": 0.0, "due_time_minutes": args.due_time}
    eval_seeds = list(range(1000, 1000 + args.eval_seeds))
    RESULTS.mkdir(exist_ok=True)

    agg = {}
    rows = []
    for arch in ["candidate", "mlp"]:
        print(f"=== {arch} ===")
        curves = [train_curve(arch, scn, args.reward_mode, args.timesteps,
                              eval_seeds, args.eval_freq, s) for s in range(args.seeds)]
        agg[arch] = aggregate(curves)
        for t, m, ci in agg[arch]:
            rows.append(dict(arch=arch, timesteps=t, on_time_pct=round(m, 2), ci=round(ci, 2)))

    with open(RESULTS / "learning_curves.csv", "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=["arch", "timesteps", "on_time_pct", "ci"])
        w.writeheader()
        w.writerows(rows)

    plt.figure(figsize=(5, 3.2))
    for arch, label, style in [("candidate", "Candidate (ours)", "-o"),
                               ("mlp", "Flattened MLP", "--s")]:
        ts = [t for t, _, _ in agg[arch]]
        ms = np.array([m for _, m, _ in agg[arch]])
        cis = np.array([c for _, _, c in agg[arch]])
        plt.plot(ts, ms, style, label=label, markersize=4)
        plt.fill_between(ts, ms - cis, ms + cis, alpha=0.2)
    plt.xlabel("training timesteps")
    plt.ylabel("SLA on-time (%)")
    plt.title(f"Learning curves ({args.reward_mode} reward, standard\\_tight)")
    plt.legend()
    plt.tight_layout()
    plt.savefig(RESULTS / "learning_curves.pdf")
    print(f"wrote {RESULTS/'learning_curves.csv'} and {RESULTS/'learning_curves.pdf'}")


if __name__ == "__main__":
    main()
