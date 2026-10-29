//! Hover card: a rich preview that opens after the pointer rests on its
//! trigger, or straight away on keyboard focus. It stays open while the pointer
//! is over the trigger or the card, and closes a moment after it leaves both.
//!
//! The machine owns the delays but not the clock: it asks the adapter to
//! [`HoverCardCommand::Wait`] and acts on the matching [`HoverCardEvent::Elapsed`].
//! A newer wait, or any event that settles the state, makes an older one stale,
//! so adapters never need to cancel a timer.

use hydroxy_statechart::{Controllable, Machine, Transition};

/// Props for [`HoverCardMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverCardInput {
    /// Controlled open state. `Some` at start fixes controlled mode.
    pub open: Option<bool>,
    /// Uncontrolled initial open state.
    pub default_open: bool,
    /// Milliseconds the pointer must rest before the card opens. 0 opens at once.
    pub open_delay_ms: u32,
    /// Milliseconds after the pointer leaves before the card closes. 0 closes at once.
    pub close_delay_ms: u32,
}

impl Default for HoverCardInput {
    fn default() -> Self {
        Self { open: None, default_open: false, open_delay_ms: 300, close_delay_ms: 150 }
    }
}

/// A delayed open or close the machine is waiting on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pending {
    /// Matches the [`HoverCardCommand::Wait`] that started it.
    pub id: u32,
    /// Open (`true`) or close (`false`) when it elapses.
    pub open: bool,
}

/// What [`HoverCardMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HoverCardState {
    /// Whether the card shows.
    pub open: Controllable<bool>,
    /// The delayed change in flight, if any.
    pub pending: Option<Pending>,
    /// Last wait id handed out.
    pub seq: u32,
}

/// Inputs to [`HoverCardMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HoverCardEvent {
    /// The pointer moved onto the trigger or the card.
    PointerEnter,
    /// The pointer left the trigger or the card.
    PointerLeave,
    /// The trigger received keyboard focus.
    Focus,
    /// The trigger lost focus.
    Blur,
    /// Esc pressed.
    Escape,
    /// The wait with this id is over.
    Elapsed(u32),
}

/// Side effects from [`HoverCardMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HoverCardCommand {
    /// Tell the owner the open state changed (or should, when controlled).
    OpenChange(bool),
    /// Send [`HoverCardEvent::Elapsed`] with `id` after `ms` milliseconds.
    Wait {
        /// Echo this back in `Elapsed`.
        id: u32,
        /// How long to wait.
        ms: u32,
    },
}

/// Open/closed state and hover delays for one hover card.
#[derive(Debug, Clone, Copy)]
pub struct HoverCardMachine;

type Step = Transition<HoverCardState, HoverCardCommand>;

/// Drop any pending wait and move to `open`, requesting the change if it is one.
fn settle(state: &HoverCardState, open: bool) -> Step {
    let mut next = state.clone();
    next.pending = None;
    if open == state.open.value {
        return Transition::update(state, next, Vec::new());
    }
    next.open.set(open);
    Transition::update(state, next, vec![HoverCardCommand::OpenChange(open)])
}

/// Start a delayed change, replacing any other.
fn wait(state: &HoverCardState, open: bool, ms: u32) -> Step {
    if ms == 0 {
        return settle(state, open);
    }
    let id = state.seq.wrapping_add(1);
    let next = HoverCardState { seq: id, pending: Some(Pending { id, open }), ..state.clone() };
    Transition::update(state, next, vec![HoverCardCommand::Wait { id, ms }])
}

impl Machine for HoverCardMachine {
    type Input = HoverCardInput;
    type State = HoverCardState;
    type Event = HoverCardEvent;
    type Command = HoverCardCommand;

    fn initial(input: &HoverCardInput) -> HoverCardState {
        HoverCardState {
            open: Controllable::new(input.open.unwrap_or(input.default_open), input.open.is_some()),
            ..HoverCardState::default()
        }
    }

    fn transition(state: &HoverCardState, event: HoverCardEvent, input: &HoverCardInput) -> Step {
        match event {
            HoverCardEvent::PointerEnter if !state.open.value => wait(state, true, input.open_delay_ms),
            HoverCardEvent::PointerEnter | HoverCardEvent::Focus => settle(state, true),
            HoverCardEvent::PointerLeave if state.open.value => wait(state, false, input.close_delay_ms),
            HoverCardEvent::PointerLeave | HoverCardEvent::Blur | HoverCardEvent::Escape => settle(state, false),
            HoverCardEvent::Elapsed(id) => match state.pending {
                Some(p) if p.id == id => settle(state, p.open),
                // Superseded or settled: ignore.
                _ => Transition::stay(),
            },
        }
    }

