use crate::transition::Transition;
use crate::trigger_behaviour::TriggerBehaviour;
use crate::StateMachineError;
use derivative::Derivative;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::FnOnce;

type Action<S, T> = Box<dyn Fn(&Transition<S, T>)>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct StateRepresentation<S, T> {
    state: S,
    trigger_behaviours: HashMap<T, Box<dyn TriggerBehaviour<S, T>>>,
    #[derivative(Debug = "ignore")]
    entry_actions: Vec<Action<S, T>>,
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
            entry_actions: Vec::new(),
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
        F: Fn(&Transition<S, T>) + 'static,
    {
        self.entry_actions.push(Box::new(f));
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

    pub fn enter(&self, transition: Transition<S, T>) {
        for action in self.entry_actions.iter() {
            action(&transition);
        }
    }

    pub(crate) fn entry_actions(&self) -> &Vec<Action<S, T>> {
        &self.entry_actions
    }
}
