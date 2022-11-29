pub struct Transition<S, T> {
    pub source: S,
    pub destination: S,
    pub trigger: T,
}

impl<S, T> Transition<S, T>
where
    S: PartialEq,
{
    pub fn new(source: S, trigger: T, destination: S) -> Self {
        Self {
            source,
            destination,
            trigger,
        }
    }

    pub fn is_reentry(&self) -> bool {
        self.source == self.destination
    }
}
