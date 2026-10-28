//! `MessageScroller` decides when a message list follows new messages. It sticks to the
//! bottom until the reader scrolls up, then counts what they missed. The adapter reports
//! scroll distance and list changes; geometry and scrolling stay in the adapter.

use oxide_statechart::{Machine, Transition};

/// Props for [`MessageScrollerMachine`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MessageScrollerInput {
    /// How far from the bottom (px) still counts as "at the bottom".
    pub threshold: f64,
}

impl Default for MessageScrollerInput {
    fn default() -> Self {
        Self { threshold: 48.0 }
    }
}

/// What [`MessageScrollerMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessageScrollerState {
    /// The reader is at the bottom and the list follows new messages.
    pub stuck: bool,
    /// Messages added while the reader was scrolled up.
    pub unread: usize,
    /// Messages last reported.
    pub count: usize,
    /// Key of the newest message last reported.
    pub last_key: Option<String>,
}

/// Inputs to [`MessageScrollerMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum MessageScrollerEvent {
    /// The user scrolled; `from_bottom` is the distance left to the bottom in px.
    Scrolled {
        /// Distance to the bottom edge.
        from_bottom: f64,
    },
    /// The message list changed. A new `last_key` means messages were appended; the same
    /// key with a larger count means older messages were prepended.
    Items {
        /// Number of messages.
        count: usize,
        /// Key of the newest message.
        last_key: Option<String>,
    },
    /// The newest message changed size in place, such as a streaming reply.
    Grew,
    /// Go to the bottom and follow again.
    JumpToLatest,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageScrollerCommand {
    /// Scroll to the bottom.
    ScrollToBottom,
    /// Older messages were added above: keep the visible content where it was.
    KeepPosition,
}

/// Follow-the-bottom behaviour for a message list.
#[derive(Debug, Clone, Copy)]
pub struct MessageScrollerMachine;

impl Machine for MessageScrollerMachine {
    type Input = MessageScrollerInput;
    type State = MessageScrollerState;
    type Event = MessageScrollerEvent;
    type Command = MessageScrollerCommand;

    fn initial(_: &MessageScrollerInput) -> MessageScrollerState {
        MessageScrollerState { stuck: true, ..MessageScrollerState::default() }
    }

    fn transition(
        state: &MessageScrollerState,
        event: MessageScrollerEvent,
        input: &MessageScrollerInput,
    ) -> Transition<MessageScrollerState, MessageScrollerCommand> {
        let mut next = state.clone();
        let mut commands = Vec::new();
        match event {
            MessageScrollerEvent::Scrolled { from_bottom } => {
                next.stuck = from_bottom <= input.threshold;
                if next.stuck {
                    next.unread = 0;
                }
            }
            MessageScrollerEvent::Items { count, last_key } => {
                let appended = last_key != state.last_key;
                let grew = count > state.count;
                if appended {
                    if state.stuck {
                        commands.push(MessageScrollerCommand::ScrollToBottom);
                    } else {
                        next.unread += count.saturating_sub(state.count).max(1);
                    }
                } else if grew {
                    commands.push(MessageScrollerCommand::KeepPosition);
                }
                next.count = count;
                next.last_key = last_key;
            }
            MessageScrollerEvent::Grew => {
                if state.stuck {
                    commands.push(MessageScrollerCommand::ScrollToBottom);
                }
            }
            MessageScrollerEvent::JumpToLatest => {
                next.stuck = true;
                next.unread = 0;
                commands.push(MessageScrollerCommand::ScrollToBottom);
            }
        }
        Transition::with(next, commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(
        state: &MessageScrollerState,
        event: MessageScrollerEvent,
    ) -> oxide_statechart::Step<MessageScrollerState, MessageScrollerCommand> {
        MessageScrollerMachine::transition(state, event, &MessageScrollerInput::default()).resolve(state)
    }

    fn items(count: usize, key: &str) -> MessageScrollerEvent {
        MessageScrollerEvent::Items { count, last_key: Some(key.into()) }
    }

    #[test]
    fn follows_while_stuck() {
        let s = MessageScrollerMachine::initial(&MessageScrollerInput::default());
        let s = run(&s, items(1, "a")).state;
        let step = run(&s, items(2, "b"));
        assert_eq!(step.commands, vec![MessageScrollerCommand::ScrollToBottom]);
        assert_eq!(step.state.unread, 0);
    }

    #[test]
    fn counts_unread_when_scrolled_up_and_clears_on_return() {
        let s = MessageScrollerMachine::initial(&MessageScrollerInput::default());
        let s = run(&s, items(1, "a")).state;
        let s = run(&s, MessageScrollerEvent::Scrolled { from_bottom: 400.0 }).state;
        assert!(!s.stuck);
        let step = run(&s, items(3, "c"));
        assert!(step.commands.is_empty());
        assert_eq!(step.state.unread, 2);
        let s = run(&step.state, MessageScrollerEvent::Scrolled { from_bottom: 10.0 }).state;
        assert!(s.stuck);
        assert_eq!(s.unread, 0);
    }

    #[test]
    fn prepend_keeps_position() {
        let s = MessageScrollerMachine::initial(&MessageScrollerInput::default());
        let s = run(&s, items(2, "b")).state;
        let s = run(&s, MessageScrollerEvent::Scrolled { from_bottom: 400.0 }).state;
        let step = run(&s, items(5, "b"));
        assert_eq!(step.commands, vec![MessageScrollerCommand::KeepPosition]);
        assert_eq!(step.state.unread, 0);
        assert_eq!(step.state.count, 5);
    }

    #[test]
    fn jump_restores_following() {
        let s = MessageScrollerState { stuck: false, unread: 4, count: 9, last_key: None };
        let step = run(&s, MessageScrollerEvent::JumpToLatest);
        assert!(step.state.stuck);
        assert_eq!(step.state.unread, 0);
        assert_eq!(step.commands, vec![MessageScrollerCommand::ScrollToBottom]);
    }

    #[test]
    fn growth_only_follows_when_stuck() {
        let stuck = MessageScrollerState { stuck: true, ..MessageScrollerState::default() };
        assert_eq!(run(&stuck, MessageScrollerEvent::Grew).commands.len(), 1);
        let away = MessageScrollerState::default();
        assert!(run(&away, MessageScrollerEvent::Grew).commands.is_empty());
    }
}
