//! `RlEnv`: drives a simulation on a worker thread and exposes a Gym-style
//! `reset` / `step` interface in pure Rust.
//!
//! The simulation runs to completion on its own thread; the `RlPolicy` inside it
//! blocks at each allocation decision waiting for an action from this struct.
//! A strict ping-pong over two bounded(1) channels means exactly one side runs
//! at a time, so the run is deterministic given the seed and the action sequence
//! — there is no concurrent access to simulation state.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crossbeam_channel::{bounded, Receiver, Sender};
use waremax_config::ScenarioConfig;
use waremax_metrics::SimulationReport;
use waremax_sim::SimulationRunner;
use waremax_testing::runner::build_world_from_config;

use crate::observation::Observation;
use crate::policy::RlPolicy;
use crate::protocol::{ActionMsg, FinalMetrics, Message, StepInfo};
use crate::reward::{self, RewardConfig, RewardSnapshot};

/// Result of a single `step`.
#[derive(Debug, Clone)]
pub struct StepResult {
    pub obs: Observation,
    pub reward: f32,
    pub done: bool,
    pub info: StepInfo,
}

/// A Gym-style environment over one waremax scenario.
///
/// Construct with [`RlEnv::new`], then call [`RlEnv::reset`] to begin an episode
/// (returns the first observation) and [`RlEnv::step`] repeatedly until
/// `StepResult::done` is true.
pub struct RlEnv {
    scenario: ScenarioConfig,
    reward_cfg: RewardConfig,

    // Live episode handles (None between episodes).
    obs_rx: Option<Receiver<Message>>,
    action_tx: Option<Sender<ActionMsg>>,
    worker: Option<JoinHandle<()>>,
    shared_report: Arc<Mutex<Option<SimulationReport>>>,
    done: bool,
}

impl RlEnv {
    pub fn new(scenario: ScenarioConfig, reward_cfg: RewardConfig) -> Self {
        Self {
            scenario,
            reward_cfg,
            obs_rx: None,
            action_tx: None,
            worker: None,
            shared_report: Arc::new(Mutex::new(None)),
            done: false,
        }
    }

    /// Whether the current episode has ended.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// The report of the most recently completed episode, if any.
    pub fn last_report(&self) -> Option<SimulationReport> {
        self.shared_report.lock().unwrap().clone()
    }

    /// Start a new episode with the given seed and return the first observation.
    pub fn reset(&mut self, seed: u64) -> Observation {
        // Drain and join any previous episode.
        self.abort_and_join();

        let (obs_tx, obs_rx) = bounded::<Message>(1);
        let (action_tx, action_rx) = bounded::<ActionMsg>(1);

        let mut scenario = self.scenario.clone();
        scenario.seed = seed;
        let duration = scenario.simulation.duration_minutes;
        let warmup = scenario.simulation.warmup_minutes;

        let reward_cfg = self.reward_cfg.clone();
        let snapshot: Arc<Mutex<RewardSnapshot>> = Arc::new(Mutex::new(RewardSnapshot::default()));
        let snapshot_for_terminal = snapshot.clone();

        // Reset the shared report slot for this episode.
        let report_slot = self.shared_report.clone();
        *report_slot.lock().unwrap() = None;

        let obs_tx_terminal = obs_tx.clone();

        let worker = thread::spawn(move || {
            let policy = RlPolicy::new(obs_tx, action_rx, reward_cfg.clone(), snapshot);
            let mut world = build_world_from_config(&scenario);
            world.policies.task_allocation = Box::new(policy);
            // Attribution-shaped reward needs the per-task delay attribution
            // collector running.
            if reward_cfg.mode.needs_attribution() {
                world.attribution_collector.enable();
            }
            let mut runner = SimulationRunner::new(world, duration, warmup);

            // Run to completion, then compute the final reward + metrics. The
            // whole thing is unwind-guarded so a simulator panic becomes a clean
            // terminal rather than a poisoned/hung channel.
            let result = catch_unwind(AssertUnwindSafe(|| {
                let report = runner.run();
                let w = runner.world();
                let attribution = if w.attribution_collector.is_enabled() {
                    Some(&w.attribution_collector)
                } else {
                    None
                };
                let final_snap = reward::snapshot_from(&w.orders, &w.tasks, attribution);
                (report, final_snap)
            }));

            let terminal = match result {
                Ok((report, final_snap)) => {
                    let prev = snapshot_for_terminal.lock().unwrap().clone();
                    let reward = reward::delta(&prev, &final_snap, &reward_cfg);
                    let info = StepInfo {
                        completed_delta: final_snap.completed as i64 - prev.completed as i64,
                        late_delta: final_snap.late as i64 - prev.late as i64,
                        lateness_delta_s: final_snap.cum_lateness_s - prev.cum_lateness_s,
                        pending: final_snap.pending,
                        sim_time_s: report.duration_s,
                        final_metrics: Some(final_metrics(&report)),
                        errored: false,
                    };
                    *report_slot.lock().unwrap() = Some(report);
                    (reward, info)
                }
                Err(_) => {
                    let info = StepInfo {
                        errored: true,
                        ..Default::default()
                    };
                    (0.0, info)
                }
            };

            let _ = obs_tx_terminal.send(Message::Terminal {
                obs: Observation::zeros(),
                reward: terminal.0,
                info: terminal.1,
            });
        });

        self.obs_rx = Some(obs_rx);
        self.action_tx = Some(action_tx);
        self.worker = Some(worker);
        self.done = false;

        // Receive the first message (the first decision, or a terminal if the
        // episode produced no allocation decisions at all).
        match self.obs_rx.as_ref().unwrap().recv() {
            Ok(Message::Decision { obs, .. }) => obs,
            Ok(Message::Terminal { obs, .. }) => {
                self.done = true;
                self.join_worker();
                obs
            }
            Err(_) => {
                self.done = true;
                Observation::zeros()
            }
        }
    }

