//! Collapsible: one disclosure that opens and closes. The open flag is the
//! only state, controlled or uncontrolled.

use hydroxy_statechart::{Controllable, Machine, Transition};

/// Props for [`CollapsibleMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CollapsibleInput {
    /// Controlled open state. `Some` at start fixes controlled mode.
    pub open: Option<bool>,
    /// Uncontrolled initial open state.
    pub default_open: bool,
    /// Ignore toggles.
    pub disabled: bool,
}

/// What [`CollapsibleMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CollapsibleState {
    /// Whether the content shows. Controlled when the owner holds it: the machine then
    /// only requests changes.
    pub open: Controllable<bool>,
}

/// Inputs to [`CollapsibleMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollapsibleEvent {
    /// The trigger was activated.
    Toggle,
}

/// Side effects from [`CollapsibleMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollapsibleCommand {
    /// Tell the owner the open state changed (or should, when controlled).
    OpenChange(bool),
}

/// Open/closed state for one disclosure.
#[derive(Debug, Clone, Copy)]
pub struct CollapsibleMachine;

impl Machine for CollapsibleMachine {
    type Input = CollapsibleInput;
    type State = CollapsibleState;
    type Event = CollapsibleEvent;
    type Command = CollapsibleCommand;

    fn initial(input: &CollapsibleInput) -> CollapsibleState {
        CollapsibleState { open: Controllable::new(input.open.unwrap_or(input.default_open), input.open.is_some()) }
    }

    fn transition(
        state: &CollapsibleState,
        event: CollapsibleEvent,
        input: &CollapsibleInput,
    ) -> Transition<CollapsibleState, CollapsibleCommand> {
        match event {
            CollapsibleEvent::Toggle if input.disabled => Transition::stay(),
            CollapsibleEvent::Toggle => {
                let open = !state.open.value;
                let mut next = state.clone();
                next.open.set(open);
                Transition::update(state, next, vec![CollapsibleCommand::OpenChange(open)])
            }
        }
    }

    fn sync(state: &CollapsibleState, input: &CollapsibleInput) -> Option<CollapsibleState> {
        let mut next = state.clone();
        next.open.sync(input.open);
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step as advance;

    #[test]
    fn toggle_commits_when_uncontrolled() {
        let input = CollapsibleInput::default();
        let step =
            advance::<CollapsibleMachine>(&CollapsibleMachine::initial(&input), CollapsibleEvent::Toggle, &input);
        assert!(step.state.open.value);
        assert_eq!(step.commands, vec![CollapsibleCommand::OpenChange(true)]);
    }

    #[test]
    fn controlled_requests_but_waits_for_sync() {
        let mut input = CollapsibleInput { open: Some(false), ..Default::default() };
        let step =
            advance::<CollapsibleMachine>(&CollapsibleMachine::initial(&input), CollapsibleEvent::Toggle, &input);
        assert!(!step.state.open.value);
        assert_eq!(step.commands, vec![CollapsibleCommand::OpenChange(true)]);
        input.open = Some(true);
        assert!(hydroxy_statechart::sync_or_keep::<CollapsibleMachine>(&step.state, &input).open.value);
    }

    #[test]
    fn disabled_ignores_toggle() {
        let input = CollapsibleInput { default_open: true, disabled: true, ..Default::default() };
        let step =
            advance::<CollapsibleMachine>(&CollapsibleMachine::initial(&input), CollapsibleEvent::Toggle, &input);
        assert!(step.state.open.value);
        assert!(step.commands.is_empty());
    }
}
