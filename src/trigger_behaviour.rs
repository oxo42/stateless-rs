use std::{fmt::Debug, marker::PhantomData};

#[derive(Debug, Clone)]
pub(crate) enum TrigBehaviour<S, T> {
    Transitioning(TransitioningTriggerBehaviour<S, T>),
    Internal(InternalTransitioningTriggerBehaviour<S, T>),
}

pub trait TriggerBehaviour<S, T>: Debug
where
    S: Copy + Debug,
    T: Debug,
{
    // TODO: figure out how args should work
    fn fire(&self, source: S) -> S;
}

#[derive(Debug, Clone)]
pub struct TransitioningTriggerBehaviour<S, T> {
    trigger: T,
    destination: S,
}

impl<S, T> TransitioningTriggerBehaviour<S, T> {
    pub fn new(trigger: T, destination: S) -> Self {
        Self {
            trigger,
            destination,
        }
    }
}

impl<S, T> TriggerBehaviour<S, T> for TransitioningTriggerBehaviour<S, T>
where
    S: Copy + Debug,
    T: Debug,
{
    fn fire(&self, _source: S) -> S {
        self.destination
    }
}

#[derive(Debug, Clone)]
pub struct InternalTransitioningTriggerBehaviour<S, T> {
    trigger: T,
    phantom: PhantomData<S>,
}

impl<S, T> InternalTransitioningTriggerBehaviour<S, T> {
    pub fn new(trigger: T) -> Self {
        Self {
            trigger,
            phantom: PhantomData,
        }
    }
}

impl<S, T> TriggerBehaviour<S, T> for InternalTransitioningTriggerBehaviour<S, T>
where
    S: Copy + Debug,
    T: Debug,
{
    fn fire(&self, source: S) -> S {
        source
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{State, Trigger};

    #[test]
    fn transitioning_trigger_sends_to_destination() {
        let b = TransitioningTriggerBehaviour::new(Trigger::Trig, State::State1);
        assert_eq!(State::State1, b.fire(State::State1));
        assert_eq!(State::State1, b.fire(State::State2));
    }

    #[test]
    fn internal_trigger_sends_to_source() {
        let b = InternalTransitioningTriggerBehaviour::new(Trigger::Trig);
        assert_eq!(State::State1, b.fire(State::State1));
        assert_eq!(State::State2, b.fire(State::State2));
    }
}
