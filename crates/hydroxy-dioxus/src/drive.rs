//! The shared half of every component's `send`: run the machine, store the new state,
//! hand back the commands. Each component keeps its own `match` over its commands.

use dioxus::prelude::*;
use hydroxy_statechart::Machine;

/// Feed `event` to machine `M`. If the state changed, store it in `state`. Returns the
/// commands to run, in order.
pub(crate) fn drive<M>(
    mut state: Signal<M::State>,
    input: impl Readable<Target = M::Input>,
    event: M::Event,
) -> Vec<M::Command>
where
    M: Machine,
    M::State: PartialEq + 'static,
    M::Input: 'static,
{
    let step = M::transition(&state.peek(), event, &input.peek());
    if let Some(next) = step.state
        && *state.peek() != next
    {
        state.set(next);
    }
    step.commands
}
