use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::Mutex;

use crate::state_representation::StateRepresentation;
use crate::transition::Transition;
use crate::StateMachineError;

#[derive(Debug)]
pub struct StateMachine<S, T, O> {
    current_state: S,
    state_representations: HashMap<S, StateRepresentation<S, T, O>>,
    object: Arc<Mutex<O>>,
}

impl<S, T, O> StateMachine<S, T, O>
where
    S: Copy + Eq + Hash + Debug,
    T: Copy + Eq + Hash + Debug,
    O: Debug,
{
    // Visible only to this crate
    pub(crate) fn new(
        initial_state: S,
        state_representations: HashMap<S, StateRepresentation<S, T, O>>,
        object: Arc<Mutex<O>>,
    ) -> Self {
        Self {
            current_state: initial_state,
            state_representations,
            object,
        }
    }

    pub fn object(&self) -> Arc<Mutex<O>> {
        Arc::clone(&self.object)
    }
    pub fn state(&self) -> S {
        self.current_state
    }

    pub fn fire(&mut self, trigger: T) -> Result<(), StateMachineError<S, T>> {
        // Set up queue
        self.fireone(trigger)
    }

    fn representation(&mut self) -> Option<&mut StateRepresentation<S, T, O>> {
        self.state_representations.get_mut(&self.current_state)
    }

    fn fireone(&mut self, trigger: T) -> Result<(), StateMachineError<S, T>> {
        let representation = self
            .representation()
            .expect("representations should all exist");

        let destination = representation.fire_trigger(trigger)?;
        let transition = Transition::new(self.current_state, trigger, destination);

        // representation.exit(transition);

        self.current_state = transition.destination;

        let state_object = Arc::clone(&self.object);

        let new_representation = self
            .representation()
            .expect("representations should all exist");
        // TODO: invoke on transitioned event
        new_representation.enter(transition, state_object);
        Ok(())
    }
}

impl<S, T, O> Display for StateMachine<S, T, O>
where
    S: Debug,
    O: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "StateMachine ( state: {:?}, object: {:?} )",
            self.current_state, self.object
        )
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
        let mut machine = builder.build(())?;

        assert_eq!(machine.state(), State::Off);
        let result = machine.fire(Trigger::Switch)?;
        assert_eq!(machine.state(), State::On);
        Ok(())
    }

    #[test]
    fn fire_for_not_defined_throws_error() -> eyre::Result<()> {
        let mut machine = StateMachineBuilder::new(State::On).build(())?;
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
            .on_entry(move |_transition, _obj| *fired_clone.borrow_mut() = true);

        let mut machine = builder.build(())?;

        assert_eq!(machine.state(), State::Off);
        machine.fire(Trigger::Switch)?;
        assert_eq!(machine.state(), State::On);
        assert!(*fired.borrow());
        Ok(())
    }

    #[test]
    fn statemachine_on_entry_fires_multiple_actions() -> eyre::Result<()> {
        let mut builder = StateMachineBuilder::new(State::Off);
        builder
            .config(State::Off)
            .permit(Trigger::Switch, State::On);
        // TODO this is too hard
        builder
            .config(State::On)
            .on_entry(move |_transition, object| {
                let mut data = object.lock().unwrap();
                *data += 1;
            })
            .on_entry(move |_transition, object| {
                let mut data = object.lock().unwrap();
                *data += 2;
            });

        let mut machine = builder.build(0)?;

        let count = machine.object();

        assert_eq!(machine.state(), State::Off);
        machine.fire(Trigger::Switch)?;
        assert_eq!(machine.state(), State::On);
        assert_eq!(*count.lock().unwrap(), 3);
        Ok(())
    }
}
