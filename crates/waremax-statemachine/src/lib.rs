//! Generic state machine primitives for Waremax simulation.
//!
//! Provides a trait-based state machine framework that can be used
//! for robot state validation, session lifecycle management, and
//! other stateful components.

use std::fmt::Debug;
use thiserror::Error;

/// Error returned when an invalid state transition is attempted.
#[derive(Clone, Debug, Error)]
#[error("Invalid transition from {from} on event {event}")]
pub struct InvalidTransitionError {
    pub from: String,
    pub event: String,
}

/// A generic state machine that validates transitions.
///
/// Implement this trait for a specific domain (robot states, session states, etc.)
/// and wrap it in [`Enforced`] to get runtime transition validation.
pub trait StateMachine {
    /// The state type (must be cloneable and comparable for rollback).
    type State: Clone + PartialEq + Debug + Send + Sync;
    /// The event type that drives transitions.
    type Event: Clone + Debug + Send + Sync;
    /// Additional context available during transition decisions.
    type Context: Send + Sync;

    /// Attempt a transition from `current` state given an `event` and `context`.
    ///
    /// Returns the new state on success, or [`InvalidTransitionError`] if the
    /// transition is not allowed.
    fn transition(
        &self,
        current: &Self::State,
        event: &Self::Event,
        ctx: &Self::Context,
    ) -> Result<Self::State, InvalidTransitionError>;

    /// Human-readable name of this state machine.
    fn name(&self) -> &str {
        "state_machine"
    }
}

/// A wrapper that enforces state machine transitions at runtime.
///
/// Holds the current state and only mutates it when the underlying
/// [`StateMachine`] approves the transition.
pub struct Enforced<S: StateMachine> {
    machine: S,
    current: S::State,
}

impl<S: StateMachine> Enforced<S> {
    /// Create a new enforced state machine starting in `initial`.
    pub fn new(machine: S, initial: S::State) -> Self {
        Self {
            machine,
            current: initial,
        }
    }

    /// Apply an event, mutating state only if the transition is valid.
    ///
    /// On success, returns a reference to the new state.
    /// On failure, state is left unchanged and the error is returned.
    pub fn apply(
        &mut self,
        event: &S::Event,
        ctx: &S::Context,
    ) -> Result<&S::State, InvalidTransitionError> {
        let new_state = self.machine.transition(&self.current, event, ctx)?;
        self.current = new_state;
        Ok(&self.current)
    }

    /// Read-only access to the current state.
    pub fn current(&self) -> &S::State {
        &self.current
    }

    /// Replace the current state unconditionally (useful for initialization).
    pub fn set_state(&mut self, state: S::State) {
        self.current = state;
    }

    /// Get the name of the underlying state machine.
    pub fn name(&self) -> &str {
        self.machine.name()
    }
}

/// A state machine that allows all transitions (useful for disabling enforcement).
pub struct PermissiveMachine<S, E, C> {
    _phantom: std::marker::PhantomData<(S, E, C)>,
}

impl<S, E, C> PermissiveMachine<S, E, C> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S, E, C> Default for PermissiveMachine<S, E, C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, E, C> StateMachine for PermissiveMachine<S, E, C>
where
    S: Clone + PartialEq + Debug + Send + Sync,
    E: Clone + Debug + Send + Sync,
    C: Send + Sync,
{
    type State = S;
    type Event = E;
    type Context = C;

    fn transition(
        &self,
        _current: &Self::State,
        _event: &Self::Event,
        _ctx: &Self::Context,
    ) -> Result<Self::State, InvalidTransitionError> {
        // Permissive machine never actually called in practice because
        // Enforced::apply would need to produce a new state.
        // This is a placeholder; in practice you'd clone current or use Option.
        unreachable!("PermissiveMachine should be used with a custom Enforced wrapper")
    }

    fn name(&self) -> &str {
        "permissive"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum LightState {
        Off,
        On,
        Broken,
    }

    #[derive(Clone, Debug, PartialEq)]
    enum LightEvent {
        SwitchOn,
        SwitchOff,
        Smash,
    }

    struct LightMachine;

    impl StateMachine for LightMachine {
        type State = LightState;
        type Event = LightEvent;
        type Context = ();

        fn transition(
            &self,
            current: &Self::State,
            event: &Self::Event,
            _ctx: &Self::Context,
        ) -> Result<Self::State, InvalidTransitionError> {
            match (current, event) {
                (LightState::Off, LightEvent::SwitchOn) => Ok(LightState::On),
                (LightState::On, LightEvent::SwitchOff) => Ok(LightState::Off),
                (LightState::On, LightEvent::Smash) => Ok(LightState::Broken),
                (LightState::Off, LightEvent::Smash) => Ok(LightState::Broken),
                _ => Err(InvalidTransitionError {
                    from: format!("{:?}", current),
                    event: format!("{:?}", event),
                }),
            }
        }
    }

    #[test]
    fn test_valid_transition() {
        let mut sm = Enforced::new(LightMachine, LightState::Off);
        assert_eq!(sm.current(), &LightState::Off);

        sm.apply(&LightEvent::SwitchOn, &()).unwrap();
        assert_eq!(sm.current(), &LightState::On);
    }

    #[test]
    fn test_invalid_transition_keeps_state() {
        let mut sm = Enforced::new(LightMachine, LightState::Off);

        let err = sm.apply(&LightEvent::SwitchOff, &()).unwrap_err();
        assert!(err.to_string().contains("Invalid transition"));
        assert_eq!(sm.current(), &LightState::Off); // unchanged
    }

    #[test]
    fn test_broken_transition() {
        let mut sm = Enforced::new(LightMachine, LightState::On);

        sm.apply(&LightEvent::Smash, &()).unwrap();
        assert_eq!(sm.current(), &LightState::Broken);

        // Once broken, can't switch on
        let err = sm.apply(&LightEvent::SwitchOn, &()).unwrap_err();
        assert!(err.to_string().contains("Invalid transition"));
    }
}
