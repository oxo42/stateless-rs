use std::time::{Duration, Instant};
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

///! Example of using the statemachine to power a phonecall
use stateless_rs::{StateMachine, StateMachineBuilder};
use strum_macros::EnumIter;

type PhoneStateMachine = StateMachine<State, Trigger, PhoneState>;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum Trigger {
    CallDialed,
    CallConnected,
    LeftMessage,
    PlacedOnHold,
    TakenOffHold,
    PhoneHurledAgainstWall,
    #[allow(dead_code)]
    MuteMicrophone,
    #[allow(dead_code)]
    UnmuteMicrophone,
    #[allow(dead_code)]
    SetVolume,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, EnumIter)]
enum State {
    OffHook,
    Ringing,
    Connected,
    OnHold,
    PhoneDestroyed,
}

fn build_statemachine(state: PhoneState) -> eyre::Result<PhoneStateMachine> {
    // TODO: the commented lines are things I need to do to get feature parity with
    // the PhoneCall.cs example in
    // https://github.com/dotnet-state-machine/stateless

    // NB: Specifying the PhoneState type here is needed to infer the closure
    // for on_entry and on_exit
    let mut builder: StateMachineBuilder<_, _, PhoneState> =
        StateMachineBuilder::new(State::OffHook);
    builder
        .config(State::OffHook)
        .permit(Trigger::CallDialed, State::Ringing);

    builder
        .config(State::Ringing)
        // .on_entry_from(setCalleeTrigger, |callee| on_dialled(callee), "caller number to call")
        .permit(Trigger::CallConnected, State::Connected);

    builder
        .config(State::Connected)
        .on_entry(|_transition, object| object.start_call())
        .on_exit(|_transition, object| object.end_call())
        // .internal_transition(Trigger::MuteMicrophone, |t| on_mute())
        // .internal_transition(Trigger::UnmuteMicrophone, |t| on_unmute())
        // .internal_transition(setVolumeTrigger, |volume, t| on_set_volume(t))
        .permit(Trigger::LeftMessage, State::OffHook)
        .permit(Trigger::PlacedOnHold, State::OnHold);

    builder
        .config(State::OnHold)
        // .substate_of(State::Connected)
        .permit(Trigger::TakenOffHold, State::Connected)
        .permit(Trigger::PhoneHurledAgainstWall, State::PhoneDestroyed);

    builder.on_transitioned(|t| {
        // TODO: parameters
        println!(
            "on_transitioned: {:?} -> {:?} via {:?}",
            t.source, t.destination, t.trigger
        )
    });

    let machine = builder.build(state)?;
    Ok(machine)
}

#[derive(Debug)]
struct PhoneState {
    call_start: Option<Instant>,
    call_duration: Option<Duration>,
}

impl PhoneState {
    fn start_call(&mut self) {
        self.call_start = Some(Instant::now());
    }

    fn end_call(&mut self) {
        let duration = Instant::now().duration_since(self.call_start.unwrap());
        self.call_duration = Some(duration);
    }
}

struct Phone {
    statemachine: PhoneStateMachine,
}

impl Display for Phone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Phone: {:?}", self.statemachine.object().lock().unwrap())
    }
}

impl Phone {
    fn new() -> eyre::Result<Self> {
        let state = PhoneState {
            call_start: None,
            call_duration: None,
        };
        Ok(Self {
            statemachine: build_statemachine(state)?,
        })
    }

    fn state(&self) -> Arc<Mutex<PhoneState>> {
        self.statemachine.object()
    }

    fn call(&mut self) -> eyre::Result<()> {
        self.statemachine.fire(Trigger::CallDialed)?;
        self.statemachine.fire(Trigger::CallConnected)?;
        println!("State: {:?}", self.statemachine.state());
        Ok(())
    }

    fn hangup(&mut self) -> eyre::Result<()> {
        self.statemachine.fire(Trigger::LeftMessage)?;
        println!("State: {:?}", self.statemachine.state());
        Ok(())
    }

    fn call_duration(&self) -> Duration {
        let duration = self.state().lock().unwrap().call_duration;
        duration.unwrap_or(Duration::default())
    }
}

fn main() -> eyre::Result<()> {
    let mut phone = Phone::new()?;
    println!("Phone: {}", phone);
    phone.call()?;
    println!("Phone: {}", phone);
    phone.hangup()?;
    println!("Phone: {}", phone);
    println!("Call duration: {:?}", phone.call_duration());
    Ok(())
}
