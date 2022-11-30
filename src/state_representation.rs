use crate::transition::Transition;
use crate::trigger_behaviour::{TrigBehaviour, TriggerBehaviour};
use crate::StateMachineError;
use derivative::Derivative;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::FnOnce;
use std::sync::{Arc, Mutex};

type Action<S, T, O> = Box<dyn FnMut(&Transition<S, T>, &mut O)>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct StateRepresentation<S, T, O> {
    state: S,
    trigger_behaviours: HashMap<T, TrigBehaviour<S, T>>,
    #[derivative(Debug = "ignore")]
    pub(crate) entry_actions: Vec<Action<S, T, O>>,
    #[derivative(Debug = "ignore")]
    pub(crate) exit_actions: Vec<Action<S, T, O>>,
    #[derivative(Debug = "ignore")]
    pub(crate) internal_actions: Vec<Action<S, T, O>>,
    // activate_actions: Vec<()>,
    // deactivate_actions: Vec<()>,
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
            internal_actions: Vec::new(),
        }
    }

    pub fn state(&self) -> S {
        self.state
    }

    pub(crate) fn add_trigger_behaviour(&mut self, trigger: T, behaviour: TrigBehaviour<S, T>) {
        self.trigger_behaviours.insert(trigger, behaviour);
    }

    pub fn add_entry_action<F>(&mut self, f: F)
    where
        F: FnMut(&Transition<S, T>, &mut O) + 'static,
    {
        self.entry_actions.push(Box::new(f));
    }

    pub fn add_exit_action<F>(&mut self, f: F)
    where
        F: FnMut(&Transition<S, T>, &mut O) + 'static,
    {
        self.exit_actions.push(Box::new(f));
    }

    pub fn add_internal_action<F>(&mut self, f: F)
    where
        F: FnMut(&Transition<S, T>, &mut O) + 'static,
    {
        self.internal_actions.push(Box::new(f));
    }

    pub(crate) fn get_behaviour(
        &self,
        trigger: T,
    ) -> Result<TrigBehaviour<S, T>, StateMachineError<S, T>> {
        let b = self.trigger_behaviours.get(&trigger).ok_or_else(|| {
            StateMachineError::TriggerNotPermitted {
                state: self.state,
                trigger,
            }
        })?;
        Ok(b.clone())
    }

    pub fn enter(&mut self, transition: &Transition<S, T>, state_object: Arc<Mutex<O>>) {
        for action in self.entry_actions.iter_mut() {
            let mut object = state_object.lock().unwrap();
            action(transition, &mut *object);
        }
    }

    pub fn exit(&mut self, transition: &Transition<S, T>, state_object: Arc<Mutex<O>>) {
        for action in self.exit_actions.iter_mut() {
            let mut object = state_object.lock().unwrap();
            action(transition, &mut *object);
        }
    }

    pub fn fire_internal_actions(
        &mut self,
        transition: &Transition<S, T>,
        state_object: Arc<Mutex<O>>,
    ) {
        for action in self.internal_actions.iter_mut() {
            let mut object = state_object.lock().unwrap();
            action(transition, &mut *object);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{State, Trigger};

    #[test]
    fn unconfigured_trigger_errors() {
        let rep = StateRepresentation::<_, _, ()>::new(State::State1);
        let result = rep.get_behaviour(Trigger::Trig);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            StateMachineError::TriggerNotPermitted {
                state: State::State1,
                trigger: Trigger::Trig
            }
        );
    }
}
