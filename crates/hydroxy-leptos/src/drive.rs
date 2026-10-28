//! The shared half of every component's `send`: run the machine, store the new state,
//! hand back the commands. Each component keeps its own `match` over its commands.

use hydroxy_statechart::{Machine, Transition};
use leptos::prelude::{RwSignal, Set, WithUntracked};

/// Feed `event` to machine `M`. If the state changed, store it in `state`. Returns the
/// commands to run, in order (empty when a signal was already disposed).
pub(crate) fn drive<M, I>(state: RwSignal<M::State>, input: I, event: M::Event) -> Vec<M::Command>
where
    M: Machine,
    M::State: PartialEq + Send + Sync + 'static,
    I: WithUntracked<Value = M::Input> + Copy,
{
    let Some(step) = input.try_with_untracked(|i| state.try_with_untracked(|s| M::transition(s, event, i))).flatten()
    else {
        return Vec::new();
    };
    if let Some(next) = step.state
        && state.with_untracked(|s| *s != next)
    {
        state.set(next);
    }
    step.commands
}

/// Like [`drive`], for callers whose input is not a signal (a `StoredValue`, a local):
/// `step` runs the machine against the current state and returns `None` when the input is gone.
pub(crate) fn drive_with<M>(
    state: RwSignal<M::State>,
    step: impl FnOnce(&M::State) -> Option<Transition<M::State, M::Command>>,
) -> Vec<M::Command>
where
    M: Machine,
    M::State: PartialEq + Send + Sync + 'static,
{
    let Some(step) = state.try_with_untracked(step).flatten() else {
        return Vec::new();
    };
    if let Some(next) = step.state
        && state.with_untracked(|s| *s != next)
    {
        state.set(next);
    }
    step.commands
}
