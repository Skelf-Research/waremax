"""Phase B smoke + reproducibility check for the Python env.

Run after `maturin develop`:
    python crates/waremax-gym/python/smoke_test.py
"""

import sys
import numpy as np

from waremax_alloc_env import WaremaxAllocEnv


def rollout(env, seed, agent="first"):
    """Run one episode. `agent`='first' picks the lowest-index valid action."""
    obs, _ = env.reset(seed=seed)
    total_r = 0.0
    steps = 0
    rewards = []
    while True:
        mask = obs["action_mask"].astype(bool)
        valid = np.flatnonzero(mask)
        assert valid.size > 0, "non-terminal step must have at least one valid action"
        action = int(valid[0]) if agent == "first" else int(np.random.choice(valid))
        obs, r, term, trunc, info = env.step(action)
        total_r += r
        rewards.append(round(r, 4))
        steps += 1
        if term or trunc:
            break
        assert steps < 1_000_000, "episode did not terminate"
    return steps, total_r, rewards, env.last_report()


def main():
    env = WaremaxAllocEnv(preset="quick", duration_minutes=10.0, warmup_minutes=0.0)

    # 1) Random masked rollout reaches done without deadlock.
    s, r, _, rep = rollout(env, seed=5, agent="random")
    print(f"[smoke] random rollout: steps={s} return={r:.3f} "
          f"orders_completed={rep['orders_completed']}")

    # 2) Determinism across the FFI boundary: same seed + same action script
    #    must reproduce exactly (this is the property the benchmark depends on).
    s1, r1, rw1, rep1 = rollout(env, seed=42, agent="first")
    s2, r2, rw2, rep2 = rollout(env, seed=42, agent="first")
    ok = (
        s1 == s2
        and rw1 == rw2
        and rep1["events_processed"] == rep2["events_processed"]
        and rep1["orders_completed"] == rep2["orders_completed"]
    )
    print(f"[determinism] steps={s1}/{s2} events={rep1['events_processed']}/"
          f"{rep2['events_processed']} reproducible={ok}")
    if not ok:
        print("FAILED: episode not reproducible across resets", file=sys.stderr)
        sys.exit(1)

    print("PHASE B SMOKE: PASS")


if __name__ == "__main__":
    main()
