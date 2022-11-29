use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;

use crate::state_representation::StateRepresentation;
use crate::transition::Transition;
use crate::StateMachineError;

#[derive(Debug)]
pub struct StateMachine<S, T> {
    current_state: S,
    state_representations: HashMap<S, StateRepresentation<S, T>>,
}

impl<S, T> Display for StateMachine<S, T>
where
    S: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StateMachine ( state: {:?} )", self.current_state)
    }
}

impl<S, T> StateMachine<S, T>
where
    S: Copy + Eq + Hash + Debug,
    T: Copy + Eq + Hash + Debug,
{
    pub fn new(
        initial_state: S,
        state_representations: HashMap<S, StateRepresentation<S, T>>,
    ) -> Self {
        Self {
            current_state: initial_state,
            state_representations,
        }
    }

    fn state(&self) -> S {
        self.current_state
    }

    pub fn fire(&mut self, trigger: T) -> Result<(), StateMachineError<S, T>> {
        // Set up queue
        self.fireone(trigger)
    }

    fn representation(
        &self,
        state: S,
    ) -> Result<&StateRepresentation<S, T>, StateMachineError<S, T>> {
        self.state_representations.get(&self.current_state).ok_or(
            StateMachineError::StateNotConfigured {
                state: self.current_state,
            },
        )
    }

    fn fireone(&mut self, trigger: T) -> Result<(), StateMachineError<S, T>> {
        let representation = self.representation(self.current_state)?;

        let destination = representation.fire_trigger(trigger)?;
        let transition = Transition::new(self.current_state, trigger, destination);

        // representation.exit(transition);

        self.current_state = transition.destination;

        // let new_representation = self.representation(self.current_state)?;
        // invoke on transitioned event
        // new_representation.enter(transition);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::StateMachineBuilder;

    use super::*;

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    enum Trigger {
        CallDialed,
        HungUp,
        CallConnected,
        LeftMessage,
        PlacedOnHold,
        TakenOffHold,
        PhoneHurledAgainstWall,
        MuteMicrophone,
        UnmuteMicrophone,
        SetVolume,
    }

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    enum State {
        OffHook,
        Ringing,
        Connected,
        OnHold,
        PhoneDestroyed,
    }

    #[test]
    fn foo() -> anyhow::Result<()> {
        let mut builder = StateMachineBuilder::new(State::OffHook);
        builder
            .config(State::OffHook)
            .permit(Trigger::CallDialed, State::Ringing);
        let mut machine = builder.build()?;

        assert_eq!(machine.state(), State::OffHook);
        machine.fire(Trigger::CallDialed)?;
        assert_eq!(machine.state(), State::Ringing);
        Ok(())
    }

    #[test]
    fn fire_for_not_defined_throws_error() -> anyhow::Result<()> {
        let mut machine = StateMachineBuilder::new(State::OffHook).build()?;
        let result = machine.fire(Trigger::CallDialed);
        assert!(result.is_err());
        Ok(())
    }
}
