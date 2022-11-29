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
    // Visible only to this crate
    pub(crate) fn new(
        initial_state: S,
        state_representations: HashMap<S, StateRepresentation<S, T>>,
    ) -> Self {
        Self {
            current_state: initial_state,
            state_representations,
        }
    }

    pub fn state(&self) -> S {
        self.current_state
    }

    pub fn fire(&mut self, trigger: T) -> Result<(), StateMachineError<S, T>> {
        // Set up queue
        self.fireone(trigger)
    }

    fn representation(&self) -> Option<&StateRepresentation<S, T>> {
        self.state_representations.get(&self.current_state)
    }

    fn fireone(&mut self, trigger: T) -> Result<(), StateMachineError<S, T>> {
        let representation = self.representation().unwrap();

        let destination = representation.fire_trigger(trigger)?;
        let transition = Transition::new(self.current_state, trigger, destination);

        // representation.exit(transition);

        self.current_state = transition.destination;

        let new_representation = match self.representation() {
            Some(r) => r,
            None => return Ok(()),
        };
        // invoke on transitioned event
        new_representation.enter(transition);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use strum_macros::EnumIter;

    use crate::StateMachineBuilder;

    use super::*;

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    enum Trigger {
        Switch,
        Bloop,
    }

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, EnumIter)]
    enum State {
        On,
        Off,
    }

    #[test]
    fn entry_into_unconfigured_state_works() -> eyre::Result<()> {
        // If the user hasn't explicitly configured a state to do something, it
        // is still part of the State enum and is a valid destination
        let mut builder = StateMachineBuilder::new(State::Off);
        builder
            .config(State::Off)
            .permit(Trigger::Switch, State::On);
        let mut machine = builder.build()?;

        assert_eq!(machine.state(), State::Off);
        let result = machine.fire(Trigger::Switch)?;
        assert_eq!(machine.state(), State::On);
        Ok(())
    }

    #[test]
    fn fire_for_not_defined_throws_error() -> eyre::Result<()> {
        let mut machine = StateMachineBuilder::new(State::On).build()?;
        let result = machine.fire(Trigger::Switch);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(
            error,
            StateMachineError::TriggerNotPermitted {
                state: State::On,
                trigger: Trigger::Switch
            }
        );
        Ok(())
    }

    #[test]
    fn statemachine_on_entry_fires() -> eyre::Result<()> {
        let fired = Rc::new(RefCell::new(false));
        let fired_clone = Rc::clone(&fired);
        let mut builder = StateMachineBuilder::new(State::Off);
        builder
            .config(State::Off)
            .permit(Trigger::Switch, State::On);
        builder
            .config(State::On)
            .on_entry(move |_t| *fired_clone.borrow_mut() = true);

        let mut machine = builder.build()?;

        assert_eq!(machine.state(), State::Off);
        machine.fire(Trigger::Switch)?;
        assert_eq!(machine.state(), State::On);
        assert!(*fired.borrow());
        Ok(())
    }
}
