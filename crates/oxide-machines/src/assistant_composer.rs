//! `AssistantComposer` owns the text being written, a queue of messages written while a
//! reply is running, recall of earlier messages with the arrow keys, and the `@` / `/`
//! trigger popover. Sending, steering and cancelling are reported as commands; the owner
//! runs the model.

use oxide_statechart::{Machine, Transition};

/// Most earlier messages kept for recall.
const HISTORY_LIMIT: usize = 50;

/// Props for [`AssistantComposerMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AssistantComposerInput {
    /// A reply is running. Sends are queued until it ends.
    pub running: bool,
    /// The owner has attachments to send even when the text is blank.
    pub has_attachments: bool,
}

/// An open `@` or `/` popover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trigger {
    /// The character that opened it.
    pub char: char,
    /// Char index of the trigger character in the text.
    pub start: usize,
    /// Text typed after the trigger character.
    pub query: String,
}

/// A message waiting for the running reply to end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedMessage {
    /// Stable key.
    pub id: u64,
    /// The message.
    pub text: String,
}

/// What [`AssistantComposerMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AssistantComposerState {
    /// The text field.
    pub text: String,
    /// Caret position as a char index.
    pub caret: usize,
    /// Messages waiting for the reply to end, oldest first.
    pub queue: Vec<QueuedMessage>,
    /// Earlier sent messages, oldest first.
    pub history: Vec<String>,
    /// Position in `history` while recalling.
    pub recall: Option<usize>,
    /// Text typed before recall began, restored when recall ends.
    pub draft: String,
    /// The open popover.
    pub trigger: Option<Trigger>,
    /// Highlighted row in the popover.
    pub active: usize,
    /// Last id given to a queued message.
    pub next_id: u64,
}

/// Inputs to [`AssistantComposerMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssistantComposerEvent {
    /// The person edited the field. `caret` is a char index.
    SetText {
        /// New text.
        text: String,
        /// Caret after the edit.
        caret: usize,
    },
    /// Send, or queue when a reply is running.
    Send,
    /// Send now, interrupting the running reply.
    Steer,
    /// Stop the running reply.
    Cancel,
    /// The running reply ended: send the next queued message.
    RunFinished,
    /// Send a queued message now, interrupting the reply.
    SteerQueued(u64),
    /// Drop a queued message.
    RemoveQueued(u64),
    /// Recall the previous message.
    HistoryPrev,
    /// Recall the next message, or the draft after the newest.
    HistoryNext,
    /// Move the popover highlight down (`1`) or up (`-1`), wrapping within `count` rows.
    MoveActive {
        /// `1` or `-1`.
        delta: i32,
        /// Rows currently shown.
        count: usize,
    },
    /// Insert the picked label in place of the trigger text.
    AcceptTrigger(String),
    /// Close the popover.
    DismissTrigger,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssistantComposerCommand {
    /// Send this message now.
    Send(String),
    /// Send this message now and interrupt the running reply.
    Steer(String),
    /// A message was queued.
    Queued(u64),
    /// Stop the running reply.
    Cancel,
}

/// The text, queue, history and popover of an assistant composer.
#[derive(Debug, Clone, Copy)]
pub struct AssistantComposerMachine;

/// Find the trigger the caret is inside, if any. `@` opens anywhere at a word start; `/`
/// only at the very start of the text.
#[must_use]
pub fn detect_trigger(text: &str, caret: usize) -> Option<Trigger> {
    let chars: Vec<char> = text.chars().collect();
    let caret = caret.min(chars.len());
    let mut i = caret;
    while i > 0 {
        let c = chars[i - 1];
        if c.is_whitespace() {
            return None;
        }
        if c == '@' || c == '/' {
            let at_word_start = i == 1 || chars[i - 2].is_whitespace();
            let ok = if c == '/' { i == 1 } else { at_word_start };
            return ok.then(|| Trigger { char: c, start: i - 1, query: chars[i..caret].iter().collect() });
        }
        i -= 1;
    }
    None
}

/// Keep items whose label contains `query`, ignoring case. A blank query keeps all.
#[must_use]
pub fn trigger_matches<'a>(items: &'a [String], query: &str) -> Vec<&'a String> {
    let q = query.trim().to_lowercase();
    items.iter().filter(|i| q.is_empty() || i.to_lowercase().contains(&q)).collect()
}

fn retrigger(next: &mut AssistantComposerState) {
    next.trigger = detect_trigger(&next.text, next.caret);
    next.active = 0;
}

fn can_send(state: &AssistantComposerState, input: &AssistantComposerInput) -> bool {
    !state.text.trim().is_empty() || input.has_attachments
}

fn remember(next: &mut AssistantComposerState, text: &str) {
    if next.history.last().map(String::as_str) != Some(text) {
        next.history.retain(|h| h != text);
        next.history.push(text.to_string());
        let excess = next.history.len().saturating_sub(HISTORY_LIMIT);
        next.history.drain(..excess);
    }
    next.recall = None;
    next.draft.clear();
}

