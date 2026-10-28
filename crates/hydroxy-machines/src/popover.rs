//! Popover: a non-modal panel anchored to a trigger. The open flag is the only
//! state, controlled or uncontrolled. Dismissal (Esc, outside click) is native
//! through the Popover API; the adapter reports it as [`PopoverEvent::Close`].

use hydroxy_statechart::{Machine, Transition};

/// Props for [`PopoverMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PopoverInput {
    /// Controlled open state. `Some` at start fixes controlled mode.
    pub open: Option<bool>,
    /// Uncontrolled initial open state.
    pub default_open: bool,
}

/// What [`PopoverMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PopoverState {
    /// Whether the panel is showing.
    pub open: bool,
    /// Fixed at start: the owner holds `open` and the machine only requests changes.
    pub controlled: bool,
}

/// Inputs to [`PopoverMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopoverEvent {
    /// The trigger was activated.
    Toggle,
    /// Close the panel. `restore_focus` sends focus back to the trigger
    /// (Esc and close buttons do; an outside click leaves focus where it landed).
    Close {
        /// Whether to focus the trigger afterwards.
        restore_focus: bool,
    },
}

/// Side effects from [`PopoverMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopoverCommand {
    /// Tell the owner the open state changed (or should, when controlled).
    OpenChange(bool),
    /// Move DOM focus back to the trigger.
    FocusTrigger,
}

/// Open/closed state for one popover.
#[derive(Debug, Clone, Copy)]
pub struct PopoverMachine;

type Step = Transition<PopoverState, PopoverCommand>;

/// Commit (when uncontrolled) and always request the change.
fn change(state: &PopoverState, open: bool) -> Step {
    let mut next = state.clone();
    if !state.controlled {
        next.open = open;
    }
    Transition::with(next, vec![PopoverCommand::OpenChange(open)])
}

impl Machine for PopoverMachine {
    type Input = PopoverInput;
    type State = PopoverState;
    type Event = PopoverEvent;
    type Command = PopoverCommand;

    fn initial(input: &PopoverInput) -> PopoverState {
        PopoverState { open: input.open.unwrap_or(input.default_open), controlled: input.open.is_some() }
    }

    fn transition(state: &PopoverState, event: PopoverEvent, _input: &PopoverInput) -> Step {
        match event {
            PopoverEvent::Toggle => change(state, !state.open),
            PopoverEvent::Close { .. } if !state.open => Transition::stay(),
            PopoverEvent::Close { restore_focus } => {
                let mut step = change(state, false);
                if restore_focus {
                    step.commands.push(PopoverCommand::FocusTrigger);
                }
                step
            }
        }
    }

    fn sync(state: &PopoverState, input: &PopoverInput) -> Option<PopoverState> {
        let mut next = state.clone();
        if let (true, Some(open)) = (state.controlled, input.open) {
            next.open = open;
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step;

    fn run(state: &PopoverState, event: PopoverEvent) -> hydroxy_statechart::Step<PopoverState, PopoverCommand> {
        step::<PopoverMachine>(state, event, &PopoverInput::default())
    }

    #[test]
    fn toggle_opens_and_closes_when_uncontrolled() {
        let opened = run(&PopoverState::default(), PopoverEvent::Toggle);
        assert!(opened.state.open);
        assert_eq!(opened.commands, vec![PopoverCommand::OpenChange(true)]);
        let closed = run(&opened.state, PopoverEvent::Toggle);
        assert!(!closed.state.open);
        assert_eq!(closed.commands, vec![PopoverCommand::OpenChange(false)]);
    }

    #[test]
    fn escape_close_returns_focus_but_outside_close_does_not() {
        let open = PopoverState { open: true, controlled: false };
        let esc = run(&open, PopoverEvent::Close { restore_focus: true });
        assert_eq!(esc.commands, vec![PopoverCommand::OpenChange(false), PopoverCommand::FocusTrigger]);
        let outside = run(&open, PopoverEvent::Close { restore_focus: false });
        assert_eq!(outside.commands, vec![PopoverCommand::OpenChange(false)]);
    }

    #[test]
    fn close_when_closed_does_nothing() {
        let step = run(&PopoverState::default(), PopoverEvent::Close { restore_focus: true });
        assert!(step.commands.is_empty());
    }

    #[test]
    fn controlled_requests_but_waits_for_sync() {
        let mut input = PopoverInput { open: Some(false), ..Default::default() };
        let step = {
            let start = &PopoverMachine::initial(&input);
            hydroxy_statechart::step::<PopoverMachine>(start, PopoverEvent::Toggle, &input)
        };
        assert!(!step.state.open);
        assert_eq!(step.commands, vec![PopoverCommand::OpenChange(true)]);
        input.open = Some(true);
        assert!(hydroxy_statechart::sync_or_keep::<PopoverMachine>(&step.state, &input).open);
    }
}
