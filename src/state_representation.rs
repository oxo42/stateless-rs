use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use crate::trigger_behaviour::TriggerBehaviour;
use crate::StateMachineError;

#[derive(Debug)]
pub struct StateRepresentation<S, T> {
    state: S,
    trigger_behaviours: HashMap<T, Box<dyn TriggerBehaviour<S, T>>>,
    // entry_actions: Vec<()>,
    // exit_actions: Vec<()>,
    // activate_actions: Vec<()>,
    // deactivate_actions: Vec<()>,
    // internal_actions: Vec<()>,
    // substates: Vec<Self>,
}

impl<S, T> StateRepresentation<S, T>
where
    S: Copy + Debug,
    T: Eq + Hash + Debug + Copy,
{
    pub fn new(state: S) -> Self {
        Self {
            state,
            trigger_behaviours: HashMap::new(),
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

    pub fn fire_trigger(&self, trigger: T) -> Result<S, StateMachineError<S, T>> {
        match self.trigger_behaviours.get(&trigger) {
            Some(b) => Ok(b.fire(self.state)),
            None => Err(StateMachineError::TriggerNotPermitted {
                state: self.state,
                trigger,
            }),
        }
    }
}
