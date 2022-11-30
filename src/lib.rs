#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
mod builder;
mod state_config;
mod state_machine;
mod state_representation;
mod statemachine_error;
mod transition;
mod transition_event;
mod trigger_behaviour;

pub use builder::StateMachineBuilder;
pub use state_machine::StateMachine;
pub use statemachine_error::StateMachineError;
pub use transition::Transition;
pub use transition_event::TransitionEventHandler;

#[cfg(test)]
mod tests {
    use strum_macros::EnumIter;

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, EnumIter)]
    pub enum State {
        State1,
        State2,
    }

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    pub enum Trigger {
        Trig,
        Trig2,
    }
}
