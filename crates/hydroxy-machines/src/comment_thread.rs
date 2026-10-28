//! `CommentThread` tracks which comment threads are resolved, which resolved threads are
//! expanded, and which comment has its reply box open. The comments belong to the owner.

use std::collections::BTreeSet;

use hydroxy_statechart::{Controllable, Machine, Transition};

/// Props for [`CommentThreadMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommentThreadInput {
    /// Controlled resolved thread ids. `Some` at start fixes controlled mode.
    pub resolved: Option<Vec<String>>,
    /// Uncontrolled initial resolved thread ids.
    pub default_resolved: Vec<String>,
}

/// What [`CommentThreadMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommentThreadState {
    /// Resolved thread ids.
    pub resolved: Controllable<BTreeSet<String>>,
    /// Resolved threads currently expanded.
    pub expanded: BTreeSet<String>,
    /// The thread whose reply box is open.
    pub replying: Option<String>,
}

impl CommentThreadState {
    /// Whether `id` is resolved.
    #[must_use]
    pub fn is_resolved(&self, id: &str) -> bool {
        self.resolved.value.contains(id)
    }

    /// Whether `id` shows its comments: open threads always do, resolved ones when expanded.
    #[must_use]
    pub fn is_open(&self, id: &str) -> bool {
        !self.is_resolved(id) || self.expanded.contains(id)
    }
}

/// Inputs to [`CommentThreadMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentThreadEvent {
    /// Mark a thread resolved.
    Resolve(String),
    /// Reopen a resolved thread.
    Reopen(String),
    /// Show or hide the comments of a resolved thread.
    ToggleExpanded(String),
    /// Open the reply box on a thread (closing any other).
    OpenReply(String),
    /// Close the reply box.
    CloseReply,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentThreadCommand {
    /// Tell the owner a thread was resolved or reopened (or should be, when controlled).
    ResolvedChange {
        /// The thread.
        id: String,
        /// The new (or proposed) value.
        resolved: bool,
    },
    /// Move focus into the reply box of a thread.
    FocusReply(String),
}

/// Resolve and reply state for a list of comment threads.
#[derive(Debug, Clone, Copy)]
pub struct CommentThreadMachine;

impl Machine for CommentThreadMachine {
    type Input = CommentThreadInput;
    type State = CommentThreadState;
    type Event = CommentThreadEvent;
    type Command = CommentThreadCommand;

    fn initial(input: &CommentThreadInput) -> CommentThreadState {
        let ids = input.resolved.as_ref().unwrap_or(&input.default_resolved);
        CommentThreadState {
            resolved: Controllable::new(ids.iter().cloned().collect(), input.resolved.is_some()),
            ..CommentThreadState::default()
        }
    }

    fn transition(
        state: &CommentThreadState,
        event: CommentThreadEvent,
        _: &CommentThreadInput,
    ) -> Transition<CommentThreadState, CommentThreadCommand> {
        let mut next = state.clone();
        let mut commands = Vec::new();
        match event {
            CommentThreadEvent::Resolve(id) => {
                if !state.is_resolved(&id) {
                    if state.replying.as_deref() == Some(&id) {
                        next.replying = None;
                    }
                    next.expanded.remove(&id);
                    let mut ids = state.resolved.value.clone();
                    ids.insert(id.clone());
                    next.resolved.set(ids);
                    commands.push(CommentThreadCommand::ResolvedChange { id, resolved: true });
                }
            }
            CommentThreadEvent::Reopen(id) => {
                if state.is_resolved(&id) {
                    next.expanded.remove(&id);
                    let mut ids = state.resolved.value.clone();
                    ids.remove(&id);
                    next.resolved.set(ids);
                    commands.push(CommentThreadCommand::ResolvedChange { id, resolved: false });
                }
            }
            CommentThreadEvent::ToggleExpanded(id) => {
                if !next.expanded.remove(&id) {
                    next.expanded.insert(id);
                }
            }
            CommentThreadEvent::OpenReply(id) => {
                next.replying = Some(id.clone());
                commands.push(CommentThreadCommand::FocusReply(id));
            }
            CommentThreadEvent::CloseReply => next.replying = None,
        }
        Transition::with(next, commands)
    }

    fn sync(state: &CommentThreadState, input: &CommentThreadInput) -> Option<CommentThreadState> {
        let mut next = state.clone();
        next.resolved.sync(input.resolved.as_ref().map(|ids| ids.iter().cloned().collect()));
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step as advance;

    fn run(
        s: &CommentThreadState,
        e: CommentThreadEvent,
    ) -> hydroxy_statechart::Step<CommentThreadState, CommentThreadCommand> {
        advance::<CommentThreadMachine>(s, e, &CommentThreadInput::default())
    }

    #[test]
    fn resolve_and_reopen_uncontrolled() {
        let s = CommentThreadMachine::initial(&CommentThreadInput::default());
        let t = run(&s, CommentThreadEvent::Resolve("a".into()));
        assert!(t.state.is_resolved("a"));
        assert!(!t.state.is_open("a"));
        assert_eq!(t.commands, vec![CommentThreadCommand::ResolvedChange { id: "a".into(), resolved: true }]);
        let t = run(&t.state, CommentThreadEvent::Reopen("a".into()));
        assert!(!t.state.is_resolved("a"));
        assert!(t.state.is_open("a"));
    }

    #[test]
    fn controlled_resolve_only_requests() {
        let input = CommentThreadInput { resolved: Some(vec![]), ..CommentThreadInput::default() };
        let s = CommentThreadMachine::initial(&input);
        let t = advance::<CommentThreadMachine>(&s, CommentThreadEvent::Resolve("a".into()), &input);
        assert!(!t.state.is_resolved("a"));
        assert_eq!(t.commands.len(), 1);
        let input = CommentThreadInput { resolved: Some(vec!["a".into()]), ..CommentThreadInput::default() };
        assert!(hydroxy_statechart::sync_or_keep::<CommentThreadMachine>(&t.state, &input).is_resolved("a"));
    }

    #[test]
    fn expanding_a_resolved_thread_opens_it() {
        let input = CommentThreadInput { default_resolved: vec!["a".into()], ..CommentThreadInput::default() };
        let s = CommentThreadMachine::initial(&input);
        assert!(!s.is_open("a"));
        let s = run(&s, CommentThreadEvent::ToggleExpanded("a".into())).state;
        assert!(s.is_open("a"));
        let s = run(&s, CommentThreadEvent::ToggleExpanded("a".into())).state;
        assert!(!s.is_open("a"));
    }

    #[test]
    fn one_reply_box_at_a_time_and_resolve_closes_it() {
        let s = CommentThreadMachine::initial(&CommentThreadInput::default());
        let s = run(&s, CommentThreadEvent::OpenReply("a".into())).state;
        let s = run(&s, CommentThreadEvent::OpenReply("b".into())).state;
        assert_eq!(s.replying.as_deref(), Some("b"));
        let s = run(&s, CommentThreadEvent::Resolve("b".into())).state;
        assert_eq!(s.replying, None);
    }
}
