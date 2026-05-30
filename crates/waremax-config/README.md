# waremax-config

**YAML/JSON scenario parsing, schema validation, and the canonical configuration model for [WareMax](../../README.md).**

Part of the [WareMax](../../README.md) workspace — a deterministic warehouse-robotics simulator and RL benchmark for Robotic Mobile Fulfillment Systems (RMFS).

## Role

A scenario is a YAML (or JSON) file fully describing a deterministic simulation: seed, horizon, fleet, stations, orders, policies, traffic, inventory, and metrics. This crate parses, validates, and exposes those structures via `ScenarioConfig`.

## Key types

| Item | Purpose |
|---|---|
| `ScenarioConfig` | Top-level scenario record; `from_file(path)`, `from_yaml`, `from_json`. |
| `SimulationParams` | `duration_minutes`, `warmup_minutes`. |
| `RobotConfig`, `StationConfig`, `OrderConfig`, `ArrivalProcess`, `LinesConfig`, `SkuPopularity`, `DueTimeConfig` | Demand and fleet description. |
| `PolicyConfig` | Selects allocation / station-assignment / batching / priority policies; `smart_bins`, `inventory_skus`. |
| `TrafficConfig` | Edge/node capacities, traffic policy, `congestion_weight` for congestion-aware routing. |
| `ChargingStationConfig`, `MaintenanceStationConfig`, `MetricsConfig` | v1/v3 extensions. |

## Example

```yaml
seed: 12345
simulation:
  duration_minutes: 60
  warmup_minutes: 5
robots:
  count: 10
  max_speed_mps: 1.5
policies:
  task_allocation: { type: routed }
  smart_bins: true
traffic:
  congestion_weight: 0.0     # > 0 enables occupancy-weighted routing
```

See [main README — Configuration](../../README.md#configuration) for the full reference.

## See also

- [`waremax-testing::ScenarioPreset`](../waremax-testing/) — built-in scenario presets.
- [`waremax-sim`](../waremax-sim/) — consumes `ScenarioConfig` to build a `World`.
