# Custom Entities

Create new simulation entity types.

---

## Overview

Entities are the actors in the simulation:

- Robots (mobile agents)
- Stations (service points)
- Tasks (work items)
- Orders (task groups)

You can extend existing entities or create new ones.

---

## Entity Design

### Core Pattern

Entities have:

1. **Identity**: Unique ID
2. **State**: Current condition
3. **Behavior**: How they respond to events

```rust
pub struct MyEntity {
    id: MyEntityId,
    state: MyEntityState,
    // ... other fields
}

impl MyEntity {
    pub fn handle_event(&mut self, event: &MyEvent) {
        // Update state based on event
    }
}
```

---

## Example: Forklift Entity

A specialized robot type with different capabilities.

### Step 1: Define the Entity

```rust
// crates/waremax-entities/src/forklift.rs

use crate::{EntityId, Position, NodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ForkliftId(pub u32);

#[derive(Debug, Clone)]
pub enum ForkliftState {
    Idle,
    Traveling { destination: NodeId },
    Loading { rack: RackId },
    Unloading { rack: RackId },
    Carrying { rack: RackId },
}

pub struct Forklift {
    id: ForkliftId,
    position: NodeId,
    state: ForkliftState,
    speed: f64,
    lift_capacity: u32,  // Max weight
    lift_time: f64,      // Time to lift/lower
}

impl Forklift {
    pub fn new(id: ForkliftId, position: NodeId, config: &ForkliftConfig) -> Self {
        Self {
            id,
            position,
            state: ForkliftState::Idle,
            speed: config.speed_m_s,
            lift_capacity: config.lift_capacity_kg,
            lift_time: config.lift_time_s,
        }
    }

    pub fn id(&self) -> ForkliftId {
        self.id
    }

    pub fn position(&self) -> NodeId {
        self.position
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.state, ForkliftState::Idle)
    }

    pub fn is_carrying(&self) -> bool {
        matches!(self.state, ForkliftState::Carrying { .. })
    }

    pub fn start_traveling(&mut self, destination: NodeId) {
        self.state = ForkliftState::Traveling { destination };
    }

    pub fn arrive(&mut self, node: NodeId) {
        self.position = node;
        self.state = ForkliftState::Idle;
    }

    pub fn start_loading(&mut self, rack: RackId) {
        self.state = ForkliftState::Loading { rack };
    }

    pub fn complete_loading(&mut self, rack: RackId) {
        self.state = ForkliftState::Carrying { rack };
    }

    pub fn start_unloading(&mut self) {
        if let ForkliftState::Carrying { rack } = self.state {
            self.state = ForkliftState::Unloading { rack };
        }
    }

    pub fn complete_unloading(&mut self) {
        self.state = ForkliftState::Idle;
    }
}
```

### Step 2: Define Events

```rust
// crates/waremax-core/src/events/forklift.rs

#[derive(Debug, Clone)]
pub enum ForkliftEvent {
    AssignRackMove {
        forklift: ForkliftId,
        rack: RackId,
        destination: NodeId,
    },
    StartTravel {
        forklift: ForkliftId,
        path: Path,
    },
    ArriveAtNode {
        forklift: ForkliftId,
        node: NodeId,
    },
    StartLoading {
        forklift: ForkliftId,
        rack: RackId,
    },
    CompleteLoading {
        forklift: ForkliftId,
        rack: RackId,
    },
    StartUnloading {
        forklift: ForkliftId,
    },
    CompleteUnloading {
        forklift: ForkliftId,
    },
}
```

### Step 3: Implement Event Handling

```rust
// crates/waremax-sim/src/handlers/forklift.rs

impl Simulation {
    pub fn handle_forklift_event(
        &mut self,
        event: ForkliftEvent,
    ) {
        match event {
            ForkliftEvent::StartLoading { forklift, rack } => {
                let fl = self.forklift_mut(forklift);
                fl.start_loading(rack);

                // Schedule completion
                self.schedule(
                    self.time + fl.lift_time,
                    ForkliftEvent::CompleteLoading { forklift, rack },
                );
            }

            ForkliftEvent::CompleteLoading { forklift, rack } => {
                let fl = self.forklift_mut(forklift);
                fl.complete_loading(rack);

                // Record metrics
                self.metrics.record_rack_lifted(forklift, rack);

                // Continue with travel to destination
                // ...
            }

            // ... other events
        }
    }
}
```

