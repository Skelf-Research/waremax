# Custom Policies

Implement your own decision-making policies.

---

## Overview

Policies define how decisions are made in the simulation. You can create custom policies for:

- Task allocation
- Station assignment
- Routing
- Charging

---

## Policy Traits

### TaskAllocationPolicy

Decides which robot handles a task.

```rust
pub trait TaskAllocationPolicy: Send + Sync {
    /// Allocate a task to a robot
    fn allocate(
        &self,
        task: &Task,
        robots: &[Robot],
        map: &WarehouseMap,
    ) -> Option<RobotId>;

    /// Name for logging/metrics
    fn name(&self) -> &str;
}
```

### StationAssignmentPolicy

Decides which station serves a task.

```rust
pub trait StationAssignmentPolicy: Send + Sync {
    /// Assign a station for a task
    fn assign(
        &self,
        task: &Task,
        stations: &[Station],
        robot_position: NodeId,
        map: &WarehouseMap,
    ) -> Option<StationId>;

    fn name(&self) -> &str;
}
```

### RoutingPolicy

Decides which path a robot takes.

```rust
pub trait RoutingPolicy: Send + Sync {
    /// Find path from source to destination
    fn find_path(
        &self,
        from: NodeId,
        to: NodeId,
        map: &WarehouseMap,
        traffic: &TrafficState,
    ) -> Option<Path>;

    fn name(&self) -> &str;
}
```

---

## Example: Custom Task Allocation

### Step 1: Define the Policy

```rust
// crates/waremax-policies/src/custom.rs

use waremax_entities::{Robot, Task, RobotId};
use waremax_map::WarehouseMap;
use crate::TaskAllocationPolicy;

/// Allocate to robot with highest battery
pub struct HighestBatteryPolicy;

impl TaskAllocationPolicy for HighestBatteryPolicy {
    fn allocate(
        &self,
        task: &Task,
        robots: &[Robot],
        _map: &WarehouseMap,
    ) -> Option<RobotId> {
        robots
            .iter()
            .filter(|r| r.is_idle())
            .max_by(|a, b| {
                a.battery_pct()
                    .partial_cmp(&b.battery_pct())
                    .unwrap()
            })
            .map(|r| r.id())
    }

    fn name(&self) -> &str {
        "highest_battery"
    }
}
```

### Step 2: Register the Policy

```rust
// crates/waremax-policies/src/registry.rs

pub fn create_task_allocation_policy(
    name: &str,
    config: &PolicyConfig,
) -> Box<dyn TaskAllocationPolicy> {
    match name {
        "nearest_idle" => Box::new(NearestIdlePolicy::new(config)),
        "least_busy" => Box::new(LeastBusyPolicy::new(config)),
        "highest_battery" => Box::new(HighestBatteryPolicy),
        _ => panic!("Unknown policy: {}", name),
    }
}
```

### Step 3: Add Configuration Support

```rust
// crates/waremax-config/src/policies.rs

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum TaskAllocationConfig {
    NearestIdle,
    LeastBusy,
    RoundRobin,
    HighestBattery,  // Add new variant
}
```

---

## Example: Custom Routing

### Step 1: Define the Policy

```rust
// Custom routing that avoids certain node types

pub struct AvoidChargingRoutingPolicy {
    fallback: ShortestPathPolicy,
}

impl RoutingPolicy for AvoidChargingRoutingPolicy {
    fn find_path(
        &self,
        from: NodeId,
        to: NodeId,
        map: &WarehouseMap,
        traffic: &TrafficState,
    ) -> Option<Path> {
        // Try to find path avoiding charging nodes
        if let Some(path) = self.find_avoiding_path(from, to, map) {
            return Some(path);
        }

        // Fallback to standard routing
        self.fallback.find_path(from, to, map, traffic)
    }

    fn name(&self) -> &str {
        "avoid_charging"
    }
}

impl AvoidChargingRoutingPolicy {
    fn find_avoiding_path(
        &self,
        from: NodeId,
        to: NodeId,
        map: &WarehouseMap,
    ) -> Option<Path> {
        // Custom pathfinding logic
        let graph = map.graph();

        // Add high cost to charging nodes
        let edge_cost = |edge: EdgeId| {
            let target = graph.edge_target(edge);
            if map.node(target).is_charging() {
                1000.0  // High cost to avoid
            } else {
                map.edge_length(edge)
            }
        };

        dijkstra_with_cost(graph, from, to, edge_cost)
    }
}
```

---

## Testing Policies

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highest_battery_allocation() {
        let policy = HighestBatteryPolicy;

        let robots = vec![
            Robot::new(RobotId(1), battery_pct: 50.0),
            Robot::new(RobotId(2), battery_pct: 80.0),
            Robot::new(RobotId(3), battery_pct: 30.0),
        ];

        let task = Task::new(TaskId(1), /* ... */);
        let map = WarehouseMap::simple_grid(3, 3);

        let result = policy.allocate(&task, &robots, &map);

        assert_eq!(result, Some(RobotId(2)));  // Highest battery
    }
}
```

### Integration Tests

```rust
#[test]
fn test_custom_policy_in_simulation() {
    let scenario = ScenarioBuilder::new()
        .robots(5)
        .policy(TaskAllocationConfig::HighestBattery)
        .build();

    let result = run_simulation(scenario);

    assert!(result.tasks_completed > 0);
}
```

---

## Policy Configuration

### Adding Parameters

```rust
pub struct WeightedAllocationPolicy {
    distance_weight: f64,
    battery_weight: f64,
    queue_weight: f64,
}

impl WeightedAllocationPolicy {
    pub fn from_config(config: &WeightedConfig) -> Self {
        Self {
            distance_weight: config.distance_weight.unwrap_or(1.0),
            battery_weight: config.battery_weight.unwrap_or(0.5),
            queue_weight: config.queue_weight.unwrap_or(0.3),
        }
    }
}
```

### YAML Configuration

```yaml
policies:
  task_allocation:
    type: weighted
    distance_weight: 1.0
    battery_weight: 0.8
    queue_weight: 0.5
```

---

## Best Practices

### Performance

- Keep `allocate()` and `find_path()` fast
- Avoid expensive computations per call
- Cache when appropriate

### Determinism

- Same inputs must produce same outputs
- Use provided RNG if randomness needed
- No external state dependencies

### Error Handling

- Return `None` rather than panic
- Log warnings for unusual situations
- Handle edge cases gracefully

---

## Related

- [Policy Concepts](../../concepts/policies/index.md)
- [Policy Configuration](../../configuration/policies.md)
- [API: waremax-policies](../api/policies.md)
