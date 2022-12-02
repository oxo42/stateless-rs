use derivative::Derivative;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::FnOnce;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::state_config::StateConfig;
use crate::state_config::WrappedStateRep;
use crate::state_machine::StateMachine;
use crate::state_representation::StateRepresentation;
use crate::transition::Transition;
use crate::StateMachineError;
use crate::TransitionEventHandler;

fn unwrap_rc_and_refcell<R>(item: Rc<RefCell<R>>) -> Result<R, Rc<RefCell<R>>> {
    let unrc = Rc::try_unwrap(item)?;
    let val = unrc.into_inner();
    Ok(val)
}

#[derive(Debug)]
pub struct StateMachineBuilder<S, T, O> {
    initial_state: S,
    states: HashMap<S, WrappedStateRep<S, T, O>>,
    transition_event: TransitionEventHandler<S, T>,
}

impl<S, T, O> StateMachineBuilder<S, T, O>
where
    S: IntoEnumIterator + Debug + Copy + Eq + Hash + 'static,
    T: Debug + Copy + Eq + Hash + 'static,
    O: Debug,
{
    pub fn new(initial_state: S) -> Self {
        let states: HashMap<S, WrappedStateRep<S, T, O>> = S::iter()
            .map(|state| {
                (
                    state,
                    Rc::new(RefCell::new(StateRepresentation::new(state))),
                )
            })
            .collect();
        StateMachineBuilder {
            initial_state,
            states,
            transition_event: TransitionEventHandler::new(),
        }
    }

    pub fn config(&mut self, state: S) -> StateConfig<S, T, O> {
        let representation = self
            .states
            .get(&state)
            .expect("all states to have been created in constructor");
        StateConfig::new(Rc::clone(representation))
    }

    pub fn on_transitioned<F>(&mut self, f: F)
    where
        F: FnMut(&Transition<S, T>) + 'static,
    {
        self.transition_event.add_event(f);
    }

    /// Will consume the `StateMachineBuilder` and return a `StateMachine`.  The
    /// `state_object` will be wrapped in a `Arc<Mutex<O>>` and you can pull it
    /// out with
    /// ```
    /// # use stateless_rs::StateMachineBuilder;
    /// # #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, strum_macros::EnumIter)]
    /// # enum State { On }
    /// # #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    /// # enum Trigger {
    /// # }
    /// # fn main() -> Result<(), stateless_rs::StateMachineError<State,Trigger>> {
    /// let object = 42;
    /// let mut builder = StateMachineBuilder::<State, Trigger, i32>::new(State::On);
    /// let machine = builder.build(object)?;
    /// let object = machine.object(); // Returns Arc<Mutex<i32>>
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, state_object: O) -> Result<StateMachine<S, T, O>, StateMachineError<S, T>> {
        // StateMachine::new(self.initial_state, self.states)
        let state_reps: Result<HashMap<S, StateRepresentation<S, T, O>>, _> = self
            .states
            .into_iter()
            .map(|(state, rc_ref_rep)| {
                let rep = unwrap_rc_and_refcell(rc_ref_rep);
                rep.map(|r| (state, r))
                    .map_err(|r| StateMachineError::<S, T>::ConfigStillInUse {
                        state: r.borrow().state(),
                    })
            })
            .collect();
        Ok(StateMachine::new(
            self.initial_state,
            state_reps?,
            Arc::new(Mutex::new(state_object)),
            self.transition_event,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{State, Trigger};

    #[test]
    fn check_all_states_are_configured_on_new() {
        let builder = StateMachineBuilder::<State, Trigger, ()>::new(State::State1);
        assert_eq!(builder.states.len(), State::iter().count());
        assert!(State::iter().all(|s| builder.states.contains_key(&s)));
    }

    #[test]
    fn test_builder_config_works() {
        let mut builder = StateMachineBuilder::new(State::State1);
        builder
            .config(State::State1)
            .permit(Trigger::Trig, State::State2);
        builder
            .config(State::State2)
            .permit(Trigger::Trig, State::State1);

        assert_eq!(builder.states.len(), 2);

        let _machine = builder.build(());
    }

    #[test]
    fn test_builder_on_entry_adds_to_state_representation() -> eyre::Result<()> {
        let mut builder = StateMachineBuilder::<State, Trigger, ()>::new(State::State1);
        builder
            .config(State::State1)
            .on_entry(|_t, _o| println!("foobar"));

        let rep = builder.states[&State::State1].borrow();
        assert_eq!(rep.entry_actions.len(), 1);
        Ok(())
    }

    #[test]
    fn test_builder_on_exit_adds_to_state_representation() -> eyre::Result<()> {
        let mut builder = StateMachineBuilder::<State, Trigger, ()>::new(State::State1);
        builder
            .config(State::State1)
            .on_exit(|_t, _o| println!("foobar"));

        let rep = builder.states[&State::State1].borrow();
        assert_eq!(rep.exit_actions.len(), 1);
        Ok(())
    }

    #[test]
    fn on_transition_twice_adds_two_events() -> eyre::Result<()> {
        let mut builder = StateMachineBuilder::<State, Trigger, ()>::new(State::State1);
        builder.on_transitioned(|_t| ());
        builder.on_transitioned(|_t| ());
        assert_eq!(builder.transition_event.events.len(), 2);

        Ok(())
    }
}
