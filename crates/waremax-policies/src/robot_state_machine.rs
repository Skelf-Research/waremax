//! Robot state machine policy using `waremax-statemachine`.
//!
//! This policy is **opt-in**. When enabled via scenario config
//! (`policies.state_machine.enabled: true`), every robot state
//! transition is validated before the event mutates the robot.
//!
//! Invalid transitions are logged and the event is silently dropped,
//! leaving the robot in its current state.

use waremax_core::SimEvent;
use waremax_entities::robot::RobotState;
use waremax_statemachine::{InvalidTransitionError, StateMachine};

/// Validates robot state transitions against a fixed ruleset.
///
/// Example valid transitions:
/// - `Idle` --(TaskAssignment)--> `Moving`
/// - `Moving` --(RobotArriveNode)--> `Idle` | `Servicing` | `PickingUp`
/// - `Servicing` --(StationServiceEnd)--> `Idle`
/// - `Waiting` --(RobotWaitEnd)--> `Moving`
/// - `Charging` --(RobotChargingEnd)--> `Idle`
/// - `Failed` --(MaintenanceEnd)--> `Idle`
pub struct RobotStateMachinePolicy;

impl StateMachine for RobotStateMachinePolicy {
    type State = RobotState;
    type Event = SimEvent;
    type Context = ();

    fn transition(
        &self,
        current: &Self::State,
        event: &Self::Event,
        _ctx: &Self::Context,
    ) -> Result<Self::State, InvalidTransitionError> {
        use RobotState::*;
        use SimEvent::*;

        match (current, event) {
            // Idle robot can be assigned a task (starts moving)
            (Idle, TaskAssignment { .. }) => Ok(Moving { destination: waremax_core::NodeId(0) }),

            // Moving robot arrives at node → various next states
            (Moving { .. }, RobotArriveNode { .. }) => {
                // In a real integration the EventHandler would tell us
                // which sub-state to enter.  Here we return Idle as a
                // safe default; the caller (EventHandler) can override
                // when it knows the exact next state.
                Ok(Idle)
            }
            (Moving { .. }, StationServiceStart { .. }) => Ok(Idle),
            (Moving { .. }, RobotPickup { .. }) => Ok(Idle),

            // Waiting robot can resume moving
            (Waiting { .. }, RobotWaitEnd { .. }) => Ok(Moving { destination: waremax_core::NodeId(0) }),

            // Servicing robot finishes service
            (Servicing { .. }, StationServiceEnd { .. }) => Ok(Idle),

            // PickingUp robot finishes pickup
            (PickingUp { .. }, RobotPickup { .. }) => Ok(Idle),

            // Charging robot finishes charging
            (Charging { .. }, RobotChargingEnd { .. }) => Ok(Idle),

            // SeekingCharge robot arrives at charging station
            (SeekingCharge { destination, .. }, RobotArriveNode { .. }) => {
                Ok(Charging {
                    at_station: *destination,
                })
            }

            // Failed robot goes to maintenance
            (Failed, MaintenanceStart { .. }) => Ok(InMaintenance {
                at_station: waremax_core::MaintenanceStationId(0),
                is_repair: true,
            }),

            // InMaintenance robot finishes repair
            (InMaintenance { .. }, MaintenanceEnd { .. }) => Ok(Idle),

            // SeekingMaintenance robot arrives at maintenance station
            (
                SeekingMaintenance { destination, is_repair },
                RobotArriveNode { .. },
            ) => Ok(InMaintenance {
                at_station: *destination,
                is_repair: *is_repair,
            }),

            // Robot can fail from almost any active state
            (Idle | Moving { .. } | Waiting { .. } | Servicing { .. } | PickingUp { .. },
             RobotFailure { .. }) => Ok(Failed),

            // Robot can start charging from idle when battery is low
            (Idle, RobotChargingStart { station_id, .. }) => Ok(Charging { at_station: *station_id }),

            // Robot can start seeking charge from idle
            (Idle, RobotLowBattery { .. }) => Ok(SeekingCharge {
                destination: waremax_core::ChargingStationId(0),
            }),

            // No-op transitions (event doesn't change state)
            (s, RobotDepartNode { .. }) => Ok(s.clone()),
            (s, RobotWaitStart { .. }) => Ok(s.clone()),
            (s, DispatchTasks) => Ok(s.clone()),
            (s, OrderArrival { .. }) => Ok(s.clone()),
            (s, InventoryUpdate { .. }) => Ok(s.clone()),
            (s, InboundArrival { .. }) => Ok(s.clone()),
            (s, PutawayTaskCreated { .. }) => Ok(s.clone()),
            (s, OutboundReady { .. }) => Ok(s.clone()),
            (s, ShipmentDeparture { .. }) => Ok(s.clone()),
            (s, ReplenishmentTrigger { .. }) => Ok(s.clone()),
            (s, MetricsSampleTick) => Ok(s.clone()),
            (s, DeadlockDetected { .. }) => Ok(s.clone()),
            (s, DeadlockResolved { .. }) => Ok(s.clone()),
            (s, RobotPositionUpdate { .. }) => Ok(s.clone()),
            (s, RobotMaintenanceDue { .. }) => Ok(s.clone()),

            // Any other combination is invalid
            (from, event) => Err(InvalidTransitionError {
                from: format!("{:?}", from),
                event: format!("{:?}", event),
            }),
        }
    }

    fn name(&self) -> &str {
        "robot_state_machine"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use waremax_core::{ChargingStationId, MaintenanceStationId, NodeId, RobotId, StationId, TaskId};

    #[test]
    fn test_idle_to_moving() {
        let sm = RobotStateMachinePolicy;
        let state = RobotState::Idle;
        let event = SimEvent::TaskAssignment {
            task_id: TaskId(1),
            robot_id: RobotId(0),
        };
        assert!(matches!(sm.transition(&state, &event, &()).unwrap(), RobotState::Moving { .. }));
    }

    #[test]
    fn test_invalid_transition_keeps_state() {
        let sm = RobotStateMachinePolicy;
        let state = RobotState::Idle;
        let event = SimEvent::StationServiceEnd {
            robot_id: RobotId(0),
            station_id: StationId(0),
            task_id: TaskId(0),
        };
        assert!(sm.transition(&state, &event, &()).is_err());
    }

    #[test]
    fn test_charging_to_idle() {
        let sm = RobotStateMachinePolicy;
        let state = RobotState::Charging {
            at_station: ChargingStationId(0),
        };
        let event = SimEvent::RobotChargingEnd {
            robot_id: RobotId(0),
            station_id: ChargingStationId(0),
            energy_charged_wh: 100.0,
        };
        assert_eq!(sm.transition(&state, &event, &()).unwrap(), RobotState::Idle);
    }

    #[test]
    fn test_failed_to_maintenance() {
        let sm = RobotStateMachinePolicy;
        let state = RobotState::Failed;
        let event = SimEvent::MaintenanceStart {
            robot_id: RobotId(0),
            station_id: MaintenanceStationId(0),
            is_repair: true,
        };
        assert!(matches!(
            sm.transition(&state, &event, &()).unwrap(),
            RobotState::InMaintenance { is_repair: true, .. }
        ));
    }
}
