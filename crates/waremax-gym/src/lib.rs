//! Python (PyO3) bindings for the waremax RL environment.
//!
//! Exposes [`WaremaxEnv`], a thin handle over `waremax_rl::RlEnv`, plus a
//! `run_baseline` helper that runs a fixed heuristic on an identical scenario/seed
//! for apples-to-apples comparison. Observations are returned as plain Python
//! lists (the Python wrapper assembles numpy arrays), which keeps the Rust side
//! free of any numpy-ABI version coupling.
//!
//! Every blocking call into the simulation releases the GIL via `allow_threads`,
//! so the simulation worker thread is never starved.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use waremax_config::{DueTimeConfig, ScenarioConfig};
use waremax_rl::{
    ActionMsg, RewardConfig, RewardMode, RlEnv, StepInfo, MAX_ROBOTS, ROBOT_FEATS, TASK_FEATS,
};
use waremax_testing::presets::ScenarioPreset;
use waremax_testing::runner::run_simulation_from_config;

/// Resolve a preset name (case-insensitive) to a `ScenarioConfig`.
fn preset_config(name: &str) -> PyResult<ScenarioConfig> {
    let preset = match name.to_lowercase().as_str() {
        "minimal" => ScenarioPreset::Minimal,
        "quick" => ScenarioPreset::Quick,
        "standard" => ScenarioPreset::Standard,
        "baseline" => ScenarioPreset::Baseline,
        "highload" | "high_load" => ScenarioPreset::HighLoad,
        "peakhours" | "peak_hours" => ScenarioPreset::PeakHours,
        "stresstest" | "stress_test" => ScenarioPreset::StressTest,
        "batterytest" | "battery_test" => ScenarioPreset::BatteryTest,
        "maintenancetest" | "maintenance_test" => ScenarioPreset::MaintenanceTest,
        other => {
            return Err(PyValueError::new_err(format!("unknown preset '{other}'")));
        }
    };
    Ok(preset.config())
}

/// Build a scenario from either a YAML path or a preset name, with optional
/// horizon and due-time overrides. A tighter `due_time_minutes` creates SLA
/// pressure (the presets are otherwise SLA-saturated).
#[allow(clippy::too_many_arguments)]
fn build_scenario(
    preset: Option<String>,
    scenario_path: Option<String>,
    duration_minutes: Option<f64>,
    warmup_minutes: Option<f64>,
    due_time_minutes: Option<f64>,
    n_robots: Option<u32>,
    order_rate: Option<f64>,
    node_capacity: Option<u32>,
    edge_capacity: Option<u32>,
    congestion_weight: Option<f64>,
    smart_bins: Option<bool>,
    inventory_skus: Option<u32>,
) -> PyResult<ScenarioConfig> {
    let mut scenario = match scenario_path {
        Some(path) => ScenarioConfig::from_file(&path)
            .map_err(|e| PyValueError::new_err(format!("failed to load scenario: {e}")))?,
        None => preset_config(&preset.unwrap_or_else(|| "quick".to_string()))?,
    };
    if let Some(d) = duration_minutes {
        scenario.simulation.duration_minutes = d;
    }
    if let Some(w) = warmup_minutes {
        scenario.simulation.warmup_minutes = w;
    }
    if let Some(m) = due_time_minutes {
        scenario.orders.due_times = Some(DueTimeConfig {
            due_type: "fixed".to_string(),
            minutes: m,
        });
    }
    // Load knobs: shrink the fleet and/or raise the order rate to create
    // queueing/contention-dominated regimes where myopic heuristics struggle.
    if let Some(n) = n_robots {
        scenario.robots.count = n;
    }
    if let Some(r) = order_rate {
        scenario.orders.arrival_process.rate_per_min = r;
    }
    // Traffic capacity: tightening node/edge capacity induces congestion, where
    // *which* robot is dispatched (and the contention it creates) matters.
    if let Some(c) = node_capacity {
        scenario.traffic.node_capacity_default = c;
    }
    if let Some(c) = edge_capacity {
        scenario.traffic.edge_capacity_default = c;
    }
    if let Some(w) = congestion_weight {
        scenario.traffic.congestion_weight = w;
    }
    if let Some(b) = smart_bins {
        scenario.policies.smart_bins = b;
    }
    if let Some(n) = inventory_skus {
        scenario.policies.inventory_skus = Some(n);
    }
    Ok(scenario)
}

/// A Gym-style environment over one waremax scenario (task-allocation control).
#[pyclass]
struct WaremaxEnv {
    env: RlEnv,
}

