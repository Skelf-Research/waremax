//! Message protocol between the simulation worker thread and the driving agent.
//!
//! The worker sends [`Message`]s out (an observation at a decision point, or a
//! terminal at episode end); the agent sends [`ActionMsg`]s back. Two bounded(1)
//! channels enforce a strict ping-pong handshake so exactly one side runs at a
//! time — which is what preserves determinism across the boundary.

use crate::observation::Observation;

/// Per-step diagnostic info (mirrors the reward breakdown; also carries final
/// metrics on the terminal message).
#[derive(Debug, Clone, Default)]
pub struct StepInfo {
    pub completed_delta: i64,
    pub late_delta: i64,
    pub lateness_delta_s: f64,
    pub pending: usize,
    pub sim_time_s: f64,
    /// Populated only on the terminal step.
    pub final_metrics: Option<FinalMetrics>,
    /// True if the episode ended because the simulation worker panicked.
    pub errored: bool,
}

/// End-of-episode summary metrics, surfaced to the agent in `info`.
#[derive(Debug, Clone)]
pub struct FinalMetrics {
    pub orders_completed: u32,
    pub orders_late: u32,
    pub avg_cycle_time_s: f64,
    pub p95_cycle_time_s: f64,
    /// SLA p95 lateness (seconds past due); 0 when no SLA data.
    pub p95_lateness_s: f64,
    pub throughput_per_hour: f64,
    pub on_time_rate: f64,
    pub robot_utilization: f64,
}

/// A message from the simulation worker to the agent.
#[derive(Debug, Clone)]
pub enum Message {
    /// A decision is required: pick a candidate for the current task.
    Decision {
        obs: Observation,
        reward: f32,
        info: StepInfo,
    },
    /// The episode has finished; no further actions are consumed.
    Terminal {
        obs: Observation,
        reward: f32,
        info: StepInfo,
    },
}

/// A message from the agent to the simulation worker.
#[derive(Debug, Clone)]
pub enum ActionMsg {
    /// Choose the candidate at this index into `Observation::candidate_robot_ids`.
    Choose(usize),
    /// Explicitly allocate no robot this round (`allocate` returns `None`).
    NoOp,
    /// Abort the in-flight episode (used when resetting early); worker drains and exits.
    Abort,
}
