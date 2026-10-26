//! Tags input: a list of short text values plus a draft being typed.
//!
//! Enter or a comma commits the draft. Pasted text is split on commas and
//! newlines. With an empty draft, Backspace first highlights the last tag and
//! removes it on a second press; arrows move the highlight.

use oxide_statechart::{Machine, Transition};

/// Props for [`TagsInputMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagsInputInput {
    /// Controlled value. `Some` at start fixes controlled mode.
    pub value: Option<Vec<String>>,
    /// Uncontrolled initial value.
    pub default_value: Vec<String>,
    /// Most tags allowed. `None` for no limit.
    pub max: Option<usize>,
    /// Ignore edits.
    pub disabled: bool,
}

/// What [`TagsInputMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagsInputState {
    /// The committed tags.
    pub value: Vec<String>,
    /// Text typed but not yet committed.
    pub draft: String,
    /// Index of the tag picked by Backspace or the arrows.
    pub highlighted: Option<usize>,
    /// Fixed at start: the owner holds `value` and the machine only requests changes.
    pub controlled: bool,
}

/// Inputs to [`TagsInputMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagsInputEvent {
    /// The draft text changed. Typed commas commit what is before them;
    /// `pasted` commits every comma- or newline-separated piece.
    SetDraft {
        /// The draft input's new text.
        text: String,
        /// The change came from a paste.
        pasted: bool,
    },
    /// Enter: commit the draft.
    Commit,
    /// Backspace with an empty draft: highlight the last tag, or remove the highlighted one.
    Backspace,
    /// Delete: remove the highlighted tag.
    Delete,
    /// Arrow towards the start, with an empty draft.
    MovePrev,
    /// Arrow towards the end, with an empty draft. Past the last tag, back to the draft.
    MoveNext,
    /// A tag's remove button was pressed.
    Remove(usize),
    /// Focus left the field.
    Blur,
}

/// Why a candidate tag was not added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    /// Already in the list (case-insensitive).
    Duplicate,
    /// The list is at `max`.
    Limit,
}

/// Side effects from [`TagsInputMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagsInputCommand {
    /// Tell the owner the tags changed (or should, when controlled).
    ValueChange(Vec<String>),
    /// A tag was added. For the live region.
    Added(String),
    /// A tag was removed. For the live region.
    Removed(String),
    /// A candidate was not added. For the live region.
    Rejected(String, RejectReason),
    /// A tag was highlighted. For the live region.
    Highlighted(String),
}

/// Tags, the draft, and the Backspace highlight.
#[derive(Debug, Clone, Copy)]
pub struct TagsInputMachine;

type Step = Transition<TagsInputState, TagsInputCommand>;

impl Machine for TagsInputMachine {
    type Input = TagsInputInput;
    type State = TagsInputState;
    type Event = TagsInputEvent;
    type Command = TagsInputCommand;

    fn initial(input: &TagsInputInput) -> TagsInputState {
        TagsInputState {
            value: input.value.clone().unwrap_or_else(|| input.default_value.clone()),
            draft: String::new(),
            highlighted: None,
            controlled: input.value.is_some(),
        }
    }

