"""Gymnasium environment for waremax task-allocation dispatching.

Wraps the compiled ``waremax_gym`` extension module. The action is a
masked-discrete choice of robot for the current pick task; the observation is a
Dict of fixed-shape arrays plus a boolean action mask. Use with
``sb3_contrib.MaskablePPO`` (vanilla PPO cannot mask invalid actions).
"""

from __future__ import annotations

import numpy as np
import gymnasium as gym
from gymnasium import spaces

import waremax_gym  # compiled extension (maturin develop)


class WaremaxAllocEnv(gym.Env):
    """Single-agent task-allocation MDP over one waremax scenario."""

    metadata = {"render_modes": []}

    def __init__(self, reward_mode: str | None = None, base_seed: int = 0, **scenario_kwargs):
        """`scenario_kwargs` are forwarded to ``waremax_gym.WaremaxEnv``: any of
        preset, scenario_path, duration_minutes, warmup_minutes, due_time_minutes,
        n_robots, order_rate, node_capacity, edge_capacity."""
        super().__init__()
        self._env = waremax_gym.WaremaxEnv(reward_mode=reward_mode, **scenario_kwargs)
        self.max_robots = self._env.max_robots
        self.robot_feats = self._env.robot_feats
        self.task_feats = self._env.task_feats

        self.observation_space = spaces.Dict(
            {
                "robots": spaces.Box(
                    low=-1.0,
                    high=1.0,
                    shape=(self.max_robots, self.robot_feats),
                    dtype=np.float32,
                ),
                "task": spaces.Box(
                    low=-1.0, high=1.0, shape=(self.task_feats,), dtype=np.float32
                ),
                "action_mask": spaces.Box(
                    low=0, high=1, shape=(self.max_robots,), dtype=np.int8
                ),
            }
        )
        self.action_space = spaces.Discrete(self.max_robots)

        # Reproducible sequence of per-episode seeds (for generalization across
        # episodes while keeping the whole training run reproducible).
        self._seed_rng = np.random.default_rng(base_seed)
        self._mask = np.zeros(self.max_robots, dtype=bool)

    def _make_obs(self, robot_feats, mask, task_feats):
        self._mask = np.asarray(mask, dtype=bool)
        return {
            "robots": np.asarray(robot_feats, dtype=np.float32).reshape(
                self.max_robots, self.robot_feats
            ),
            "task": np.asarray(task_feats, dtype=np.float32),
            "action_mask": self._mask.astype(np.int8),
        }

    def reset(self, *, seed=None, options=None):
        super().reset(seed=seed)
        if seed is None:
            seed = int(self._seed_rng.integers(0, 2**63 - 1))
        rf, mask, tf = self._env.reset(int(seed))
        return self._make_obs(rf, mask, tf), {}

    def step(self, action):
        rf, mask, tf, reward, done, info = self._env.step(int(action))
        obs = self._make_obs(rf, mask, tf)
        # SMDP episodes end naturally (time horizon), so this is termination,
        # not truncation.
        return obs, float(reward), bool(done), False, info

    def action_masks(self) -> np.ndarray:
        """Valid-action mask for sb3-contrib MaskablePPO."""
        return self._mask

    def last_report(self) -> dict | None:
        import json

        js = self._env.last_report_json()
        return json.loads(js) if js else None


def make_env(**kwargs) -> WaremaxAllocEnv:
    return WaremaxAllocEnv(**kwargs)
