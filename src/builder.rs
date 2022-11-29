use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::rc::Rc;

use crate::state_machine::StateMachine;
use crate::state_representation::StateRepresentation;
use crate::trigger_behaviour::TransitioningTriggerBehaviour;
use crate::StateMachineError;

#[derive(Debug)]
pub struct StateMachineBuilder<S, T> {
    initial_state: S,
    states: HashMap<S, Rc<RefCell<StateRepresentation<S, T>>>>,
}

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
}

fn unwrap_rc_and_refcell<R>(item: Rc<RefCell<R>>) -> Result<R, Rc<RefCell<R>>> {
    let unrc = Rc::try_unwrap(item)?;
    let val = unrc.into_inner();
    Ok(val)
}

impl<S, T> StateMachineBuilder<S, T>
where
    S: Debug + Copy + Eq + Hash + 'static,
    T: Debug + Copy + Eq + Hash + 'static,
{
    pub fn new(initial_state: S) -> Self {
        StateMachineBuilder {
            initial_state,
            states: HashMap::new(),
        }
    }

    pub fn config(&mut self, state: S) -> StateConfig<S, T> {
        let representation = self
            .states
            .entry(state)
            .or_insert_with(|| Rc::new(RefCell::new(StateRepresentation::new(state))));
        StateConfig::new(Rc::clone(representation))
    }

    pub fn build(self) -> Result<StateMachine<S, T>, StateMachineError<S, T>> {
        // StateMachine::new(self.initial_state, self.states)
        let x: Result<HashMap<S, StateRepresentation<S, T>>, _> = self
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
        Ok(StateMachine::new(self.initial_state, x?))
    }
}

struct BuilderConfig<'a, S, T> {
    representation: &'a StateRepresentation<S, T>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    enum State {
        State1,
        State2,
    }

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    enum Trigger {
        Trig,
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
}
