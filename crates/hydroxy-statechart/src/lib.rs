//! The machine contract. A machine is pure: `initial`, `transition` and `sync`
//! return the next state (if it changed) plus commands for the adapter to run.
//! No I/O, no framework.

/// The result of handling one event: the next state, if it changed, and the side
/// effects the adapter must run, in order.
#[derive(Debug, Clone, PartialEq)]
#[must_use = "a transition does nothing until the adapter applies its state and runs its commands"]
pub struct Transition<S, C> {
    /// The state after the event, or `None` when the event changed nothing. Adapters
    /// only store (and so only notify subscribers) when this is `Some`.
    pub state: Option<S>,
    /// Side effects for the adapter, in the order they should run.
    pub commands: Vec<C>,
}

impl<S, C> Transition<S, C> {
    /// No change and nothing to run.
    pub fn stay() -> Self {
        Self { state: None, commands: Vec::new() }
    }

    /// Move to `state` with nothing to run.
    pub fn to(state: S) -> Self {
        Self { state: Some(state), commands: Vec::new() }
    }

    /// Move to `state`, then run `commands`.
    pub fn with(state: S, commands: Vec<C>) -> Self {
        Self { state: Some(state), commands }
    }

    /// Keep the state and run `commands`.
    pub fn emit(commands: Vec<C>) -> Self {
        Self { state: None, commands }
    }

    /// The state to hold after this transition: the new one, or a copy of `current`
    /// when nothing changed. For tests and for machines that compose other machines.
    pub fn resolve(self, current: &S) -> Step<S, C>
    where
        S: Clone,
    {
        Step { state: self.state.unwrap_or_else(|| current.clone()), commands: self.commands }
    }
}

/// A transition with its state resolved against the state it started from. See
/// [`Transition::resolve`].
#[derive(Debug, Clone, PartialEq)]
pub struct Step<S, C> {
    /// The state after the event, changed or not.
    pub state: S,
    /// Side effects for the adapter, in the order they should run.
    pub commands: Vec<C>,
}

/// `Some(after)` when it differs from `before`, else `None`. For `sync` impls that
/// build a candidate state and only want to report a real change.
pub fn changed<S: PartialEq>(before: &S, after: S) -> Option<S> {
    (after != *before).then_some(after)
}

/// `M::sync`, keeping `state` when it is already aligned. For tests.
pub fn sync_or_keep<M: Machine>(state: &M::State, input: &M::Input) -> M::State
where
    M::State: Clone,
{
    M::sync(state, input).unwrap_or_else(|| state.clone())
}

/// One event through `M::transition`, with the state resolved against `state`. For
/// tests, so modules do not each carry a `run` helper.
pub fn step<M: Machine>(state: &M::State, event: M::Event, input: &M::Input) -> Step<M::State, M::Command>
where
    M::State: Clone,
{
    M::transition(state, event, input).resolve(state)
}

/// A value that is either owned by the machine (uncontrolled) or held by the owner
/// (controlled). Replaces the `controlled` flag and sync step each machine repeated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Controllable<T> {
    /// The current value.
    pub value: T,
    /// The owner holds the value: [`set`](Self::set) does not change it.
    pub controlled: bool,
}

impl<T> Controllable<T> {
    /// Start at `value`; `controlled` fixes whether the owner holds it.
    pub fn new(value: T, controlled: bool) -> Self {
        Self { value, controlled }
    }

    /// Commit `value` when uncontrolled. Returns whether it was committed; a controlled
    /// value only changes through [`sync`](Self::sync), after the owner accepts the request.
    pub fn set(&mut self, value: T) -> bool {
        if self.controlled {
            return false;
        }
        self.value = value;
        true
    }

    /// Take the owner's value, if it gave one. `None` (uncontrolled) leaves the value alone.
    pub fn sync(&mut self, external: Option<T>) {
        if let Some(value) = external {
            self.value = value;
        }
    }
}

/// A pure state machine. Adapters own the state and feed events in; the
/// machine never touches the DOM or a framework.
pub trait Machine {
    /// Props from the owner. Read on every transition, never mutated.
    type Input;
    /// What the machine remembers between events.
    type State;
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
    /// `None` means the state is already aligned.
    #[must_use]
    fn sync(_state: &Self::State, _input: &Self::Input) -> Option<Self::State> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{Controllable, Machine, Transition, changed, step};

    struct Counter;

    impl Machine for Counter {
        type Input = ();
        type State = u32;
        type Event = bool;
        type Command = ();

        fn initial((): &()) -> u32 {
            0
        }

        fn transition(state: &u32, bump: bool, (): &()) -> Transition<u32, ()> {
            if bump { Transition::to(state + 1) } else { Transition::stay() }
        }
    }

    #[test]
    fn a_no_op_event_reports_no_state() {
        let step = Counter::transition(&3, false, &());
        assert_eq!(step.state, None);
        assert_eq!(step.commands, Vec::<()>::new());
        assert_eq!(step.resolve(&3).state, 3);
    }

    #[test]
    fn a_change_reports_the_new_state() {
        assert_eq!(Counter::transition(&3, true, &()).state, Some(4));
    }

    #[test]
    fn sync_defaults_to_no_change_and_changed_filters_equal_states() {
        assert_eq!(Counter::sync(&3, &()), None);
        assert_eq!(changed(&3, 3), None);
        assert_eq!(changed(&3, 4), Some(4));
    }

    #[test]
    fn step_resolves_against_the_start_state() {
        assert_eq!(step::<Counter>(&3, false, &()).state, 3);
        assert_eq!(step::<Counter>(&3, true, &()).state, 4);
    }

    #[test]
    fn controllable_commits_only_when_uncontrolled() {
        let mut free = Controllable::new(1, false);
        assert!(free.set(2));
        assert_eq!(free.value, 2);
        let mut held = Controllable::new(1, true);
        assert!(!held.set(2));
        assert_eq!(held.value, 1);
        held.sync(Some(5));
        assert_eq!(held.value, 5);
        held.sync(None);
        assert_eq!(held.value, 5);
    }
}
