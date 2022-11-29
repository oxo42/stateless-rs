use stateless_rs::{StateMachine, StateMachineBuilder};
use strum_macros::EnumIter;

///! Example of using the statemachine to power a phonecall
///     
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

fn build_statemachine() -> eyre::Result<StateMachine<State, Trigger>> {
    // the commented lines are things I need to do to get feature parity with
    // https://github.com/dotnet-state-machine/stateless
    let mut builder = StateMachineBuilder::new(State::OffHook);
    builder
        .config(State::OffHook)
        .permit(Trigger::CallDialed, State::Ringing);

    builder
        .config(State::Ringing)
        // .on_entry_from(setCalleeTrigger, |callee| on_dialled(callee), "caller number to call")
        .permit(Trigger::CallConnected, State::Connected);

    builder
        .config(State::Connected)
        // .on_entry(t => start_call_timer())
        // .on_exit(t => stop_call_timer())
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

    // builder.on_transitioned(|t| {
    //     println!(
    //         "on_transitioned: {} -> {} via {}({})",
    //         t.source,
    //         t.destination,
    //         t.trigger,
    //         join(t.parameters)
    //     )
    // });

    let machine = builder.build()?;
    Ok(machine)
}

fn main() -> eyre::Result<()> {
    let machine = build_statemachine();
    println!("{:?}", machine);
    Ok(())
}
