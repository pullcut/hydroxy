//! The machine contract. A machine is pure: `initial`, `transition` and `sync`
//! return the next state plus commands for the adapter to run. No I/O, no framework.

/// The result of handling one event: the next state and the side effects the
/// adapter must run, in order.
#[derive(Debug, Clone, PartialEq)]
#[must_use = "a transition does nothing until the adapter applies its state and runs its commands"]
pub struct Transition<S, C> {
    /// The state after the event.
    pub state: S,
    /// Side effects for the adapter, in the order they should run.
    pub commands: Vec<C>,
}

impl<S, C> Transition<S, C> {
    /// No change and nothing to run.
    pub fn stay(state: S) -> Self {
        Self { state, commands: Vec::new() }
    }
}

/// A pure state machine. Adapters own the state and feed events in; the
/// machine never touches the DOM or a framework.
pub trait Machine {
    /// Props from the owner. Read on every transition, never mutated.
    type Input;
    /// What the machine remembers between events.
    type State: Clone;
    /// Things that happen to the machine (clicks, keys, focus).
    type Event;
    /// Side effects the adapter runs after a transition.
    type Command;

    /// The state before any event.
    #[must_use]
    fn initial(input: &Self::Input) -> Self::State;

    /// Handle one event.
    fn transition(
        state: &Self::State,
        event: Self::Event,
        input: &Self::Input,
    ) -> Transition<Self::State, Self::Command>;

    /// Re-align state after the adapter's input changed (e.g. a controlled value).
    #[must_use]
    fn sync(state: &Self::State, _input: &Self::Input) -> Self::State {
        state.clone()
    }
}
