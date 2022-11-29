use stateless_rs::StateMachineBuilder;
use strum_macros::EnumIter;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, EnumIter)]
enum State {
    On,
    Off,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum Trigger {
    Switch,
}

#[test]
fn check_simple_machine_builds_and_works() -> eyre::Result<()> {
    let mut builder = StateMachineBuilder::new(State::Off);
    builder
        .config(State::Off)
        .permit(Trigger::Switch, State::On);
    builder
        .config(State::On)
        .permit(Trigger::Switch, State::Off);
    let mut machine = builder.build()?;

    assert_eq!(machine.state(), State::Off);
    machine.fire(Trigger::Switch)?;
    assert_eq!(machine.state(), State::On);
    machine.fire(Trigger::Switch)?;
    assert_eq!(machine.state(), State::Off);
    Ok(())
}