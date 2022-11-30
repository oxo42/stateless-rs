use std::fmt::Debug;

use crate::{transition, Transition};

type EventAction<S, T> = Box<dyn FnMut(&Transition<S, T>)>;

pub struct TransitionEventHandler<S, T> {
    pub(crate) events: Vec<EventAction<S, T>>,
}

impl<S, T> TransitionEventHandler<S, T> {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn add_event<F>(&mut self, f: F)
    where
        F: FnMut(&Transition<S, T>) + 'static,
    {
        self.events.push(Box::new(f));
    }

    pub fn fire_events(&mut self, transition: &Transition<S, T>) {
        for event in self.events.iter_mut() {
            event(transition);
        }
    }
}

impl<S, T> Default for TransitionEventHandler<S, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, T> Debug for TransitionEventHandler<S, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransitionEventHandler")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use strum_macros::EnumIter;

    use super::*;

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, EnumIter)]
    enum State {
        State1,
        State2,
    }

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    enum Trigger {
        Trig,
    }

    #[test]
    fn test_add_two_events_fires_both() {
        let mut handler = TransitionEventHandler::<State, Trigger>::new();
        let count = Arc::new(Mutex::new(0));
        let count1 = Arc::clone(&count);
        let count2 = Arc::clone(&count);
        handler.add_event(move |_t| {
            let mut data = count1.lock().unwrap();
            *data += 1;
        });
        handler.add_event(move |_t| {
            let mut data = count2.lock().unwrap();
            *data += 1;
        });
        let transition = Transition::new(State::State1, Trigger::Trig, State::State2);
        handler.fire_events(&transition);
        assert_eq!(*count.lock().unwrap(), 2);
    }
}
