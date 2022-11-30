use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::Mutex;

use crate::state_representation::StateRepresentation;
use crate::transition::Transition;
use crate::transition_event;
use crate::StateMachineError;
use crate::TransitionEventHandler;

#[derive(Debug)]
pub struct StateMachine<S, T, O> {
    current_state: S,
    state_representations: HashMap<S, StateRepresentation<S, T, O>>,
    object: Arc<Mutex<O>>,
    transition_event: TransitionEventHandler<S, T>,
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
        transition_event: TransitionEventHandler<S, T>,
    ) -> Self {
        Self {
            current_state: initial_state,
            state_representations,
            object,
            transition_event,
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
        let state_object = Arc::clone(&self.object);
        let current_state = self.current_state;

        let transition = {
            let representation = self
                .representation()
                .expect("representations should all exist");
            let destination = representation.fire_trigger(trigger)?;
            let transition = Transition::new(current_state, trigger, destination);
            representation.exit(&transition, Arc::clone(&state_object));
            transition
        };

        self.current_state = transition.destination;
        self.transition_event.fire_events(&transition);

        {
            let representation = self
                .representation()
                .expect("representations should all exist");
            representation.enter(&transition, state_object);
        }
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
        let mut builder = StateMachineBuilder::new(State::Off);
        builder
            .config(State::Off)
            .permit(Trigger::Switch, State::On);
        builder
            .config(State::On)
            .on_entry(move |_transition, obj| *obj = true);

        let mut machine = builder.build(false)?;

        assert_eq!(machine.state(), State::Off);
        machine.fire(Trigger::Switch)?;
        assert_eq!(machine.state(), State::On);
        assert!(*machine.object().lock().unwrap());
        Ok(())
    }

    #[test]
    fn statemachine_on_entry_fires_multiple_actions() -> eyre::Result<()> {
        let mut builder = StateMachineBuilder::new(State::Off);
        builder
            .config(State::Off)
            .permit(Trigger::Switch, State::On);
        builder
            .config(State::On)
            .on_entry(move |_transition, object| {
                *object += 1;
            })
            .on_entry(move |_transition, object| {
                *object += 2;
            });

        let mut machine = builder.build(0)?;

        let count = machine.object();

        assert_eq!(machine.state(), State::Off);
        machine.fire(Trigger::Switch)?;
        assert_eq!(machine.state(), State::On);
        assert_eq!(*count.lock().unwrap(), 3);
        Ok(())
    }

    #[test]
    fn statemachine_on_exit_fires_multiple_actions() -> eyre::Result<()> {
        let mut builder = StateMachineBuilder::new(State::Off);
        builder
            .config(State::Off)
            .on_exit(move |_transition, object| {
                *object += 1;
            })
            .on_exit(move |_transition, object| {
                *object += 2;
            })
            .permit(Trigger::Switch, State::On);

        let mut machine = builder.build(0)?;

        let count = machine.object();

        assert_eq!(machine.state(), State::Off);
        machine.fire(Trigger::Switch)?;
        assert_eq!(machine.state(), State::On);
        assert_eq!(*count.lock().unwrap(), 3);
        Ok(())
    }

    #[test]
    fn transitioned_event_happens_on_transition() -> eyre::Result<()> {
        let count = Arc::new(Mutex::new(0));
        let count1 = Arc::clone(&count);

        let mut builder = StateMachineBuilder::new(State::Off);
        builder
            .config(State::Off)
            .permit(Trigger::Switch, State::On);

        builder.on_transitioned(move |_t| {
            let mut data = count1.lock().unwrap();
            *data += 1
        });

        let mut machine = builder.build(())?;
        machine.fire(Trigger::Switch)?;

        assert_eq!(*count.lock().unwrap(), 1);
        Ok(())
    }
}
