//! Reward computation for the task-allocation MDP.
//!
//! Reward is the *delta* of cumulative world aggregates between two consecutive
//! allocation decisions. All terms are built from order/task aggregates (counts
//! and sums) so they are independent of HashMap iteration order, hence
//! deterministic.
//!
//! Three modes are supported:
//! - [`RewardMode::Sparse`]: throughput minus an SLA-miss penalty only.
//! - [`RewardMode::Dense`]: hand-designed shaping (throughput, lateness, backlog).
//! - [`RewardMode::Attribution`]: the contribution — penalize the *causally
//!   attributed* delay components an allocation decision actually influences
//!   (assignment/queue/congestion "waste" and travel-to-pickup), using the
//!   simulator's per-task delay attribution, instead of a flat lateness penalty.

use std::collections::HashMap;
use waremax_analysis::{AttributionCollector, DelayCategory};
use waremax_core::{OrderId, TaskId};
use waremax_entities::{Order, Task};

/// Which reward signal to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewardMode {
    Sparse,
    Dense,
    Attribution,
    /// Ablation: attribution that also penalizes allocation-uncontrollable delay
    /// (congestion + station queue). Used to demonstrate the controllability
    /// principle (it should underperform `Attribution`).
    AttributionFull,
    /// Per-decision *routed* credit: each assignment's controllable cost
    /// (estimated travel-to-pickup + the chosen robot's backlog) is charged to
    /// the exact decision that made it, rather than smeared across completions.
    Routed,
}

impl RewardMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sparse" => Some(Self::Sparse),
            "dense" => Some(Self::Dense),
            "attribution" => Some(Self::Attribution),
            "attribution_full" => Some(Self::AttributionFull),
            "routed" => Some(Self::Routed),
            _ => None,
        }
    }

    /// Whether this mode needs the attribution collector enabled.
    pub fn needs_attribution(self) -> bool {
        matches!(self, Self::Attribution | Self::AttributionFull)
    }
}

/// Controllable cost of a single allocation decision, charged to the decision
/// that made it (used by [`RewardMode::Routed`]). `travel_min` is the estimated
/// travel-to-pickup time for the chosen robot; `backlog` is how many tasks were
/// already queued on it (a proxy for assignment/queue delay this task inherits).
pub fn decision_cost(travel_min: f64, backlog: f64, cfg: &RewardConfig) -> f32 {
    (cfg.w_travel * travel_min + cfg.w_backlog * backlog) as f32
}

/// Weights for the reward terms. Defaults are a reasonable starting point.
#[derive(Debug, Clone)]
pub struct RewardConfig {
    pub mode: RewardMode,
    /// Reward per newly completed order (all modes).
    pub w_throughput: f64,
    /// Penalty per newly late order (sparse, dense).
    pub w_late: f64,
    /// Penalty per minute of newly accrued lateness (dense).
    pub w_lateness: f64,
    /// Penalty proportional to the current pending-task backlog (dense).
    pub w_pending: f64,
    /// Penalty per minute of newly attributed "waste" delay — assignment wait,
    /// congestion, queueing (attribution mode).
    pub w_waste: f64,
    /// Penalty per minute of attributed/estimated travel-to-pickup
    /// (attribution and routed modes).
    pub w_travel: f64,
    /// Penalty per queued task on the chosen robot at decision time (routed mode).
    pub w_backlog: f64,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            mode: RewardMode::Dense,
            w_throughput: 1.0,
            w_late: 2.0,
            w_lateness: 0.1,
            w_pending: 0.01,
            w_waste: 0.2,
            w_travel: 0.1,
            w_backlog: 0.1,
        }
    }
}

/// Cumulative aggregates captured at a point in time.
#[derive(Debug, Clone, Default)]
pub struct RewardSnapshot {
    pub completed: u32,
    pub late: u32,
    pub cum_lateness_s: f64,
    pub pending: usize,
    /// Cumulative attributed CONTROLLABLE waste (assignment wait), seconds.
    pub attr_waste_s: f64,
    /// Cumulative attributed travel-to-pickup over completed tasks (seconds).
    pub attr_travel_s: f64,
    /// Cumulative attributed UNCONTROLLABLE delay (congestion + station queue),
    /// seconds. Used only by the AttributionFull ablation.
    pub attr_uncontrollable_s: f64,
}

