use std::fmt::Debug;

pub trait TriggerBehaviour<S, T>: Debug
where
    S: Copy + Debug,
    T: Debug,
{
    // TODO: figure out how args should work
    fn fire(&self, source: S) -> S;
}

#[derive(Debug)]
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
