"""Candidate-centric (permutation-equivariant) policy for MaskablePPO.

The default ``MultiInputPolicy`` flattens the (MAX_ROBOTS x ROBOT_FEATS)
candidate matrix and maps it to MAX_ROBOTS logits with a dense layer — it has to
learn, from scratch and over a mostly-zero padded input, how to *compare*
candidates. That plateaus near the weakest heuristic regardless of reward shaping
or training budget.

This policy instead applies a *shared* scorer to each candidate's features
(concatenated with the task features), producing one logit per candidate. Because
the same network scores every candidate, the inductive bias is exactly "pick the
best candidate by its features" (a pointer/deep-sets style head). The critic is a
masked-mean pool over valid candidates. Invalid/padding rows are masked out.
"""

from __future__ import annotations

import torch as th
import torch.nn as nn
from sb3_contrib.common.maskable.policies import MaskableActorCriticPolicy


class CandidateScoringPolicy(MaskableActorCriticPolicy):
    def __init__(self, observation_space, action_space, lr_schedule, hidden: int = 128, **kwargs):
        self._hidden = hidden
        # net_arch is unused (we build our own nets), but accept/ignore it.
        kwargs.pop("net_arch", None)
        super().__init__(observation_space, action_space, lr_schedule, **kwargs)

    def _build(self, lr_schedule) -> None:
        rf = int(self.observation_space["robots"].shape[1])  # ROBOT_FEATS
        tf = int(self.observation_space["task"].shape[0])    # TASK_FEATS
        h = self._hidden
        in_dim = rf + tf

        # Shared per-candidate scorer -> 1 logit per candidate.
        self.candidate_scorer = nn.Sequential(
            nn.Linear(in_dim, h), nn.Tanh(),
            nn.Linear(h, h), nn.Tanh(),
            nn.Linear(h, 1),
        )
        # Per-candidate value features, masked-mean pooled, then a scalar head.
        self.value_feat = nn.Sequential(
            nn.Linear(in_dim, h), nn.Tanh(),
            nn.Linear(h, h), nn.Tanh(),
        )
        self.value_out = nn.Linear(h, 1)

        self.optimizer = self.optimizer_class(
            self.parameters(), lr=lr_schedule(1), **self.optimizer_kwargs
        )

    # --- helpers -----------------------------------------------------------
    def _prep(self, obs):
        robots = obs["robots"]                     # (B, N, rf)
        task = obs["task"]                         # (B, tf)
        mask = obs["action_mask"].float()          # (B, N)
        b, n, _ = robots.shape
        task_exp = task.unsqueeze(1).expand(b, n, task.shape[-1])
        cat = th.cat([robots, task_exp], dim=-1)   # (B, N, rf+tf)
        return cat, mask

    def _logits(self, obs):
        cat, _ = self._prep(obs)
        return self.candidate_scorer(cat).squeeze(-1)  # (B, N)

    def _value(self, obs):
        cat, mask = self._prep(obs)
        feats = self.value_feat(cat)                   # (B, N, h)
        m = mask.unsqueeze(-1)                         # (B, N, 1)
        pooled = (feats * m).sum(dim=1) / m.sum(dim=1).clamp(min=1.0)
        return self.value_out(pooled)                  # (B, 1)

    def _dist(self, obs, action_masks):
        distribution = self.action_dist.proba_distribution(action_logits=self._logits(obs))
        if action_masks is not None:
            distribution.apply_masking(action_masks)
        return distribution

    # --- overrides used by MaskablePPO ------------------------------------
    def forward(self, obs, deterministic: bool = False, action_masks=None):
        distribution = self._dist(obs, action_masks)
        actions = distribution.get_actions(deterministic=deterministic)
        log_prob = distribution.log_prob(actions)
        return actions, self._value(obs), log_prob

    def evaluate_actions(self, obs, actions, action_masks=None):
        distribution = self._dist(obs, action_masks)
        return self._value(obs), distribution.log_prob(actions), distribution.entropy()

    def get_distribution(self, obs, action_masks=None):
        return self._dist(obs, action_masks)

    def predict_values(self, obs):
        return self._value(obs)

    def _predict(self, observation, deterministic: bool = False, action_masks=None):
        return self._dist(observation, action_masks).get_actions(deterministic=deterministic)
