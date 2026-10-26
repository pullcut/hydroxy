//! Tooltip: opens on hover or keyboard focus, closes on pointer leave, blur or
//! Esc. The open delay lives in CSS (`transition-delay`), so the machine has no clock.

use oxide_statechart::{Machine, Transition};

/// Props for [`TooltipMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TooltipInput {
    /// Controlled open state. `Some` at start fixes controlled mode.
    pub open: Option<bool>,
    /// Uncontrolled initial open state.
    pub default_open: bool,
    /// Never open from hover or focus.
    pub disabled: bool,
}

/// What [`TooltipMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TooltipState {
    /// Whether the tooltip shows.
    pub open: bool,
    /// Fixed at start: the owner holds `open` and the machine only requests changes.
    pub controlled: bool,
}

/// Inputs to [`TooltipMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TooltipEvent {
    /// The pointer moved onto the trigger.
    PointerEnter,
    /// The pointer left the trigger.
    PointerLeave,
    /// The trigger received focus.
    Focus,
    /// The trigger lost focus.
    Blur,
    /// Esc pressed on the trigger.
    Escape,
}

/// Side effects from [`TooltipMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TooltipCommand {
    /// Tell the owner the open state changed (or should, when controlled).
    OpenChange(bool),
}

/// Open/closed state for one tooltip.
#[derive(Debug, Clone, Copy)]
pub struct TooltipMachine;

impl Machine for TooltipMachine {
    type Input = TooltipInput;
    type State = TooltipState;
    type Event = TooltipEvent;
    type Command = TooltipCommand;

    fn initial(input: &TooltipInput) -> TooltipState {
        TooltipState { open: input.open.unwrap_or(input.default_open), controlled: input.open.is_some() }
    }

    fn transition(
        state: &TooltipState,
        event: TooltipEvent,
        input: &TooltipInput,
    ) -> Transition<TooltipState, TooltipCommand> {
        let open = matches!(event, TooltipEvent::PointerEnter | TooltipEvent::Focus);
        if open == state.open || (open && input.disabled) {
            return Transition::stay(state.clone());
        }
        let mut next = state.clone();
        if !state.controlled {
            next.open = open;
        }
        Transition { state: next, commands: vec![TooltipCommand::OpenChange(open)] }
    }

    fn sync(state: &TooltipState, input: &TooltipInput) -> TooltipState {
        let mut next = state.clone();
        if let (true, Some(open)) = (state.controlled, input.open) {
            next.open = open;
        }
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(
        state: &TooltipState,
        event: TooltipEvent,
        input: &TooltipInput,
    ) -> Transition<TooltipState, TooltipCommand> {
        TooltipMachine::transition(state, event, input)
    }

    #[test]
    fn hover_and_focus_open_leave_and_blur_close() {
        let input = TooltipInput::default();
        let closed = TooltipMachine::initial(&input);
        for (open_with, close_with) in
            [(TooltipEvent::PointerEnter, TooltipEvent::PointerLeave), (TooltipEvent::Focus, TooltipEvent::Blur)]
        {
            let opened = run(&closed, open_with, &input);
            assert!(opened.state.open);
            assert_eq!(opened.commands, vec![TooltipCommand::OpenChange(true)]);
            let shut = run(&opened.state, close_with, &input);
            assert!(!shut.state.open);
            assert_eq!(shut.commands, vec![TooltipCommand::OpenChange(false)]);
        }
    }

    #[test]
    fn escape_closes_and_repeats_are_quiet() {
        let input = TooltipInput { default_open: true, ..Default::default() };
        let open = TooltipMachine::initial(&input);
        assert!(run(&open, TooltipEvent::PointerEnter, &input).commands.is_empty());
        let shut = run(&open, TooltipEvent::Escape, &input);
        assert!(!shut.state.open);
        assert!(run(&shut.state, TooltipEvent::Escape, &input).commands.is_empty());
    }

    #[test]
    fn disabled_never_opens() {
        let input = TooltipInput { disabled: true, ..Default::default() };
        let state = TooltipMachine::initial(&input);
        assert!(run(&state, TooltipEvent::PointerEnter, &input).commands.is_empty());
        assert!(run(&state, TooltipEvent::Focus, &input).commands.is_empty());
    }

    #[test]
    fn controlled_requests_but_waits_for_sync() {
        let mut input = TooltipInput { open: Some(false), ..Default::default() };
        let step = run(&TooltipMachine::initial(&input), TooltipEvent::Focus, &input);
        assert!(!step.state.open);
        assert_eq!(step.commands, vec![TooltipCommand::OpenChange(true)]);
        input.open = Some(true);
        assert!(TooltipMachine::sync(&step.state, &input).open);
    }
}
