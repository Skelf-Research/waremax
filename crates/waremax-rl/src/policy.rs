//! The `RlPolicy`: a `TaskAllocationPolicy` whose decisions come from an
//! external agent over channels.
//!
//! `allocate` builds an observation, computes the reward accrued since the
//! previous decision, sends both to the agent, and blocks for the chosen
//! action. Control is inverted without touching the event handler: from the
//! simulator's perspective this is just another allocation policy.

use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};
use waremax_core::{RobotId, TaskId};
use waremax_policies::{PolicyContext, TaskAllocationPolicy};

use crate::observation::build_observation;
use crate::protocol::{ActionMsg, Message, StepInfo};
use crate::reward::{self, RewardConfig, RewardMode, RewardSnapshot};

/// Allocation policy driven by an external agent over channels.
pub struct RlPolicy {
    obs_tx: Sender<Message>,
    action_rx: Receiver<ActionMsg>,
    reward_cfg: RewardConfig,
    /// Shared with the env so the terminal reward delta can be computed against
    /// the last decision's snapshot. Interior mutability is required because the
    /// trait method takes `&self` and the trait is `Sync`.
    snapshot: Arc<Mutex<RewardSnapshot>>,
    /// Routed mode only: the controllable cost of the *previous* decision,
    /// charged to it by emitting it with the reward returned for that action.
    pending_decision_cost: Mutex<f32>,
}

impl RlPolicy {
    pub fn new(
        obs_tx: Sender<Message>,
        action_rx: Receiver<ActionMsg>,
        reward_cfg: RewardConfig,
        snapshot: Arc<Mutex<RewardSnapshot>>,
    ) -> Self {
        Self {
            obs_tx,
            action_rx,
            reward_cfg,
            snapshot,
            pending_decision_cost: Mutex::new(0.0),
        }
    }

    /// Controllable cost of assigning `chosen` to `task_id`: estimated
    /// travel-to-pickup time plus the robot's current backlog. Zero for a no-op.
    fn routed_decision_cost(
        &self,
        ctx: &PolicyContext,
        task_id: TaskId,
        chosen: Option<RobotId>,
    ) -> f32 {
        let (Some(robot_id), Some(task)) = (chosen, ctx.tasks.get(&task_id)) else {
            return 0.0;
        };
        let Some(robot) = ctx.robots.get(&robot_id) else {
            return 0.0;
        };
        let dist = ctx
            .map
            .euclidean_distance(robot.current_node, task.source.access_node);
        let travel_min = (dist / robot.max_speed_mps.max(0.1)) / 60.0;
        let backlog = robot.task_queue.len() as f64;
        reward::decision_cost(travel_min, backlog, &self.reward_cfg)
    }
}

impl TaskAllocationPolicy for RlPolicy {
    fn allocate(&self, ctx: &PolicyContext, task_id: TaskId) -> Option<RobotId> {
        // No available candidates => no decision; never consult the agent.
        let obs = build_observation(ctx, task_id)?;
        let candidate_ids = obs.candidate_robot_ids.clone();

        // Reward = global delta since the previous decision; then advance the
        // snapshot. In routed mode, subtract the previous decision's own
        // controllable cost (charged to the action that incurred it).
        let cur = reward::snapshot_from(ctx.orders, ctx.tasks, ctx.attribution);
        let (reward, info) = {
            let mut prev = self.snapshot.lock().unwrap();
            let mut reward = reward::delta(&prev, &cur, &self.reward_cfg);
            if self.reward_cfg.mode == RewardMode::Routed {
                reward -= *self.pending_decision_cost.lock().unwrap();
            }
            let info = StepInfo {
                completed_delta: cur.completed as i64 - prev.completed as i64,
                late_delta: cur.late as i64 - prev.late as i64,
                lateness_delta_s: cur.cum_lateness_s - prev.cum_lateness_s,
                pending: cur.pending,
                sim_time_s: ctx.current_time.as_seconds(),
                final_metrics: None,
                errored: false,
            };
            *prev = cur;
            (reward, info)
        };

        // Hand the decision to the agent and block for its action. A send/recv
        // error means the env was dropped mid-episode: bail out gracefully.
        if self
            .obs_tx
            .send(Message::Decision { obs, reward, info })
            .is_err()
        {
            return None;
        }

        let chosen = match self.action_rx.recv() {
            Ok(ActionMsg::Choose(i)) => candidate_ids.get(i).copied(),
            Ok(ActionMsg::NoOp) | Ok(ActionMsg::Abort) | Err(_) => None,
        };

        // Routed mode: compute the controllable cost of THIS assignment and
        // stash it so it is charged to this action on the next reward emission.
        if self.reward_cfg.mode == RewardMode::Routed {
            let cost = self.routed_decision_cost(ctx, task_id, chosen);
            *self.pending_decision_cost.lock().unwrap() = cost;
        }

        chosen
    }

    fn name(&self) -> &'static str {
        "rl_agent"
    }
}