    fn sync(state: &HoverCardState, input: &HoverCardInput) -> Option<HoverCardState> {
        let mut next = state.clone();
        next.open.sync(input.open);
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step as advance;

    fn waited(step: &hydroxy_statechart::Step<HoverCardState, HoverCardCommand>) -> u32 {
        match step.commands.as_slice() {
            [HoverCardCommand::Wait { id, .. }] => *id,
            other => panic!("expected one Wait, got {other:?}"),
        }
    }

    #[test]
    fn hover_opens_after_the_delay() {
        let input = HoverCardInput::default();
        let step =
            advance::<HoverCardMachine>(&HoverCardMachine::initial(&input), HoverCardEvent::PointerEnter, &input);
        assert!(!step.state.open.value);
        assert_eq!(step.commands, vec![HoverCardCommand::Wait { id: 1, ms: 300 }]);
        let opened = advance::<HoverCardMachine>(&step.state, HoverCardEvent::Elapsed(1), &input);
        assert!(opened.state.open.value);
        assert_eq!(opened.commands, vec![HoverCardCommand::OpenChange(true)]);
    }

    #[test]
    fn leaving_before_the_delay_cancels_the_open() {
        let input = HoverCardInput::default();
        let entered =
            advance::<HoverCardMachine>(&HoverCardMachine::initial(&input), HoverCardEvent::PointerEnter, &input);
        let left = advance::<HoverCardMachine>(&entered.state, HoverCardEvent::PointerLeave, &input);
        assert!(left.commands.is_empty());
        let late = advance::<HoverCardMachine>(&left.state, HoverCardEvent::Elapsed(waited(&entered)), &input);
        assert!(!late.state.open.value);
        assert!(late.commands.is_empty());
    }

    #[test]
    fn moving_from_trigger_to_card_keeps_it_open() {
        let input = HoverCardInput { default_open: true, ..Default::default() };
        let open = HoverCardMachine::initial(&input);
        let left = advance::<HoverCardMachine>(&open, HoverCardEvent::PointerLeave, &input);
        let close_id = waited(&left);
        // The pointer reaches the card before the close delay runs out.
        let entered = advance::<HoverCardMachine>(&left.state, HoverCardEvent::PointerEnter, &input);
        assert!(entered.commands.is_empty());
        let stale = advance::<HoverCardMachine>(&entered.state, HoverCardEvent::Elapsed(close_id), &input);
        assert!(stale.state.open.value);
        // Leaving the card for good closes it after the delay.
        let left = advance::<HoverCardMachine>(&stale.state, HoverCardEvent::PointerLeave, &input);
        let closed = advance::<HoverCardMachine>(&left.state, HoverCardEvent::Elapsed(waited(&left)), &input);
        assert!(!closed.state.open.value);
        assert_eq!(closed.commands, vec![HoverCardCommand::OpenChange(false)]);
    }

    #[test]
    fn focus_opens_at_once_and_blur_or_escape_close() {
        let input = HoverCardInput::default();
        let focused = advance::<HoverCardMachine>(&HoverCardMachine::initial(&input), HoverCardEvent::Focus, &input);
        assert!(focused.state.open.value);
        assert_eq!(focused.commands, vec![HoverCardCommand::OpenChange(true)]);
        for close in [HoverCardEvent::Blur, HoverCardEvent::Escape] {
            let step = advance::<HoverCardMachine>(&focused.state, close, &input);
            assert!(!step.state.open.value);
            assert_eq!(step.commands, vec![HoverCardCommand::OpenChange(false)]);
            assert!(advance::<HoverCardMachine>(&step.state, close, &input).commands.is_empty());
        }
    }

    #[test]
    fn zero_delays_act_immediately() {
        let input = HoverCardInput { open_delay_ms: 0, close_delay_ms: 0, ..Default::default() };
        let opened =
            advance::<HoverCardMachine>(&HoverCardMachine::initial(&input), HoverCardEvent::PointerEnter, &input);
        assert!(opened.state.open.value);
        assert!(!advance::<HoverCardMachine>(&opened.state, HoverCardEvent::PointerLeave, &input).state.open.value);
    }

    #[test]
    fn controlled_requests_but_waits_for_sync() {
        let mut input = HoverCardInput { open: Some(false), ..Default::default() };
        let step = advance::<HoverCardMachine>(&HoverCardMachine::initial(&input), HoverCardEvent::Focus, &input);
        assert!(!step.state.open.value);
        assert_eq!(step.commands, vec![HoverCardCommand::OpenChange(true)]);
        input.open = Some(true);
        assert!(hydroxy_statechart::sync_or_keep::<HoverCardMachine>(&step.state, &input).open.value);
    }
}