/// Compute a snapshot from the current order/task maps and (optional) attribution.
pub fn snapshot_from(
    orders: &HashMap<OrderId, Order>,
    tasks: &HashMap<TaskId, Task>,
    attribution: Option<&AttributionCollector>,
) -> RewardSnapshot {
    let mut completed = 0u32;
    let mut late = 0u32;
    let mut cum_lateness_s = 0.0;

    for order in orders.values() {
        if order.is_complete() {
            completed += 1;
            if let (Some(due), Some(comp)) = (order.due_time, order.completion_time) {
                let lateness = comp.as_seconds() - due.as_seconds();
                if lateness > 0.0 {
                    late += 1;
                    cum_lateness_s += lateness;
                }
            }
        }
    }

    let pending = tasks.values().filter(|t| t.is_pending()).count();

    // Attributed delay over completed tasks, restricted to the components the
    // *allocation* decision actually controls: assignment wait (assign promptly
    // to an available robot) and travel-to-pickup (pick a near robot). We
    // deliberately EXCLUDE congestion and station-queue: under allocation-only
    // control the agent cannot influence them, and penalizing such uncontrollable
    // delay injects reward variance that degrades learning (shown empirically).
    let (attr_waste_s, attr_travel_s, attr_uncontrollable_s) = match attribution {
        Some(ac) => {
            let mut waste = 0.0;
            let mut travel = 0.0;
            let mut uncontrollable = 0.0;
            for attr in ac.completed_attributions() {
                for (cat, secs) in &attr.time_breakdown {
                    match cat {
                        DelayCategory::TravelToPickup => travel += *secs,
                        DelayCategory::RobotAssignment => waste += *secs,
                        // Allocation-uncontrollable delay (for the AttributionFull ablation).
                        DelayCategory::CongestionWait | DelayCategory::StationQueue => {
                            uncontrollable += *secs
                        }
                        _ => {}
                    }
                }
            }
            (waste, travel, uncontrollable)
        }
        None => (0.0, 0.0, 0.0),
    };

    RewardSnapshot {
        completed,
        late,
        cum_lateness_s,
        pending,
        attr_waste_s,
        attr_travel_s,
        attr_uncontrollable_s,
    }
}

/// Reward earned between `prev` and `cur`, per the configured mode.
pub fn delta(prev: &RewardSnapshot, cur: &RewardSnapshot, cfg: &RewardConfig) -> f32 {
    let d_completed = (cur.completed as i64 - prev.completed as i64) as f64;
    let d_late = (cur.late as i64 - prev.late as i64) as f64;
    let d_lateness = cur.cum_lateness_s - prev.cum_lateness_s;
    let d_waste = cur.attr_waste_s - prev.attr_waste_s;
    let d_travel = cur.attr_travel_s - prev.attr_travel_s;
    let d_uncontrollable = cur.attr_uncontrollable_s - prev.attr_uncontrollable_s;

    let r = match cfg.mode {
        RewardMode::Sparse => cfg.w_throughput * d_completed - cfg.w_late * d_late,
        RewardMode::Dense => {
            cfg.w_throughput * d_completed
                - cfg.w_late * d_late
                - cfg.w_lateness * (d_lateness / 60.0)
                - cfg.w_pending * cur.pending as f64
        }
        RewardMode::Attribution => {
            cfg.w_throughput * d_completed
                - cfg.w_waste * (d_waste / 60.0)
                - cfg.w_travel * (d_travel / 60.0)
        }
        // Ablation: attribution that ALSO penalizes uncontrollable delay
        // (congestion + station queue). Used to show the controllability effect.
        RewardMode::AttributionFull => {
            cfg.w_throughput * d_completed
                - cfg.w_waste * ((d_waste + d_uncontrollable) / 60.0)
                - cfg.w_travel * (d_travel / 60.0)
        }
        // Routed: the global objective only; the per-decision controllable cost
        // is added in the policy (charged to the responsible decision) via
        // `decision_cost`, not here.
        RewardMode::Routed => cfg.w_throughput * d_completed - cfg.w_late * d_late,
    };

    r as f32
}
