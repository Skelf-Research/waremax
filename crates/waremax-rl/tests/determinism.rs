//! Phase A determinism gate.
//!
//! The benchmark's whole value is reproducibility, so we prove that driving the
//! simulation through the RL seam with a fixed seed and a fixed action sequence
//! yields byte-identical outcomes — within one env reused, and across two
//! freshly spawned worker threads (no global/thread state leaks in).

use waremax_config::ScenarioConfig;
use waremax_rl::{ActionMsg, RewardConfig, RewardMode, RlEnv};
use waremax_testing::ScenarioPreset;

/// A short scenario with enough activity to exercise many allocation decisions.
fn test_scenario() -> ScenarioConfig {
    scenario_from(ScenarioPreset::Quick, 10.0)
}

/// Build a config from a preset with a shortened (fast) horizon.
fn scenario_from(preset: ScenarioPreset, minutes: f64) -> ScenarioConfig {
    let mut cfg = preset.config();
    cfg.simulation.duration_minutes = minutes;
    cfg.simulation.warmup_minutes = 0.0;
    cfg
}

/// Drive an episode with the deterministic "always pick the first candidate"
/// agent. Row 0 is always a valid candidate because decisions are only emitted
/// when at least one candidate exists. Returns a fingerprint of the run.
fn drive(env: &mut RlEnv, seed: u64) -> Fingerprint {
    let _first_obs = env.reset(seed);
    let mut rewards = Vec::new();
    let mut steps: u64 = 0;
    loop {
        let res = env.step(ActionMsg::Choose(0));
        rewards.push(res.reward);
        steps += 1;
        if res.done {
            break;
        }
        assert!(steps < 1_000_000, "episode did not terminate");
    }
    let report = env.last_report().expect("a completed episode has a report");
    Fingerprint {
        steps,
        rewards,
        events_processed: report.events_processed,
        orders_completed: report.orders_completed,
        orders_late: report.orders_late,
        p95_cycle_time_s: report.p95_cycle_time_s,
    }
}

#[derive(PartialEq, Debug)]
struct Fingerprint {
    steps: u64,
    rewards: Vec<f32>,
    events_processed: u64,
    orders_completed: u32,
    orders_late: u32,
    p95_cycle_time_s: f64,
}

#[test]
fn same_seed_same_actions_is_deterministic_within_one_env() {
    let mut env = RlEnv::new(test_scenario(), RewardConfig::default());
    let a = drive(&mut env, 42);
    let b = drive(&mut env, 42); // reuse env: must drain + restart cleanly
    assert_eq!(a, b, "reusing an env with the same seed must reproduce exactly");
}

#[test]
fn same_seed_is_deterministic_across_fresh_envs() {
    let a = drive(&mut RlEnv::new(test_scenario(), RewardConfig::default()), 7);
    let b = drive(&mut RlEnv::new(test_scenario(), RewardConfig::default()), 7);
    assert_eq!(a, b, "two independent envs/worker threads must reproduce exactly");
}

#[test]
fn deterministic_across_presets() {
    // Larger maps, multiple stations, and charging stations expose other
    // potential HashMap-iteration nondeterminism beyond inventory placement.
    for preset in [
        ScenarioPreset::Quick,
        ScenarioPreset::Standard,
        ScenarioPreset::BatteryTest,
    ] {
        let cfg = scenario_from(preset, 8.0);
        let a = drive(&mut RlEnv::new(cfg.clone(), RewardConfig::default()), 99);
        let b = drive(&mut RlEnv::new(cfg, RewardConfig::default()), 99);
        assert_eq!(
            a, b,
            "preset {preset:?} must be deterministic across fresh envs"
        );
    }
}

#[test]
fn attribution_mode_is_deterministic() {
    // The attribution-shaped reward iterates the per-task delay-attribution
    // collector; verify it stays reproducible across fresh envs.
    let cfg = RewardConfig {
        mode: RewardMode::Attribution,
        ..RewardConfig::default()
    };
    let a = drive(&mut RlEnv::new(test_scenario(), cfg.clone()), 13);
    let b = drive(&mut RlEnv::new(test_scenario(), cfg), 13);
    assert_eq!(a, b, "attribution-mode reward must be deterministic");
    // The shaped reward should actually be exercised (non-zero somewhere),
    // otherwise the attribution wiring is silently inert.
    assert!(
        a.rewards.iter().any(|r| *r != 0.0),
        "attribution reward was all zeros — collector likely not populated"
    );
}

#[test]
fn routed_mode_is_deterministic() {
    // Per-decision routed credit charges each assignment's controllable cost to
    // the action that made it; verify reproducibility and that it is exercised.
    let cfg = RewardConfig {
        mode: RewardMode::Routed,
        ..RewardConfig::default()
    };
    let a = drive(&mut RlEnv::new(test_scenario(), cfg.clone()), 21);
    let b = drive(&mut RlEnv::new(test_scenario(), cfg), 21);
    assert_eq!(a, b, "routed-mode reward must be deterministic");
    assert!(
        a.rewards.iter().any(|r| *r != 0.0),
        "routed reward was all zeros — per-decision cost not applied"
    );
}

#[test]
fn episode_runs_and_terminates() {
    let mut env = RlEnv::new(test_scenario(), RewardConfig::default());
    let fp = drive(&mut env, 1);
    assert!(fp.steps > 0, "scenario should produce at least one allocation decision");
    assert!(env.is_done(), "env should be marked done after a full episode");
}

#[test]
fn reset_midway_does_not_hang() {
    // Start an episode, take a few steps, then reset early. The previous worker
    // must drain and join without deadlocking.
    let mut env = RlEnv::new(test_scenario(), RewardConfig::default());
    env.reset(123);
    for _ in 0..3 {
        if env.step(ActionMsg::Choose(0)).done {
            break;
        }
    }
    let fp = drive(&mut env, 123); // reset midway, then run a full episode
    assert!(fp.steps > 0);
}
