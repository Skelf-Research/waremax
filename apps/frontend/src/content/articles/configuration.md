---
title: "Configuring Simulations"
description: "Deep dive into scenario configuration, presets, and policy selection"
pubDate: 2024-01-20
author: "Waremax Team"
tags: ["configuration", "policies", "advanced"]
---

# Configuring Simulations

Waremax simulations are driven by **scenario YAML files** that define the warehouse layout, robot fleet, and behavioral policies.

## Scenario Structure

A scenario file contains several top-level sections:

```yaml
map:
  grid_rows: 10
  grid_cols: 10
  aisle_width_m: 2.0

robots:
  count: 15
  max_speed_mps: 1.5
  battery_capacity_wh: 500

stations:
  pick_stations: 4
  drop_stations: 4
  charging_stations: 2

traffic:
  policy: continuous
  continuous:
    safety_distance_m: 1.0
    position_update_interval_s: 0.1
```

## Traffic Policies

Waremax ships with multiple traffic policies:

### Coarse Traffic Policy (Default)

Robots occupy entire edges. Simple capacity-based entry control. Fast but robots can appear to "jump" between nodes.

### Continuous Traffic Policy

Tracks robot position along edges with sub-meter precision:

- **Direction locking**: Prevents head-on collisions
- **Following distance**: Enforces minimum gap between robots
- **Position updates**: Periodic `RobotPositionUpdate` events for smooth visualization

Enable it by setting `traffic.policy: continuous`.

## Presets

Presets are pre-configured scenarios stored in `crates/waremax-config/presets/`. They let you quickly start common warehouse layouts without writing YAML.

## Custom Policies

Because Waremax is a library-first engine, you can implement custom policies by implementing the `EdgeTrafficPolicy` trait:

```rust
pub trait EdgeTrafficPolicy: Send + Sync {
    fn can_enter_edge(&self, traffic: &TrafficManager, edge: EdgeId, robot: RobotId, from: NodeId, to: NodeId) -> bool;
    fn on_enter_edge(&mut self, traffic: &mut TrafficManager, edge: EdgeId, robot: RobotId, from: NodeId, to: NodeId);
    fn on_leave_edge(&mut self, traffic: &mut TrafficManager, edge: EdgeId, robot: RobotId);
    fn on_position_update(&mut self, traffic: &mut TrafficManager, edge: EdgeId, robot: RobotId, progress: f64);
    fn tick(&mut self, traffic: &mut TrafficManager, current_time: SimTime);
}
```

Register your policy in `policy_factory.rs` to make it selectable from scenario files.