#[pymethods]
impl WaremaxEnv {
    #[new]
    #[pyo3(signature = (preset=None, scenario_path=None, duration_minutes=None, warmup_minutes=None, reward_mode=None, due_time_minutes=None, n_robots=None, order_rate=None, node_capacity=None, edge_capacity=None, congestion_weight=None, smart_bins=None, inventory_skus=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        preset: Option<String>,
        scenario_path: Option<String>,
        duration_minutes: Option<f64>,
        warmup_minutes: Option<f64>,
        reward_mode: Option<String>,
        due_time_minutes: Option<f64>,
        n_robots: Option<u32>,
        order_rate: Option<f64>,
        node_capacity: Option<u32>,
        edge_capacity: Option<u32>,
        congestion_weight: Option<f64>,
        smart_bins: Option<bool>,
        inventory_skus: Option<u32>,
    ) -> PyResult<Self> {
        let scenario = build_scenario(
            preset,
            scenario_path,
            duration_minutes,
            warmup_minutes,
            due_time_minutes,
            n_robots,
            order_rate,
            node_capacity,
            edge_capacity,
            congestion_weight,
            smart_bins,
            inventory_skus,
        )?;
        let mode = match reward_mode.as_deref() {
            None => RewardMode::Dense,
            Some(s) => RewardMode::from_str(s)
                .ok_or_else(|| PyValueError::new_err(format!("unknown reward mode '{s}'")))?,
        };
        let reward_cfg = RewardConfig {
            mode,
            ..RewardConfig::default()
        };
        Ok(Self {
            env: RlEnv::new(scenario, reward_cfg),
        })
    }

    #[getter]
    fn max_robots(&self) -> usize {
        MAX_ROBOTS
    }
    #[getter]
    fn robot_feats(&self) -> usize {
        ROBOT_FEATS
    }
    #[getter]
    fn task_feats(&self) -> usize {
        TASK_FEATS
    }

    /// Begin an episode. Returns (robot_feats, action_mask, task_feats) for the
    /// first decision; the Python wrapper turns these into the observation dict.
    fn reset(&mut self, py: Python<'_>, seed: u64) -> (Vec<f32>, Vec<bool>, Vec<f32>) {
        let obs = py.allow_threads(|| self.env.reset(seed));
        (obs.robot_feats, obs.action_mask, obs.task_feats)
    }

    /// Apply an action (index into the masked candidate list). Returns
    /// (robot_feats, action_mask, task_feats, reward, done, info).
    fn step(
        &mut self,
        py: Python<'_>,
        action: usize,
    ) -> PyResult<(Vec<f32>, Vec<bool>, Vec<f32>, f32, bool, Py<PyDict>)> {
        let res = py.allow_threads(|| self.env.step(ActionMsg::Choose(action)));
        let info = step_info_to_dict(py, &res.info)?;
        Ok((
            res.obs.robot_feats,
            res.obs.action_mask,
            res.obs.task_feats,
            res.reward,
            res.done,
            info,
        ))
    }

    /// JSON of the most recent completed episode's full simulation report.
    fn last_report_json(&self) -> Option<String> {
        self.env
            .last_report()
            .and_then(|r| serde_json::to_string(&r).ok())
    }
}

fn step_info_to_dict(py: Python<'_>, info: &StepInfo) -> PyResult<Py<PyDict>> {
    let d = PyDict::new_bound(py);
    d.set_item("completed_delta", info.completed_delta)?;
    d.set_item("late_delta", info.late_delta)?;
    d.set_item("lateness_delta_s", info.lateness_delta_s)?;
    d.set_item("pending", info.pending)?;
    d.set_item("sim_time_s", info.sim_time_s)?;
    d.set_item("errored", info.errored)?;
    if let Some(fm) = &info.final_metrics {
        d.set_item("orders_completed", fm.orders_completed)?;
        d.set_item("orders_late", fm.orders_late)?;
        d.set_item("avg_cycle_time_s", fm.avg_cycle_time_s)?;
        d.set_item("p95_cycle_time_s", fm.p95_cycle_time_s)?;
        d.set_item("p95_lateness_s", fm.p95_lateness_s)?;
        d.set_item("throughput_per_hour", fm.throughput_per_hour)?;
        d.set_item("on_time_rate", fm.on_time_rate)?;
        d.set_item("robot_utilization", fm.robot_utilization)?;
    }
    Ok(d.unbind())
}

/// Run a fixed heuristic dispatcher on the given scenario/seed and return the
/// full report as JSON. Used by the baseline-comparison harness so heuristics
/// and the RL agent are evaluated on identical conditions.
#[pyfunction]
#[pyo3(signature = (alloc_type, preset=None, scenario_path=None, seed=0, duration_minutes=None, warmup_minutes=None, due_time_minutes=None, n_robots=None, order_rate=None, node_capacity=None, edge_capacity=None, congestion_weight=None, smart_bins=None, inventory_skus=None))]
#[allow(clippy::too_many_arguments)]
fn run_baseline(
    py: Python<'_>,
    alloc_type: String,
    preset: Option<String>,
    scenario_path: Option<String>,
    seed: u64,
    duration_minutes: Option<f64>,
    warmup_minutes: Option<f64>,
    due_time_minutes: Option<f64>,
    n_robots: Option<u32>,
    order_rate: Option<f64>,
    node_capacity: Option<u32>,
    edge_capacity: Option<u32>,
    congestion_weight: Option<f64>,
    smart_bins: Option<bool>,
    inventory_skus: Option<u32>,
) -> PyResult<String> {
    let mut scenario = build_scenario(
        preset,
        scenario_path,
        duration_minutes,
        warmup_minutes,
        due_time_minutes,
        n_robots,
        order_rate,
        node_capacity,
        edge_capacity,
        congestion_weight,
        smart_bins,
        inventory_skus,
    )?;
    scenario.seed = seed;
    scenario.policies.task_allocation.alloc_type = alloc_type;
    let report = py.allow_threads(|| run_simulation_from_config(&scenario));
    serde_json::to_string(&report).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pymodule]
fn waremax_gym(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<WaremaxEnv>()?;
    m.add_function(wrap_pyfunction!(run_baseline, m)?)?;
    m.add("MAX_ROBOTS", MAX_ROBOTS)?;
    m.add("ROBOT_FEATS", ROBOT_FEATS)?;
    m.add("TASK_FEATS", TASK_FEATS)?;
    Ok(())
}
