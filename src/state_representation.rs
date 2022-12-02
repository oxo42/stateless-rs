use crate::transition::Transition;
use crate::trigger_behaviour::TriggerBehaviour;
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
    trigger_behaviours: HashMap<T, TriggerBehaviour<S, T>>,
    #[derivative(Debug = "ignore")]
    pub(crate) entry_actions: Vec<Action<S, T, O>>,
    #[derivative(Debug = "ignore")]
    pub(crate) exit_actions: Vec<Action<S, T, O>>,
    #[derivative(Debug = "ignore")]
    pub(crate) internal_actions: HashMap<T, Vec<Action<S, T, O>>>,
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
            internal_actions: HashMap::new(),
        }
    }

    pub fn state(&self) -> S {
        self.state
    }

    pub(crate) fn add_trigger_behaviour(&mut self, trigger: T, behaviour: TriggerBehaviour<S, T>) {
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

    pub fn add_internal_action<F>(&mut self, trigger: T, f: F)
    where
        F: FnMut(&Transition<S, T>, &mut O) + 'static,
    {
        self.internal_actions
            .entry(trigger)
            .or_default()
            .push(Box::new(f));
    }

    pub(crate) fn get_behaviour(
        &self,
        trigger: T,
    ) -> Result<TriggerBehaviour<S, T>, StateMachineError<S, T>> {
        let b = self.trigger_behaviours.get(&trigger).ok_or(
            StateMachineError::TriggerNotPermitted {
                state: self.state,
                trigger,
            },
        )?;
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
        let Some(actions) = self.internal_actions.get_mut(&transition.trigger) else {
            return;
        };
        for action in actions.iter_mut() {
            let mut object = state_object.lock().unwrap();
            action(transition, &mut *object);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        tests::{State, Trigger},
        transition,
    };

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

    #[test]
    fn internal_actions_fire_for_correct_trigger() -> eyre::Result<()> {
        let trig_fired = Arc::new(Mutex::new(false));
        let trig_fired_clone = Arc::clone(&trig_fired);
        let state = Arc::new(Mutex::new(()));
        let mut rep = StateRepresentation::<_, _, ()>::new(State::State1);
        rep.add_internal_action(Trigger::Trig, move |_, _| {
            *trig_fired_clone.lock().unwrap() = true
        });
        rep.add_internal_action(Trigger::Trig2, |_, _| panic!("trig2 should not have fired"));
        rep.fire_internal_actions(
            &Transition::new(State::State1, Trigger::Trig, State::State1),
            Arc::clone(&state),
        );
        assert!(*trig_fired.lock().unwrap(), "trig should have fired");
        Ok(())
    }

    #[test]
    fn multiple_internal_actions_fire() -> eyre::Result<()> {
        let count = Arc::new(Mutex::new(0));
        let c1 = Arc::clone(&count);
        let c2 = Arc::clone(&count);
        let state = Arc::new(Mutex::new(()));
        let mut rep = StateRepresentation::<_, _, ()>::new(State::State1);
        rep.add_internal_action(Trigger::Trig, move |_, _| *c1.lock().unwrap() += 1);
        rep.add_internal_action(Trigger::Trig, move |_, _| *c2.lock().unwrap() += 1);
        rep.fire_internal_actions(
            &Transition::new(State::State1, Trigger::Trig, State::State1),
            Arc::clone(&state),
        );
        assert_eq!(*count.lock().unwrap(), 2, "trig should have fired twice");
        Ok(())
    }
}
