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

use crate::state_machine::StateMachine;
use crate::state_representation::StateRepresentation;
use crate::transition::Transition;
use crate::trigger_behaviour::InternalTransitioningTriggerBehaviour;
use crate::trigger_behaviour::TransitioningTriggerBehaviour;
use crate::StateMachineError;
use crate::TransitionEventHandler;

pub(crate) type WrappedStateRep<S, T, O> = Rc<RefCell<StateRepresentation<S, T, O>>>;

pub struct StateConfig<S, T, O> {
    rep: WrappedStateRep<S, T, O>,
}

impl<S, T, O> StateConfig<S, T, O>
where
    S: Debug + Copy + Eq + Hash + 'static,
    T: Debug + Copy + Eq + Hash + 'static,
{
    pub(crate) fn new(rep: WrappedStateRep<S, T, O>) -> Self {
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

    pub fn internal_transition<F>(self, trigger: T, internal_action: F) -> Self
    where
        F: FnMut(&Transition<S, T>, &mut O) + 'static,
    {
        let behaviour = InternalTransitioningTriggerBehaviour::new(trigger);
        self.rep
            .borrow_mut()
            .add_trigger_behaviour(trigger, behaviour);
        self
    }

    pub fn on_entry<F>(self, f: F) -> Self
    where
        F: FnMut(&Transition<S, T>, &mut O) + 'static,
    {
        self.rep.borrow_mut().add_entry_action(f);
        self
    }

    pub fn on_exit<F>(self, f: F) -> Self
    where
        F: FnMut(&Transition<S, T>, &mut O) + 'static,
    {
        self.rep.borrow_mut().add_exit_action(f);
        self
    }
}
