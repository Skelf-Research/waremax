//! Observation construction for the RL task-allocation MDP.
//!
//! An [`Observation`] is a fully-owned, fixed-shape snapshot of the world at a
//! single allocation decision point. It is built from a [`PolicyContext`] and
//! must contain no borrows so it can be sent across a channel to the agent.
//!
//! Candidate robots (the available ones) occupy the first rows of `robot_feats`,
//! sorted by `RobotId.0` for determinism (HashMap iteration order is otherwise
//! randomized per-process), and `action_mask[i]` marks row `i` as a real
//! candidate. The action is an index into these rows.

use waremax_core::{RobotId, TaskId};
use waremax_entities::Robot;
use waremax_policies::PolicyContext;

/// Maximum number of robots the observation/action space supports.
pub const MAX_ROBOTS: usize = 64;
/// Number of features per robot row.
pub const ROBOT_FEATS: usize = 8;
/// Number of task-level features.
pub const TASK_FEATS: usize = 6;

// Normalization constants (rough; exact values are not critical for a baseline).
const DIST_NORM: f64 = 50.0;
const TRAVEL_NORM: f64 = 1000.0;
const QUEUE_NORM: f64 = 10.0;
const TASKS_DONE_NORM: f64 = 100.0;
const PENDING_NORM: f64 = 50.0;
const DUE_CLAMP_MIN: f64 = 120.0; // +/- 120 minutes window
const PHASE_NORM_S: f64 = 7200.0; // 2-hour phase scale

/// A fixed-shape, fully-owned observation at one allocation decision point.
#[derive(Debug, Clone)]
pub struct Observation {
    /// Flattened `MAX_ROBOTS * ROBOT_FEATS`; row `r` is candidate `r` (or zero-padded).
    pub robot_feats: Vec<f32>,
    /// Length `MAX_ROBOTS`; `true` if row `r` is a real available candidate.
    pub action_mask: Vec<bool>,
    /// Length `TASK_FEATS`; features of the task being allocated.
    pub task_feats: Vec<f32>,
    /// Actual `RobotId` for each filled row, in row order. Length == number of candidates.
    /// Used to decode the agent's action index back into a robot.
    pub candidate_robot_ids: Vec<RobotId>,
}

impl Observation {
    /// An all-zero observation with an all-false mask (used for terminal steps).
    pub fn zeros() -> Self {
        Self {
            robot_feats: vec![0.0; MAX_ROBOTS * ROBOT_FEATS],
            action_mask: vec![false; MAX_ROBOTS],
            task_feats: vec![0.0; TASK_FEATS],
            candidate_robot_ids: Vec::new(),
        }
    }
}

/// Build an observation for allocating `task_id`.
///
/// Returns `None` when there are no available candidate robots — the caller
/// treats this as "no decision needed" and never consults the agent, keeping
/// every emitted decision guaranteed to have at least one valid action.
pub fn build_observation(ctx: &PolicyContext, task_id: TaskId) -> Option<Observation> {
    let task = ctx.tasks.get(&task_id)?;
    let pickup = task.source.access_node;

    // Collect available robots and sort by id.0 for determinism.
    let mut candidates: Vec<&Robot> = ctx.robots.values().filter(|r| r.is_available()).collect();
    if candidates.is_empty() {
        return None;
    }
    candidates.sort_by_key(|r| r.id.0);
    candidates.truncate(MAX_ROBOTS);

    let (px, py) = node_xy(ctx, pickup);

    let mut robot_feats = vec![0.0f32; MAX_ROBOTS * ROBOT_FEATS];
    let mut action_mask = vec![false; MAX_ROBOTS];
    let mut candidate_robot_ids = Vec::with_capacity(candidates.len());

    for (row, robot) in candidates.iter().enumerate() {
        let (rx, ry) = node_xy(ctx, robot.current_node);
        let dist = ctx.map.euclidean_distance(robot.current_node, pickup);
        let base = row * ROBOT_FEATS;
        robot_feats[base] = norm(dist, DIST_NORM);
        robot_feats[base + 1] = robot.battery.soc as f32;
        robot_feats[base + 2] = norm(robot.task_queue.len() as f64, QUEUE_NORM);
        robot_feats[base + 3] = 1.0; // availability flag (always 1 for a candidate row)
        robot_feats[base + 4] = norm(robot.tasks_completed as f64, TASKS_DONE_NORM);
        robot_feats[base + 5] = ((rx - px) / DIST_NORM).clamp(-1.0, 1.0) as f32;
        robot_feats[base + 6] = ((ry - py) / DIST_NORM).clamp(-1.0, 1.0) as f32;
        robot_feats[base + 7] = norm(robot.total_distance, TRAVEL_NORM);
        action_mask[row] = true;
        candidate_robot_ids.push(robot.id);
    }

    // Task features.
    let now_s = ctx.current_time.as_seconds();
    let time_to_due = match task.order_id.and_then(|oid| ctx.orders.get(&oid)) {
        Some(order) => match order.due_time {
            Some(due) => {
                let mins = (due.as_seconds() - now_s) / 60.0;
                (mins / DUE_CLAMP_MIN).clamp(-1.0, 1.0)
            }
            None => 0.0,
        },
        None => 0.0,
    };
    let pending = ctx.tasks.values().filter(|t| t.is_pending()).count();
    let station_queue = ctx
        .stations
        .get(&task.destination_station)
        .map(|s| s.queue_length())
        .unwrap_or(0);

    let task_feats = vec![
        time_to_due as f32,
        norm(task.quantity as f64, QUEUE_NORM),
        norm(pending as f64, PENDING_NORM),
        (candidates.len() as f64 / MAX_ROBOTS as f64) as f32,
        (now_s / PHASE_NORM_S).clamp(0.0, 1.0) as f32,
        norm(station_queue as f64, QUEUE_NORM),
    ];

    Some(Observation {
        robot_feats,
        action_mask,
        task_feats,
        candidate_robot_ids,
    })
}

fn node_xy(ctx: &PolicyContext, node: waremax_core::NodeId) -> (f64, f64) {
    ctx.map.get_node(node).map(|n| (n.x, n.y)).unwrap_or((0.0, 0.0))
}

fn norm(v: f64, scale: f64) -> f32 {
    (v / scale).clamp(0.0, 1.0) as f32
}
