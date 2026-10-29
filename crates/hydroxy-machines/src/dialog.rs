//! Dialog: one modal that opens and closes. The open flag is the only state,
//! controlled or uncontrolled. An alert dialog only closes through its own
//! buttons: Escape and backdrop clicks are ignored.

use hydroxy_statechart::{Controllable, Machine, Transition};

/// Props for [`DialogMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DialogInput {
    /// Controlled open state. `Some` at start fixes controlled mode.
    pub open: Option<bool>,
    /// Uncontrolled initial open state.
    pub default_open: bool,
    /// An alert dialog: Escape and backdrop clicks don't close it.
    pub alert: bool,
}

/// What [`DialogMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DialogState {
    /// Whether the dialog shows.
    pub open: Controllable<bool>,
}

/// What asked the dialog to close.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloseReason {
    /// The Escape key.
    Escape,
    /// A press on the backdrop, outside the panel.
    Outside,
    /// A close button inside the dialog.
    CloseTrigger,
}

/// Inputs to [`DialogMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogEvent {
    /// The trigger was activated.
    Open,
    /// Something asked the dialog to close.
    Close(CloseReason),
}

/// Side effects from [`DialogMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogCommand {
    /// Tell the owner the open state changed (or should, when controlled).
    OpenChange(bool),
}

/// Open/closed state for one modal dialog.
#[derive(Debug, Clone, Copy)]
pub struct DialogMachine;

type Step = Transition<DialogState, DialogCommand>;

impl Machine for DialogMachine {
    type Input = DialogInput;
    type State = DialogState;
    type Event = DialogEvent;
    type Command = DialogCommand;

    fn initial(input: &DialogInput) -> DialogState {
        DialogState { open: Controllable::new(input.open.unwrap_or(input.default_open), input.open.is_some()) }
    }

    fn transition(state: &DialogState, event: DialogEvent, input: &DialogInput) -> Step {
        match event {
            DialogEvent::Open if !state.open.value => request(state, true),
            DialogEvent::Close(reason) if state.open.value && (!input.alert || reason == CloseReason::CloseTrigger) => {
                request(state, false)
            }
            DialogEvent::Open | DialogEvent::Close(..) => Transition::stay(),
        }
    }

    fn sync(state: &DialogState, input: &DialogInput) -> Option<DialogState> {
        let mut next = state.clone();
        next.open.sync(input.open);
        hydroxy_statechart::changed(state, next)
    }
}

/// Commit (when uncontrolled) and always request the change.
fn request(state: &DialogState, open: bool) -> Step {
    let mut next = state.clone();
    next.open.set(open);
    Transition::update(state, next, vec![DialogCommand::OpenChange(open)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step as advance;

    #[test]
    fn opens_and_closes_when_uncontrolled() {
        let input = DialogInput::default();
        let opened = advance::<DialogMachine>(&DialogMachine::initial(&input), DialogEvent::Open, &input);
        assert!(opened.state.open.value);
        assert_eq!(opened.commands, vec![DialogCommand::OpenChange(true)]);
        let closed = advance::<DialogMachine>(&opened.state, DialogEvent::Close(CloseReason::Escape), &input);
        assert!(!closed.state.open.value);
        assert_eq!(closed.commands, vec![DialogCommand::OpenChange(false)]);
    }

    #[test]
    fn open_while_open_and_close_while_closed_do_nothing() {
        let input = DialogInput::default();
        let closed = DialogMachine::initial(&input);
        assert!(
            advance::<DialogMachine>(&closed, DialogEvent::Close(CloseReason::CloseTrigger), &input)
                .commands
                .is_empty()
        );
        let open = DialogState { open: Controllable::new(true, false) };
        assert!(advance::<DialogMachine>(&open, DialogEvent::Open, &input).commands.is_empty());
    }

    #[test]
    fn controlled_requests_but_waits_for_sync() {
        let mut input = DialogInput { open: Some(false), ..Default::default() };
        let step = advance::<DialogMachine>(&DialogMachine::initial(&input), DialogEvent::Open, &input);
        assert!(!step.state.open.value);
        assert_eq!(step.commands, vec![DialogCommand::OpenChange(true)]);
        input.open = Some(true);
        assert!(hydroxy_statechart::sync_or_keep::<DialogMachine>(&step.state, &input).open.value);
    }

    #[test]
    fn alert_only_closes_through_its_buttons() {
        let input = DialogInput { default_open: true, alert: true, ..Default::default() };
        let state = DialogMachine::initial(&input);
        assert!(advance::<DialogMachine>(&state, DialogEvent::Close(CloseReason::Escape), &input).commands.is_empty());
        assert!(advance::<DialogMachine>(&state, DialogEvent::Close(CloseReason::Outside), &input).commands.is_empty());
        assert!(
            !advance::<DialogMachine>(&state, DialogEvent::Close(CloseReason::CloseTrigger), &input).state.open.value
        );
    }
}