    fn transition(state: &TagsInputState, event: TagsInputEvent, input: &TagsInputInput) -> Step {
        if input.disabled {
            return Transition::stay(state.clone());
        }
        let mut next = state.clone();
        match event {
            TagsInputEvent::SetDraft { text, pasted } => {
                next.highlighted = None;
                let mut pieces: Vec<&str> = text.split([',', '\n']).collect();
                // Typing keeps the part after the last comma as the draft; a paste commits it all.
                next.draft = if pasted { String::new() } else { pieces.pop().unwrap_or_default().to_string() };
                add(next, input, &pieces)
            }
            TagsInputEvent::Commit => {
                let draft = std::mem::take(&mut next.draft);
                let step = add(next, input, &[draft.as_str()]);
                // A rejected draft stays put so the user can fix it.
                if step.commands.iter().any(|c| matches!(c, TagsInputCommand::Rejected(..))) {
                    Transition { state: TagsInputState { draft, ..step.state }, commands: step.commands }
                } else {
                    step
                }
            }
            // Backspace and arrows only work on tags once the draft is empty.
            TagsInputEvent::Backspace | TagsInputEvent::MovePrev if !state.draft.is_empty() => Transition::stay(next),
            TagsInputEvent::Backspace => match state.highlighted {
                Some(i) => remove(next, i),
                None => highlight(next, state.value.len().checked_sub(1)),
            },
            TagsInputEvent::Delete => match state.highlighted {
                Some(i) => remove(next, i),
                None => Transition::stay(next),
            },
            TagsInputEvent::MovePrev => {
                let target = match state.highlighted {
                    Some(i) => Some(i.saturating_sub(1)),
                    None => state.value.len().checked_sub(1),
                };
                highlight(next, target)
            }
            TagsInputEvent::MoveNext => match state.highlighted {
                Some(i) if i + 1 < state.value.len() => highlight(next, Some(i + 1)),
                Some(_) => {
                    next.highlighted = None;
                    Transition::stay(next)
                }
                None => Transition::stay(next),
            },
            TagsInputEvent::Remove(i) => remove(next, i),
            TagsInputEvent::Blur => {
                next.highlighted = None;
                Transition::stay(next)
            }
        }
    }

    fn sync(state: &TagsInputState, input: &TagsInputInput) -> TagsInputState {
        let mut next = state.clone();
        if let (true, Some(value)) = (state.controlled, &input.value) {
            next.value.clone_from(value);
            next.highlighted = next.highlighted.filter(|&i| i < next.value.len());
        }
        next
    }
}

/// Add each trimmed, non-empty candidate unless it is a duplicate or over `max`.
fn add(mut state: TagsInputState, input: &TagsInputInput, candidates: &[&str]) -> Step {
    let mut value = state.value.clone();
    let mut commands = Vec::new();
    for candidate in candidates.iter().map(|c| c.trim()).filter(|c| !c.is_empty()) {
        let folded = candidate.to_lowercase();
        if value.iter().any(|v| v.to_lowercase() == folded) {
            commands.push(TagsInputCommand::Rejected(candidate.to_string(), RejectReason::Duplicate));
        } else if input.max.is_some_and(|max| value.len() >= max) {
            commands.push(TagsInputCommand::Rejected(candidate.to_string(), RejectReason::Limit));
        } else {
            value.push(candidate.to_string());
            commands.push(TagsInputCommand::Added(candidate.to_string()));
        }
    }
    if value.len() != state.value.len() {
        commands.insert(0, TagsInputCommand::ValueChange(value.clone()));
        if !state.controlled {
            state.value = value;
        }
    }
    Transition { state, commands }
}

fn remove(mut state: TagsInputState, index: usize) -> Step {
    state.highlighted = None;
    let Some(tag) = state.value.get(index).cloned() else {
        return Transition::stay(state);
    };
    let mut value = state.value.clone();
    value.remove(index);
    if !state.controlled {
        state.value.clone_from(&value);
    }
    Transition { state, commands: vec![TagsInputCommand::ValueChange(value), TagsInputCommand::Removed(tag)] }
}

