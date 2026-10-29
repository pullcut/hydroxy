//! `ThreadList` owns which conversation is selected, the search text, and which row is
//! being renamed. The conversations themselves belong to the owner: renames and deletes
//! are reported as commands for the owner to apply.

use hydroxy_statechart::{Machine, Transition};

/// Props for [`ThreadListMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThreadListInput {
    /// Controlled selection. `Some` at start fixes controlled mode.
    pub selected: Option<Option<String>>,
    /// Uncontrolled initial selection.
    pub default_selected: Option<String>,
}

/// What [`ThreadListMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThreadListState {
    /// The open conversation.
    pub selected: Option<String>,
    /// Fixed at start: the owner holds the selection.
    pub controlled: bool,
    /// Search text.
    pub query: String,
    /// The row being renamed, if any.
    pub renaming: Option<String>,
    /// Text in the rename field.
    pub draft: String,
}

/// Inputs to [`ThreadListMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreadListEvent {
    /// Open a conversation.
    Select(String),
    /// Set the search text.
    SetQuery(String),
    /// Start renaming `id`, beginning from its current title.
    StartRename {
        /// The conversation.
        id: String,
        /// Its current title.
        title: String,
    },
    /// Edit the rename field.
    SetDraft(String),
    /// Accept the rename. A blank title cancels instead.
    CommitRename,
    /// Leave the title as it was.
    CancelRename,
    /// Remove a conversation.
    Delete(String),
}

/// Side effects for the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreadListCommand {
    /// Tell the owner the selection changed (or should, when controlled).
    SelectedChange(Option<String>),
    /// The owner should give `id` this title.
    Rename {
        /// The conversation.
        id: String,
        /// The trimmed new title.
        title: String,
    },
    /// The owner should remove `id`.
    Delete(String),
}

/// Selection, search and rename state for a conversation list.
#[derive(Debug, Clone, Copy)]
pub struct ThreadListMachine;

impl Machine for ThreadListMachine {
    type Input = ThreadListInput;
    type State = ThreadListState;
    type Event = ThreadListEvent;
    type Command = ThreadListCommand;

    fn initial(input: &ThreadListInput) -> ThreadListState {
        ThreadListState {
            selected: input.selected.clone().unwrap_or_else(|| input.default_selected.clone()),
            controlled: input.selected.is_some(),
            ..ThreadListState::default()
        }
    }

    fn transition(
        state: &ThreadListState,
        event: ThreadListEvent,
        _: &ThreadListInput,
    ) -> Transition<ThreadListState, ThreadListCommand> {
        let mut next = state.clone();
        let mut commands = Vec::new();
        match event {
            ThreadListEvent::Select(id) => {
                if state.selected.as_deref() != Some(&id) {
                    commands.push(ThreadListCommand::SelectedChange(Some(id.clone())));
                    if !state.controlled {
                        next.selected = Some(id);
                    }
                }
            }
            ThreadListEvent::SetQuery(q) => next.query = q,
            ThreadListEvent::StartRename { id, title } => {
                next.renaming = Some(id);
                next.draft = title;
            }
            ThreadListEvent::SetDraft(d) => {
                if state.renaming.is_some() {
                    next.draft = d;
                }
            }
            ThreadListEvent::CommitRename => {
                if let Some(id) = next.renaming.take() {
                    let title = state.draft.trim().to_string();
                    if !title.is_empty() {
                        commands.push(ThreadListCommand::Rename { id, title });
                    }
                    next.draft.clear();
                }
            }
            ThreadListEvent::CancelRename => {
                next.renaming = None;
                next.draft.clear();
            }
            ThreadListEvent::Delete(id) => {
                if state.renaming.as_deref() == Some(&id) {
                    next.renaming = None;
                    next.draft.clear();
                }
                if state.selected.as_deref() == Some(&id) {
                    commands.push(ThreadListCommand::SelectedChange(None));
                    if !state.controlled {
                        next.selected = None;
                    }
                }
                commands.push(ThreadListCommand::Delete(id));
            }
        }
        Transition::update(state, next, commands)
    }

    fn sync(state: &ThreadListState, input: &ThreadListInput) -> Option<ThreadListState> {
        let mut next = state.clone();
        if let (true, Some(selected)) = (state.controlled, input.selected.clone()) {
            next.selected = selected;
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step;

    fn run(s: &ThreadListState, e: ThreadListEvent) -> hydroxy_statechart::Step<ThreadListState, ThreadListCommand> {
        step::<ThreadListMachine>(s, e, &ThreadListInput::default())
    }

    #[test]
    fn select_commits_when_uncontrolled() {
        let s = ThreadListMachine::initial(&ThreadListInput::default());
        let t = run(&s, ThreadListEvent::Select("a".into()));
        assert_eq!(t.state.selected.as_deref(), Some("a"));
        assert_eq!(t.commands, vec![ThreadListCommand::SelectedChange(Some("a".into()))]);
        assert!(run(&t.state, ThreadListEvent::Select("a".into())).commands.is_empty());
    }

    #[test]
    fn controlled_select_only_requests() {
        let input = ThreadListInput { selected: Some(None), ..ThreadListInput::default() };
        let s = ThreadListMachine::initial(&input);
        let t = step::<ThreadListMachine>(&s, ThreadListEvent::Select("a".into()), &input);
        assert_eq!(t.state.selected, None);
        assert_eq!(t.commands.len(), 1);
        let input = ThreadListInput { selected: Some(Some("a".into())), ..ThreadListInput::default() };
        assert_eq!(
            hydroxy_statechart::sync_or_keep::<ThreadListMachine>(&t.state, &input).selected.as_deref(),
            Some("a")
        );
    }

    #[test]
    fn rename_commits_trimmed_and_blank_cancels() {
        let s = ThreadListMachine::initial(&ThreadListInput::default());
        let s = run(&s, ThreadListEvent::StartRename { id: "a".into(), title: "Old".into() }).state;
        assert_eq!(s.draft, "Old");
        let s = run(&s, ThreadListEvent::SetDraft("  New title ".into())).state;
        let t = run(&s, ThreadListEvent::CommitRename);
        assert_eq!(t.commands, vec![ThreadListCommand::Rename { id: "a".into(), title: "New title".into() }]);
        assert!(t.state.renaming.is_none());

        let s = run(&s, ThreadListEvent::SetDraft("   ".into())).state;
        let t = run(&s, ThreadListEvent::CommitRename);
        assert!(t.commands.is_empty());
        assert!(t.state.renaming.is_none());
    }

    #[test]
    fn deleting_the_selected_row_clears_selection() {
        let input = ThreadListInput { default_selected: Some("a".into()), ..ThreadListInput::default() };
        let s = ThreadListMachine::initial(&input);
        let t = run(&s, ThreadListEvent::Delete("a".into()));
        assert_eq!(t.state.selected, None);
        assert_eq!(t.commands, vec![ThreadListCommand::SelectedChange(None), ThreadListCommand::Delete("a".into())]);
    }
}
