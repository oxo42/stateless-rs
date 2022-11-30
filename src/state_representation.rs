use crate::transition::Transition;
use crate::trigger_behaviour::TriggerBehaviour;
use crate::StateMachineError;
use derivative::Derivative;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::FnOnce;
use std::sync::{Arc, Mutex};

type Action<S, T, O> = Box<dyn FnMut(&Transition<S, T>, Arc<Mutex<O>>)>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct StateRepresentation<S, T, O> {
    state: S,
    trigger_behaviours: HashMap<T, Box<dyn TriggerBehaviour<S, T>>>,
    #[derivative(Debug = "ignore")]
    pub(crate) entry_actions: Vec<Action<S, T, O>>,
    #[derivative(Debug = "ignore")]
    pub(crate) exit_actions: Vec<Action<S, T, O>>,
    // activate_actions: Vec<()>,
    // deactivate_actions: Vec<()>,
    // internal_actions: Vec<()>,
    // substates: Vec<Self>,
}

impl<S, T, O> StateRepresentation<S, T, O>
where
    S: Copy + Debug,
    T: Eq + Hash + Debug + Copy,
{
    pub fn new(state: S) -> Self {
        Self {
            state,
            trigger_behaviours: HashMap::new(),
            entry_actions: Vec::new(),
            exit_actions: Vec::new(),
        }
    }

    pub fn state(&self) -> S {
        self.state
    }

    pub fn add_trigger_behaviour(
        &mut self,
        trigger: T,
        behaviour: impl TriggerBehaviour<S, T> + 'static,
    ) {
        self.trigger_behaviours.insert(trigger, Box::new(behaviour));
    }

    pub fn add_entry_action<F>(&mut self, f: F)
    where
        F: FnMut(&Transition<S, T>, Arc<Mutex<O>>) + 'static,
    {
        self.entry_actions.push(Box::new(f));
    }

    pub fn add_exit_action<F>(&mut self, f: F)
    where
        F: FnMut(&Transition<S, T>, Arc<Mutex<O>>) + 'static,
    {
        self.exit_actions.push(Box::new(f));
    }

    pub fn fire_trigger(&self, trigger: T) -> Result<S, StateMachineError<S, T>> {
        match self.trigger_behaviours.get(&trigger) {
            Some(b) => Ok(b.fire(self.state)),
            None => Err(StateMachineError::TriggerNotPermitted {
                state: self.state,
                trigger,
            }),
        }
    }

    pub fn enter(&mut self, transition: Transition<S, T>, state_object: Arc<Mutex<O>>) {
        for action in self.entry_actions.iter_mut() {
            action(&transition, Arc::clone(&state_object));
        }
    }

    pub fn exit(&mut self, transition: Transition<S, T>, state_object: Arc<Mutex<O>>) {
        for action in self.exit_actions.iter_mut() {
            action(&transition, Arc::clone(&state_object));
        }
    }
}
