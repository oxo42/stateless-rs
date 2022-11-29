use stateless_rs::StateMachineBuilder;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum State {
    On,
    Off,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum Trigger {
    Switch,
}

fn main() -> anyhow::Result<()> {
    let mut builder = StateMachineBuilder::new(State::Off);
    builder
        .config(State::Off)
        .permit(Trigger::Switch, State::On);
    builder
        .config(State::On)
        .permit(Trigger::Switch, State::Off);
    let mut machine = builder.build()?;

    println!("Machine: {}", machine);
    println!("Hitting switch");
    machine.fire(Trigger::Switch)?;
    println!("Machine: {}", machine);
    println!("Hitting switch");
    machine.fire(Trigger::Switch)?;
    println!("Machine: {}", machine);
    Ok(())
}
