# waremax-testing

**Testing, benchmarking, and experiment scaffolding for [WareMax](../../README.md): scenario presets, programmatic builders, parallel batch runs, parameter sweeps, A/B testing with Welch's t-test, and benchmarking with regression detection.**

Part of the [WareMax](../../README.md) workspace — a deterministic warehouse-robotics simulator and RL benchmark for Robotic Mobile Fulfillment Systems (RMFS).

## Role

Everything you need to run *experiments* on WareMax: build a scenario, build a world from it, run one or many, compare results statistically, detect performance regressions.

## Key types

| Item | Purpose |
|---|---|
| `ScenarioPreset` | Built-in scenarios: `Minimal`, `Quick`, `Standard`, `Baseline`, `HighLoad`, `PeakHours`, `StressTest`, `BatteryTest`, `MaintenanceTest`. |
| `ScenarioBuilder` | Programmatic, fluent `ScenarioConfig` construction. |
| `BatchRunner` | Parallel execution of many `(label, ScenarioConfig)` pairs via rayon; replications across seeds. |
| `SweepGenerator`, `SweepDimension` | Cartesian-product parameter exploration. |
| `ABTestRunner`, `ABTestConfig`, `welchs_t_test` | Statistical comparison of two configurations. |
| `BenchmarkSuite`, `BenchmarkHistory`, `RegressionAlert` | Performance tracking over time. |
| `ScenarioComparator`, `ComparisonReport`, `AggregatedStats` | Mean / stddev / CI aggregation across runs. |
| `runner::run_simulation_from_config`, `runner::build_world_from_config` | Library-level world builder + one-shot runner. |

## Example

```rust
use waremax_testing::{ScenarioPreset, BatchRunner};
let cfg = ScenarioPreset::Standard.config();
let results = BatchRunner::new(vec![("baseline".into(), cfg)])
    .run_with_replications(&[1, 2, 3, 4, 5]);
for r in results {
    println!("{}: throughput={:.1}/h", r.label, r.throughput());
}
```

## See also

- [`waremax-rl`](../waremax-rl/) — uses `build_world_from_config` on its worker thread.
- [WareMax README — RL Quickstart](../../README.md#reinforcement-learning-quickstart) — the Python `experiments.py` is the cross-language analog.
