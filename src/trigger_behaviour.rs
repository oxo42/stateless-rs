use std::{fmt::Debug, marker::PhantomData};

#[derive(Debug, Clone)]
pub(crate) enum TriggerBehaviour<S, T> {
    Transitioning(Transitioning<S, T>),
    Internal(Internal<S, T>),
}

#[derive(Debug, Clone)]
pub struct Transitioning<S, T> {
    trigger: T,
    destination: S,
}

impl<S, T> Transitioning<S, T>
where
    S: Copy + Debug,
    T: Debug,
{
    pub fn new(trigger: T, destination: S) -> Self {
        Self {
            trigger,
            destination,
        }
    }

    pub fn fire(&self, _source: S) -> S {
        self.destination
    }
}

#[derive(Debug, Clone)]
pub struct Internal<S, T> {
    trigger: T,
    phantom: PhantomData<S>,
}

impl<S, T> Internal<S, T>
where
    S: Copy + Debug,
    T: Debug,
{
    pub fn new(trigger: T) -> Self {
        Self {
            trigger,
            phantom: PhantomData,
        }
    }

    pub fn fire(&self, source: S) -> S {
        source
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{State, Trigger};

    #[test]
    fn transitioning_trigger_sends_to_destination() {
        let b = Transitioning::new(Trigger::Trig, State::State1);
        assert_eq!(State::State1, b.fire(State::State1));
        assert_eq!(State::State1, b.fire(State::State2));
    }

    #[test]
    fn internal_trigger_sends_to_source() {
        let b = Internal::new(Trigger::Trig);
        assert_eq!(State::State1, b.fire(State::State1));
        assert_eq!(State::State2, b.fire(State::State2));
    }
}