    /// Apply an action and advance to the next decision (or episode end).
    pub fn step(&mut self, action: ActionMsg) -> StepResult {
        if self.done {
            return StepResult {
                obs: Observation::zeros(),
                reward: 0.0,
                done: true,
                info: StepInfo::default(),
            };
        }

        let action_tx = self.action_tx.as_ref().expect("step called before reset");
        if action_tx.send(action).is_err() {
            self.done = true;
            return StepResult {
                obs: Observation::zeros(),
                reward: 0.0,
                done: true,
                info: StepInfo::default(),
            };
        }

        match self.obs_rx.as_ref().unwrap().recv() {
            Ok(Message::Decision { obs, reward, info }) => StepResult {
                obs,
                reward,
                done: false,
                info,
            },
            Ok(Message::Terminal { obs, reward, info }) => {
                self.done = true;
                self.join_worker();
                StepResult {
                    obs,
                    reward,
                    done: true,
                    info,
                }
            }
            Err(_) => {
                self.done = true;
                StepResult {
                    obs: Observation::zeros(),
                    reward: 0.0,
                    done: true,
                    info: StepInfo::default(),
                }
            }
        }
    }

    fn join_worker(&mut self) {
        if let Some(h) = self.worker.take() {
            let _ = h.join();
        }
    }

    /// Drain an in-flight episode to completion (replying Abort to every pending
    /// decision) and join the worker, so no thread is leaked across episodes.
    fn abort_and_join(&mut self) {
        if let (Some(tx), Some(rx)) = (self.action_tx.take(), self.obs_rx.take()) {
            loop {
                if tx.send(ActionMsg::Abort).is_err() {
                    break;
                }
                match rx.recv() {
                    Ok(Message::Decision { .. }) => continue,
                    Ok(Message::Terminal { .. }) | Err(_) => break,
                }
            }
        }
        self.join_worker();
    }
}

impl Drop for RlEnv {
    fn drop(&mut self) {
        self.abort_and_join();
    }
}

fn final_metrics(report: &SimulationReport) -> FinalMetrics {
    // Prefer SLA-based on-time rate / lateness when available, matching the
    // `metrics_from_report` path on the Python side so the comparison table is
    // apples-to-apples. Fall back to completed-vs-late when no SLA data.
    let (on_time_rate, p95_lateness_s) = match &report.sla {
        Some(sla) => (1.0 - sla.sla_miss_rate, sla.p95_lateness_s),
        None => {
            let rate = if report.orders_completed > 0 {
                (report.orders_completed.saturating_sub(report.orders_late)) as f64
                    / report.orders_completed as f64
            } else {
                0.0
            };
            (rate, 0.0)
        }
    };
    FinalMetrics {
        orders_completed: report.orders_completed,
        orders_late: report.orders_late,
        avg_cycle_time_s: report.avg_cycle_time_s,
        p95_cycle_time_s: report.p95_cycle_time_s,
        p95_lateness_s,
        throughput_per_hour: report.throughput_per_hour,
        on_time_rate,
        robot_utilization: report.robot_utilization,
    }
}