fn clear_field(next: &mut AssistantComposerState) {
    next.text.clear();
    next.caret = 0;
    next.trigger = None;
    next.active = 0;
}

impl Machine for AssistantComposerMachine {
    type Input = AssistantComposerInput;
    type State = AssistantComposerState;
    type Event = AssistantComposerEvent;
    type Command = AssistantComposerCommand;

    fn initial(_: &AssistantComposerInput) -> AssistantComposerState {
        AssistantComposerState::default()
    }

    #[allow(clippy::too_many_lines, reason = "one match over the events")]
    fn transition(
        state: &AssistantComposerState,
        event: AssistantComposerEvent,
        input: &AssistantComposerInput,
    ) -> Transition<AssistantComposerState, AssistantComposerCommand> {
        use AssistantComposerCommand as C;
        use AssistantComposerEvent as E;
        let mut next = state.clone();
        let mut commands = Vec::new();
        match event {
            E::SetText { text, caret } => {
                next.caret = caret.min(text.chars().count());
                next.text = text;
                next.recall = None;
                retrigger(&mut next);
            }
            E::Send => {
                if can_send(state, input) {
                    let text = state.text.trim().to_string();
                    remember(&mut next, &text);
                    clear_field(&mut next);
                    if input.running {
                        next.next_id += 1;
                        next.queue.push(QueuedMessage { id: next.next_id, text });
                        commands.push(C::Queued(next.next_id));
                    } else {
                        commands.push(C::Send(text));
                    }
                }
            }
            E::Steer => {
                if can_send(state, input) {
                    let text = state.text.trim().to_string();
                    remember(&mut next, &text);
                    clear_field(&mut next);
                    commands.push(if input.running { C::Steer(text) } else { C::Send(text) });
                }
            }
            E::Cancel => {
                if input.running {
                    commands.push(C::Cancel);
                }
            }
            E::RunFinished => {
                if !next.queue.is_empty() {
                    commands.push(C::Send(next.queue.remove(0).text));
                }
            }
            E::SteerQueued(id) => {
                if let Some(pos) = next.queue.iter().position(|q| q.id == id) {
                    commands.push(C::Steer(next.queue.remove(pos).text));
                }
            }
            E::RemoveQueued(id) => next.queue.retain(|q| q.id != id),
            E::HistoryPrev => {
                let at_start = state.recall.is_none();
                // Only start recalling from an empty field, so Up still moves the caret in real text.
                if state.history.is_empty() || (at_start && !state.text.trim().is_empty()) {
                    return Transition::with(next, commands);
                }
                if at_start {
                    next.draft.clone_from(&state.text);
                }
                let index = state.recall.map_or(state.history.len() - 1, |r| r.saturating_sub(1));
                next.recall = Some(index);
                next.text.clone_from(&state.history[index]);
                next.caret = next.text.chars().count();
                next.trigger = None;
            }
            E::HistoryNext => {
                if let Some(r) = state.recall {
                    if r + 1 < state.history.len() {
                        next.recall = Some(r + 1);
                        next.text.clone_from(&state.history[r + 1]);
                    } else {
                        next.recall = None;
                        next.text = std::mem::take(&mut next.draft);
                    }
                    next.caret = next.text.chars().count();
                    next.trigger = None;
                }
            }
            E::MoveActive { delta, count } => {
                if state.trigger.is_some() && count > 0 {
                    let count = i64::try_from(count).unwrap_or(i64::MAX);
                    let cur = i64::try_from(state.active).unwrap_or(0);
                    next.active = usize::try_from((cur + i64::from(delta)).rem_euclid(count)).unwrap_or(0);
                }
            }
            E::AcceptTrigger(label) => {
                if let Some(t) = &state.trigger {
                    let chars: Vec<char> = state.text.chars().collect();
                    let end = state.caret.min(chars.len());
                    let before: String = chars[..t.start].iter().collect();
                    let after: String = chars[end..].iter().collect();
                    let insert = format!("{}{label} ", t.char);
                    next.caret = before.chars().count() + insert.chars().count();
                    next.text = format!("{before}{insert}{after}");
                    next.trigger = None;
                    next.active = 0;
                }
            }
            E::DismissTrigger => {
                next.trigger = None;
                next.active = 0;
            }
        }
        Transition::with(next, commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use AssistantComposerCommand as C;
    use AssistantComposerEvent as E;

    fn run(
        s: &AssistantComposerState,
        e: AssistantComposerEvent,
        running: bool,
    ) -> oxide_statechart::Step<AssistantComposerState, AssistantComposerCommand> {
        AssistantComposerMachine::transition(s, e, &AssistantComposerInput { running, has_attachments: false }).resolve(s)
    }

    fn typed(s: &AssistantComposerState, text: &str) -> AssistantComposerState {
        run(s, E::SetText { text: text.into(), caret: text.chars().count() }, false).state
    }

    #[test]
    fn send_trims_clears_and_ignores_blank() {
        let s = AssistantComposerState::default();
        assert!(run(&typed(&s, "   "), E::Send, false).commands.is_empty());
        let t = run(&typed(&s, " hello "), E::Send, false);
        assert_eq!(t.commands, vec![C::Send("hello".into())]);
        assert!(t.state.text.is_empty());
    }

    #[test]
    fn attachments_allow_a_blank_send() {
        let s = AssistantComposerState::default();
        let input = AssistantComposerInput { running: false, has_attachments: true };
        let t = AssistantComposerMachine::transition(&s, E::Send, &input).resolve(&s);
        assert_eq!(t.commands, vec![C::Send(String::new())]);
    }

    #[test]
    fn sending_while_running_queues_then_drains_in_order() {
        let s = typed(&AssistantComposerState::default(), "first");
        let t = run(&s, E::Send, true);
        assert_eq!(t.commands, vec![C::Queued(1)]);
        let s = typed(&t.state, "second");
        let s = run(&s, E::Send, true).state;
        assert_eq!(s.queue.len(), 2);
        let t = run(&s, E::RunFinished, false);
        assert_eq!(t.commands, vec![C::Send("first".into())]);
        assert_eq!(t.state.queue.len(), 1);
    }

    #[test]
    fn steer_interrupts_only_when_running() {
        let s = typed(&AssistantComposerState::default(), "now");
        assert_eq!(run(&s, E::Steer, true).commands, vec![C::Steer("now".into())]);
        assert_eq!(run(&s, E::Steer, false).commands, vec![C::Send("now".into())]);
    }

    #[test]
    fn queued_messages_can_be_steered_or_removed() {
        let s = typed(&AssistantComposerState::default(), "a");
        let s = run(&s, E::Send, true).state;
        let s = typed(&s, "b");
        let s = run(&s, E::Send, true).state;
        let t = run(&s, E::SteerQueued(2), true);
        assert_eq!(t.commands, vec![C::Steer("b".into())]);
        let t = run(&t.state, E::RemoveQueued(1), true);
        assert!(t.state.queue.is_empty());
    }

    #[test]
    fn cancel_only_while_running() {
        let s = AssistantComposerState::default();
        assert!(run(&s, E::Cancel, false).commands.is_empty());
        assert_eq!(run(&s, E::Cancel, true).commands, vec![C::Cancel]);
    }

    #[test]
    fn history_walks_back_and_restores_the_draft() {
        let mut s = AssistantComposerState::default();
        for m in ["one", "two"] {
            s = typed(&s, m);
            s = run(&s, E::Send, false).state;
        }
        s = run(&s, E::HistoryPrev, false).state;
        assert_eq!(s.text, "two");
        s = run(&s, E::HistoryPrev, false).state;
        assert_eq!(s.text, "one");
        s = run(&s, E::HistoryPrev, false).state;
        assert_eq!(s.text, "one");
        s = run(&s, E::HistoryNext, false).state;
        assert_eq!(s.text, "two");
        s = run(&s, E::HistoryNext, false).state;
        assert_eq!(s.text, "");
        assert_eq!(s.recall, None);
    }

    #[test]
    fn history_does_not_start_over_typed_text() {
        let mut s = typed(&AssistantComposerState::default(), "one");
        s = run(&s, E::Send, false).state;
        s = typed(&s, "draft");
        assert_eq!(run(&s, E::HistoryPrev, false).state.text, "draft");
    }

    #[test]
    fn at_trigger_opens_at_word_start_only() {
        assert_eq!(detect_trigger("hi @al", 6), Some(Trigger { char: '@', start: 3, query: "al".into() }));
        assert_eq!(detect_trigger("a@b", 3), None);
        assert_eq!(detect_trigger("hi @al x", 8), None);
    }

    #[test]
    fn slash_trigger_opens_only_at_the_start() {
        assert_eq!(detect_trigger("/su", 3).map(|t| t.query), Some("su".into()));
        assert_eq!(detect_trigger("a /su", 5), None);
    }

    #[test]
    fn accepting_replaces_the_trigger_text() {
        let s = typed(&AssistantComposerState::default(), "ask @al");
        assert!(s.trigger.is_some());
        let s = run(&s, E::AcceptTrigger("alice".into()), false).state;
        assert_eq!(s.text, "ask @alice ");
        assert_eq!(s.caret, 11);
        assert!(s.trigger.is_none());
    }

    #[test]
    fn active_row_wraps() {
        let s = typed(&AssistantComposerState::default(), "@");
        let s = run(&s, E::MoveActive { delta: -1, count: 3 }, false).state;
        assert_eq!(s.active, 2);
        let s = run(&s, E::MoveActive { delta: 1, count: 3 }, false).state;
        assert_eq!(s.active, 0);
    }

    #[test]
    fn matches_ignore_case() {
        let items = vec!["Alice".to_string(), "Bob".to_string()];
        assert_eq!(trigger_matches(&items, "AL").len(), 1);
        assert_eq!(trigger_matches(&items, "").len(), 2);
    }
}