fn highlight(mut state: TagsInputState, index: Option<usize>) -> Step {
    if index == state.highlighted {
        return Transition::stay(state);
    }
    state.highlighted = index;
    let commands =
        index.and_then(|i| state.value.get(i)).map(|t| TagsInputCommand::Highlighted(t.clone())).into_iter().collect();
    Transition { state, commands }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(tags: &[&str]) -> TagsInputInput {
        TagsInputInput { default_value: tags.iter().map(ToString::to_string).collect(), ..TagsInputInput::default() }
    }

    fn run(state: &TagsInputState, event: TagsInputEvent, input: &TagsInputInput) -> Step {
        TagsInputMachine::transition(state, event, input)
    }

    fn typed(text: &str) -> TagsInputEvent {
        TagsInputEvent::SetDraft { text: text.into(), pasted: false }
    }

    #[test]
    fn enter_commits_the_trimmed_draft() {
        let input = input(&[]);
        let state = run(&TagsInputMachine::initial(&input), typed("  billing "), &input).state;
        let step = run(&state, TagsInputEvent::Commit, &input);
        assert_eq!(step.state.value, ["billing"]);
        assert_eq!(step.state.draft, "");
        assert_eq!(step.commands[0], TagsInputCommand::ValueChange(vec!["billing".into()]));
    }

    #[test]
    fn a_typed_comma_commits_and_keeps_the_rest_as_draft() {
        let input = input(&[]);
        let step = run(&TagsInputMachine::initial(&input), typed("billing,ex"), &input);
        assert_eq!(step.state.value, ["billing"]);
        assert_eq!(step.state.draft, "ex");
    }

    #[test]
    fn paste_adds_every_piece_and_skips_duplicates_and_blanks() {
        let input = input(&["Billing"]);
        let event = TagsInputEvent::SetDraft { text: "billing, export,\nonboarding,,".into(), pasted: true };
        let step = run(&TagsInputMachine::initial(&input), event, &input);
        assert_eq!(step.state.value, ["Billing", "export", "onboarding"]);
        assert!(step.commands.contains(&TagsInputCommand::Rejected("billing".into(), RejectReason::Duplicate)));
    }

    #[test]
    fn max_is_respected_and_the_rejected_draft_stays() {
        let mut input = input(&["a", "b"]);
        input.max = Some(2);
        let state = run(&TagsInputMachine::initial(&input), typed("c"), &input).state;
        let step = run(&state, TagsInputEvent::Commit, &input);
        assert_eq!(step.state.value, ["a", "b"]);
        assert_eq!(step.state.draft, "c");
        assert_eq!(step.commands, [TagsInputCommand::Rejected("c".into(), RejectReason::Limit)]);
    }

    #[test]
    fn backspace_highlights_then_removes() {
        let input = input(&["a", "b"]);
        let first = run(&TagsInputMachine::initial(&input), TagsInputEvent::Backspace, &input);
        assert_eq!(first.state.highlighted, Some(1));
        let second = run(&first.state, TagsInputEvent::Backspace, &input);
        assert_eq!(second.state.value, ["a"]);
        assert_eq!(second.state.highlighted, None);
    }

    #[test]
    fn backspace_with_a_draft_does_nothing() {
        let input = input(&["a"]);
        let state = run(&TagsInputMachine::initial(&input), typed("x"), &input).state;
        assert_eq!(run(&state, TagsInputEvent::Backspace, &input).state, state);
    }

    #[test]
    fn arrows_move_the_highlight_and_leave_past_the_end() {
        let input = input(&["a", "b"]);
        let s = TagsInputMachine::initial(&input);
        let s = run(&s, TagsInputEvent::MovePrev, &input).state;
        assert_eq!(s.highlighted, Some(1));
        let s = run(&s, TagsInputEvent::MovePrev, &input).state;
        assert_eq!(s.highlighted, Some(0));
        let s = run(&s, TagsInputEvent::MoveNext, &input).state;
        let s = run(&s, TagsInputEvent::MoveNext, &input).state;
        assert_eq!(s.highlighted, None);
    }

    #[test]
    fn controlled_requests_and_waits_for_sync() {
        let mut input = TagsInputInput { value: Some(vec!["a".into()]), ..TagsInputInput::default() };
        let step = run(&TagsInputMachine::initial(&input), TagsInputEvent::Remove(0), &input);
        assert_eq!(step.state.value, ["a"]);
        assert_eq!(step.commands[0], TagsInputCommand::ValueChange(vec![]));
        input.value = Some(vec![]);
        assert!(TagsInputMachine::sync(&step.state, &input).value.is_empty());
    }

    #[test]
    fn disabled_ignores_everything() {
        let mut input = input(&["a"]);
        input.disabled = true;
        let state = TagsInputMachine::initial(&input);
        assert_eq!(run(&state, TagsInputEvent::Remove(0), &input).state, state);
    }
}
