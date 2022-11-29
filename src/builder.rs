use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::FnOnce;
use std::rc::Rc;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::state_machine::StateMachine;
use crate::state_representation::StateRepresentation;
use crate::transition::Transition;
use crate::trigger_behaviour::TransitioningTriggerBehaviour;
use crate::StateMachineError;

pub struct StateConfig<S, T> {
    rep: Rc<RefCell<StateRepresentation<S, T>>>,
}

impl<S, T> StateConfig<S, T>
where
    S: Debug + Copy + Eq + Hash + 'static,
    T: Debug + Copy + Eq + Hash + 'static,
{
    fn new(rep: Rc<RefCell<StateRepresentation<S, T>>>) -> Self {
        Self { rep }
    }

    pub fn state(&self) -> S {
        self.rep.borrow().state()
    }

    pub fn permit(self, trigger: T, destination_state: S) -> Self {
        let behaviour = TransitioningTriggerBehaviour::new(trigger, destination_state);
        self.rep
            .borrow_mut()
            .add_trigger_behaviour(trigger, behaviour);
        self
    }

    pub fn on_entry<F>(self, f: F) -> Self
    where
        F: Fn(&Transition<S, T>) + 'static,
    {
        self.rep.borrow_mut().add_entry_action(f);
        self
    }
}

fn unwrap_rc_and_refcell<R>(item: Rc<RefCell<R>>) -> Result<R, Rc<RefCell<R>>> {
    let unrc = Rc::try_unwrap(item)?;
    let val = unrc.into_inner();
    Ok(val)
}

#[derive(Debug)]
pub struct StateMachineBuilder<S, T> {
    initial_state: S,
    states: HashMap<S, Rc<RefCell<StateRepresentation<S, T>>>>,
}

impl<S, T> StateMachineBuilder<S, T>
where
    S: IntoEnumIterator + Debug + Copy + Eq + Hash + 'static,
    T: Debug + Copy + Eq + Hash + 'static,
{
    pub fn new(initial_state: S) -> Self {
        let states: HashMap<S, Rc<RefCell<StateRepresentation<S, T>>>> = S::iter()
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
        }
    }

    pub fn config(&mut self, state: S) -> StateConfig<S, T> {
        let representation = self
            .states
            .get(&state)
            .expect("all states to have been created in constructor");
        StateConfig::new(Rc::clone(representation))
    }

    pub fn build(self) -> Result<StateMachine<S, T>, StateMachineError<S, T>> {
        // StateMachine::new(self.initial_state, self.states)
        let state_reps: Result<HashMap<S, StateRepresentation<S, T>>, _> = self
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
        Ok(StateMachine::new(self.initial_state, state_reps?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, EnumIter)]
    enum State {
        State1,
        State2,
    }

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    enum Trigger {
        Trig,
    }

    #[test]
    fn check_all_states_are_configured_on_new() {
        let builder = StateMachineBuilder::<State, Trigger>::new(State::State1);
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

        let _machine = builder.build();
    }

    #[test]
    fn test_builder_on_entry_adds_to_state_representation() -> eyre::Result<()> {
        let mut builder = StateMachineBuilder::<State, Trigger>::new(State::State1);
        builder
            .config(State::State1)
            .on_entry(|_t| println!("foobar"));

        let rep = builder.states[&State::State1].borrow();
        assert_eq!(rep.entry_actions().len(), 1);
        Ok(())
    }
}
