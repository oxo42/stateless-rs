use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum StateMachineError<S, T> {
    #[error("state {state:?} not configured")]
    StateNotConfigured { state: S },
    #[error("trigger {trigger:?} not permitted for {state:?}")]
    TriggerNotPermitted { state: S, trigger: T },
    #[error("StateConfig for {state:?} still in use in Builder")]
    ConfigStillInUse { state: S },
    #[error("unknown StateMachine error")]
    Unknown,
}