### Step 4: Add Configuration

```rust
// crates/waremax-config/src/forklift.rs

#[derive(Debug, Deserialize)]
pub struct ForkliftConfig {
    pub count: u32,
    pub speed_m_s: f64,
    pub lift_capacity_kg: u32,
    pub lift_time_s: f64,
}

impl Default for ForkliftConfig {
    fn default() -> Self {
        Self {
            count: 0,
            speed_m_s: 1.0,
            lift_capacity_kg: 1000,
            lift_time_s: 5.0,
        }
    }
}
```

```yaml
# scenario.yaml
forklifts:
  count: 3
  speed_m_s: 1.0
  lift_capacity_kg: 1500
  lift_time_s: 4.0
```

---

## Extending Existing Entities

### Adding Fields to Robot

```rust
// Extend Robot with custom fields
pub struct ExtendedRobot {
    base: Robot,
    custom_field: CustomType,
}

impl Deref for ExtendedRobot {
    type Target = Robot;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
```

### Adding Behavior

```rust
// Add method via extension trait
pub trait RobotExtensions {
    fn custom_behavior(&mut self, param: &Param);
}

impl RobotExtensions for Robot {
    fn custom_behavior(&mut self, param: &Param) {
        // Custom logic
    }
}
```

---

## Entity Lifecycle

### Creation

```rust
// In simulation initialization
fn create_entities(&mut self, config: &Scenario) {
    // Create robots
    for i in 0..config.robots.count {
        let robot = Robot::new(RobotId(i), &config.robots);
        self.robots.push(robot);
    }

    // Create forklifts (custom entity)
    for i in 0..config.forklifts.count {
        let forklift = Forklift::new(ForkliftId(i), &config.forklifts);
        self.forklifts.push(forklift);
    }
}
```

### State Updates

```rust
// Entities update through event handlers
fn handle_event(&mut self, event: Event) {
    match event {
        Event::Robot(e) => self.handle_robot_event(e),
        Event::Forklift(e) => self.handle_forklift_event(e),
        // ...
    }
}
```

### Destruction

Entities typically live for the simulation duration. For dynamic creation/removal:

```rust
pub fn remove_robot(&mut self, id: RobotId) {
    self.robots.retain(|r| r.id() != id);
    self.metrics.record_robot_removed(id);
}
```

---

## Metrics for Custom Entities

```rust
// crates/waremax-metrics/src/forklift.rs

#[derive(Default)]
pub struct ForkliftMetrics {
    pub racks_moved: u64,
    pub total_lift_time: f64,
    pub total_travel_time: f64,
    pub utilization_samples: Vec<f64>,
}

impl ForkliftMetrics {
    pub fn record_rack_lifted(&mut self, _id: ForkliftId) {
        self.racks_moved += 1;
    }

    pub fn record_lift_time(&mut self, duration: f64) {
        self.total_lift_time += duration;
    }

    pub fn utilization(&self) -> f64 {
        // Calculate from samples
        self.utilization_samples.iter().sum::<f64>()
            / self.utilization_samples.len() as f64
    }
}
```

---

## Testing Entities

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forklift_state_transitions() {
        let mut forklift = Forklift::new(
            ForkliftId(1),
            NodeId(0),
            &ForkliftConfig::default(),
        );

        assert!(forklift.is_idle());

        forklift.start_loading(RackId(1));
        assert!(matches!(forklift.state, ForkliftState::Loading { .. }));

        forklift.complete_loading(RackId(1));
        assert!(forklift.is_carrying());

        forklift.start_unloading();
        assert!(matches!(forklift.state, ForkliftState::Unloading { .. }));

        forklift.complete_unloading();
        assert!(forklift.is_idle());
    }
}
```

---

## Related

- [Architecture Overview](../architecture/overview.md)
- [Entities API](../api/entities.md)
- [Events](../../concepts/simulation/events.md)
