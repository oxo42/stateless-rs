use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use crate::state_representation::StateRepresentation;
use crate::transition::Transition;
use crate::transition_event;
use crate::trigger_behaviour::TriggerBehaviour;
use crate::StateMachineError;
use crate::TransitionEventHandler;

/// A finite state machine which holds a state object.
///
/// This can only be built by a [`crate::StateMachineBuilder`].
///
/// TODO:
/// * Make this thread safe
///
/// ## State Object
///
/// Whatever you want to put into the state machine.  This will be wrapped
/// inside a [`std::sync::Mutex`].  If you want to pull it out you will need to
/// call `.object()` which will return a [`std::sync::MutexGuard`] and will need
/// to be dereferenced
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
    // Must create with StateMachineBuilder
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

    /// Pull out the object that went into the
    /// [`crate::StateMachineBuilder.build`] as a [`std::sync::MutexGuard`]
    ///
    /// ## Example
    /// ```
    /// # use stateless_rs::StateMachineBuilder;
    /// # #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, strum_macros::EnumIter)]
    /// # enum State { On }
    /// # #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    /// # enum Trigger {
    /// # }
    /// # fn main() -> Result<(), stateless_rs::StateMachineError<State,Trigger>> {
    /// let mut builder = StateMachineBuilder::<State, Trigger, i32>::new(State::On);
    /// let machine = builder.build(42)?;
    /// {
    ///     let the_answer = *machine.object();
    ///     println!("The answer is {the_answer}");
    /// } // Mutex unlocked here
    /// # Ok(())
    /// # }
    /// ```
    pub fn object(&self) -> MutexGuard<O> {
        let o = self.object.lock().unwrap();
        o
    }

    /// Returns the current state of the state machine
    pub fn state(&self) -> S {
        self.current_state
    }

    /// Fire a trigger.  Will return `()` on success and a
    /// [`crate::StateMachineError`] on failure
    ///
    /// TODO
    /// * Implement a queue and concurrent access
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

        let behaviour = {
            let representation = self
                .representation()
                .expect("representations should all exist");
            representation.get_behaviour(trigger)?
        };
        let transition = match behaviour {
            TriggerBehaviour::Transitioning(b) => {
                let representation = self
                    .representation()
                    .expect("representations should all exist");
                let destination = b.fire(current_state);
                let transition = Transition::new(current_state, trigger, destination);
                representation.exit(&transition, Arc::clone(&state_object));
                self.current_state = transition.destination;
                let representation = self
                    .representation()
                    .expect("representations should all exist");
                representation.enter(&transition, state_object);
                transition
            }
            TriggerBehaviour::Internal(b) => {
                b.fire(current_state); // TODO: does nothing now. Maybe needed for parameters
                let representation = self
                    .representation()
                    .expect("representations should all exist");
                let transition = Transition::new(current_state, trigger, current_state);
                representation.fire_internal_actions(&transition, Arc::clone(&state_object));
                transition
            }
        };

        self.transition_event.fire_events(&transition);

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
    use super::*;
    use crate::tests::{State, Trigger};
    use crate::StateMachineBuilder;

    #[test]
    fn entry_into_unconfigured_state_works() -> eyre::Result<()> {
        // If the user hasn't explicitly configured a state to do something, it
        // is still part of the State enum and is a valid destination
        let mut builder = StateMachineBuilder::new(State::State1);
        builder
            .config(State::State1)
            .permit(Trigger::Trig, State::State2);
        let mut machine = builder.build(())?;

        assert_eq!(machine.state(), State::State1);
        let result = machine.fire(Trigger::Trig)?;
        assert_eq!(machine.state(), State::State2);
        Ok(())
    }

    #[test]
    fn fire_for_not_defined_throws_error() -> eyre::Result<()> {
        let mut machine = StateMachineBuilder::new(State::State2).build(())?;
        let result = machine.fire(Trigger::Trig);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(
            error,
            StateMachineError::TriggerNotPermitted {
                state: State::State2,
                trigger: Trigger::Trig
            }
        );
        Ok(())
    }

    #[test]
    fn statemachine_on_entry_fires() -> eyre::Result<()> {
        let mut builder = StateMachineBuilder::new(State::State1);
        builder
            .config(State::State1)
            .permit(Trigger::Trig, State::State2);
        builder
            .config(State::State2)
            .on_entry(move |_transition, obj| *obj = true);

        let mut machine = builder.build(false)?;

        assert_eq!(machine.state(), State::State1);
        machine.fire(Trigger::Trig)?;
        assert_eq!(machine.state(), State::State2);
        assert!(*machine.object());
        Ok(())
    }

    #[test]
    fn statemachine_on_entry_fires_multiple_actions() -> eyre::Result<()> {
        let mut builder = StateMachineBuilder::new(State::State1);
        builder
            .config(State::State1)
            .permit(Trigger::Trig, State::State2);
        builder
            .config(State::State2)
            .on_entry(move |_transition, object| {
                *object += 1;
            })
            .on_entry(move |_transition, object| {
                *object += 2;
            });

        let mut machine = builder.build(0)?;

        assert_eq!(machine.state(), State::State1);
        machine.fire(Trigger::Trig)?;
        assert_eq!(machine.state(), State::State2);
        assert_eq!(*machine.object(), 3);
        Ok(())
    }

    #[test]
    fn statemachine_on_exit_fires_multiple_actions() -> eyre::Result<()> {
        let mut builder = StateMachineBuilder::new(State::State1);
        builder
            .config(State::State1)
            .on_exit(move |_transition, object| {
                *object += 1;
            })
            .on_exit(move |_transition, object| {
                *object += 2;
            })
            .permit(Trigger::Trig, State::State2);

        let mut machine = builder.build(0)?;

        assert_eq!(machine.state(), State::State1);
        machine.fire(Trigger::Trig)?;
        assert_eq!(machine.state(), State::State2);
        assert_eq!(*machine.object(), 3);
        Ok(())
    }

    #[test]
    fn transitioned_event_happens_on_transition() -> eyre::Result<()> {
        let count = Arc::new(Mutex::new(0));
        let count1 = Arc::clone(&count);

        let mut builder = StateMachineBuilder::new(State::State1);
        builder
            .config(State::State1)
            .permit(Trigger::Trig, State::State2);

        builder.on_transitioned(move |_t| {
            let mut data = count1.lock().unwrap();
            *data += 1
        });

        let mut machine = builder.build(())?;
        machine.fire(Trigger::Trig)?;

        assert_eq!(*count.lock().unwrap(), 1);
        Ok(())
    }

    #[test]
    fn internal_transition_fires() -> eyre::Result<()> {
        let mut builder = StateMachineBuilder::<_, _, i32>::new(State::State1);
        builder
            .config(State::State1)
            .internal_transition(Trigger::Trig, |_t, o| *o += 1);

        let mut machine = builder.build(0)?;
        machine.fire(Trigger::Trig)?;

        assert_eq!(*machine.object(), 1);
        Ok(())
    }

    #[test]
    fn internal_transition_does_not_fire_on_entry() -> eyre::Result<()> {
        let mut builder = StateMachineBuilder::<_, _, i32>::new(State::State1);
        builder
            .config(State::State1)
            .permit(Trigger::Trig, State::State2);

        builder
            .config(State::State2)
            .internal_transition(Trigger::Trig, |_t, o| *o += 1);

        let mut machine = builder.build(0)?;
        machine.fire(Trigger::Trig)?; // send to state2
        assert_eq!(machine.state(), State::State2);
        assert_eq!(*machine.object(), 0, "internal not fired");
        machine.fire(Trigger::Trig)?; // re-enter to state2
        assert_eq!(machine.state(), State::State2);
        assert_eq!(*machine.object(), 1, "internal has fired");
        Ok(())
    }

    #[test]
    fn entry_action_does_not_fire_on_internal_transition() -> eyre::Result<()> {
        let mut builder = StateMachineBuilder::<_, _, i32>::new(State::State1);
        builder
            .config(State::State1)
            .permit(Trigger::Trig, State::State2);

        builder
            .config(State::State2)
            .on_entry(|_t, o| *o += 1)
            .internal_transition(Trigger::Trig, |_, _| ());

        let mut machine = builder.build(0)?;
        machine.fire(Trigger::Trig)?; // send to state2
        assert_eq!(machine.state(), State::State2);
        assert_eq!(*machine.object(), 1, "entry has fired");
        machine.fire(Trigger::Trig)?; // re-enter to state2
        assert_eq!(machine.state(), State::State2);
        assert_eq!(*machine.object(), 1, "entry not fired");
        Ok(())
    }
}
