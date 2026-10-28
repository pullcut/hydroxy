//! Inline edit owns a value that reads as plain text until the user edits it.
//! Edit starts a draft, Enter or blur commits it, Esc throws it away.
//! Only a commit that changes the value reports a [`InlineEditCommand::Change`].

use hydroxy_statechart::{Controllable, Machine, Transition};

/// Props for [`InlineEditMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InlineEditInput {
    /// Controlled value. `Some` at start fixes controlled mode.
    pub value: Option<String>,
    /// Uncontrolled initial value.
    pub default_value: String,
    /// Block editing.
    pub disabled: bool,
}

/// What [`InlineEditMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InlineEditState {
    /// The committed value. Controlled when the owner holds it.
    pub value: Controllable<String>,
    /// The text in the field while editing.
    pub draft: String,
    /// The field is showing instead of the preview.
    pub editing: bool,
}

/// Inputs to [`InlineEditMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineEditEvent {
    /// Click, Enter or F2 on the preview.
    Edit,
    /// The field's text changed.
    Input(String),
    /// Enter in the field: commit and return focus to the preview.
    Commit,
    /// The field lost focus: commit, but leave focus where it went.
    Blur,
    /// Esc in the field.
    Cancel,
}

/// Side effects from [`InlineEditMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineEditCommand {
    /// The value changed (or should, when controlled).
    Change(String),
    /// Focus the field and select its text.
    FocusField,
    /// Focus the preview.
    FocusPreview,
}

/// Preview-to-field editing state.
#[derive(Debug, Clone, Copy)]
pub struct InlineEditMachine;

type Step = Transition<InlineEditState, InlineEditCommand>;

fn finish(state: &InlineEditState, refocus: bool) -> Step {
    let mut next = state.clone();
    next.editing = false;
    let mut commands = Vec::new();
    if state.draft != state.value.value {
        next.value.set(state.draft.clone());
        commands.push(InlineEditCommand::Change(state.draft.clone()));
    }
    next.draft.clone_from(&next.value.value);
    if refocus {
        commands.push(InlineEditCommand::FocusPreview);
    }
    Transition::with(next, commands)
}

impl Machine for InlineEditMachine {
    type Input = InlineEditInput;
    type State = InlineEditState;
    type Event = InlineEditEvent;
    type Command = InlineEditCommand;

    fn initial(input: &InlineEditInput) -> InlineEditState {
        let value = input.value.clone().unwrap_or_else(|| input.default_value.clone());
        InlineEditState { draft: value.clone(), value: Controllable::new(value, input.value.is_some()), editing: false }
    }

    fn transition(state: &InlineEditState, event: InlineEditEvent, input: &InlineEditInput) -> Step {
        match (event, state.editing) {
            (InlineEditEvent::Edit, false) if !input.disabled => {
                let mut next = state.clone();
                next.editing = true;
                next.draft.clone_from(&state.value.value);
                Transition::with(next, vec![InlineEditCommand::FocusField])
            }
            (InlineEditEvent::Input(text), true) => {
                let mut next = state.clone();
                next.draft = text;
                Transition::to(next)
            }
            (InlineEditEvent::Commit, true) => finish(state, true),
            (InlineEditEvent::Blur, true) => finish(state, false),
            (InlineEditEvent::Cancel, true) => {
                let mut next = state.clone();
                next.editing = false;
                next.draft.clone_from(&state.value.value);
                Transition::with(next, vec![InlineEditCommand::FocusPreview])
            }
            _ => Transition::stay(),
        }
    }

    fn sync(state: &InlineEditState, input: &InlineEditInput) -> Option<InlineEditState> {
        let mut next = state.clone();
        next.value.sync(input.value.clone());
        if state.value.controlled && !state.editing {
            next.draft.clone_from(&next.value.value);
        }
        if input.disabled {
            next.editing = false;
            next.draft.clone_from(&next.value.value);
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step as advance;

    fn input() -> InlineEditInput {
        InlineEditInput { default_value: "Q3 plan".into(), ..InlineEditInput::default() }
    }

    fn editing(text: &str, input: &InlineEditInput) -> InlineEditState {
        let s = advance::<InlineEditMachine>(&InlineEditMachine::initial(input), InlineEditEvent::Edit, input).state;
        advance::<InlineEditMachine>(&s, InlineEditEvent::Input(text.into()), input).state
    }

    #[test]
    fn edit_focuses_the_field() {
        let i = input();
        let step = advance::<InlineEditMachine>(&InlineEditMachine::initial(&i), InlineEditEvent::Edit, &i);
        assert!(step.state.editing);
        assert_eq!(step.commands, [InlineEditCommand::FocusField]);
    }

    #[test]
    fn disabled_cannot_edit() {
        let i = InlineEditInput { disabled: true, ..input() };
        let step = advance::<InlineEditMachine>(&InlineEditMachine::initial(&i), InlineEditEvent::Edit, &i);
        assert!(!step.state.editing);
    }

    #[test]
    fn commit_changes_value_and_refocuses() {
        let i = input();
        let step = advance::<InlineEditMachine>(&editing("Q4 plan", &i), InlineEditEvent::Commit, &i);
        assert!(!step.state.editing);
        assert_eq!(step.state.value.value, "Q4 plan");
        assert_eq!(step.commands, [InlineEditCommand::Change("Q4 plan".into()), InlineEditCommand::FocusPreview]);
    }

    #[test]
    fn blur_commits_without_stealing_focus() {
        let i = input();
        let step = advance::<InlineEditMachine>(&editing("Q4 plan", &i), InlineEditEvent::Blur, &i);
        assert_eq!(step.commands, [InlineEditCommand::Change("Q4 plan".into())]);
    }

    #[test]
    fn unchanged_commit_reports_nothing() {
        let i = input();
        let step = advance::<InlineEditMachine>(&editing("Q3 plan", &i), InlineEditEvent::Blur, &i);
        assert!(step.commands.is_empty());
    }

    #[test]
    fn cancel_restores_the_value() {
        let i = input();
        let step = advance::<InlineEditMachine>(&editing("junk", &i), InlineEditEvent::Cancel, &i);
        assert_eq!(step.state.value.value, "Q3 plan");
        assert_eq!(step.state.draft, "Q3 plan");
        assert_eq!(step.commands, [InlineEditCommand::FocusPreview]);
        // A blur right after the cancel has nothing left to commit.
        assert!(advance::<InlineEditMachine>(&step.state, InlineEditEvent::Blur, &i).commands.is_empty());
    }

    #[test]
    fn controlled_waits_for_the_owner() {
        let i = InlineEditInput { value: Some("A".into()), ..InlineEditInput::default() };
        let step = advance::<InlineEditMachine>(&editing("B", &i), InlineEditEvent::Commit, &i);
        assert_eq!(step.state.value.value, "A");
        assert_eq!(step.commands[0], InlineEditCommand::Change("B".into()));
        let i2 = InlineEditInput { value: Some("B".into()), ..InlineEditInput::default() };
        assert_eq!(hydroxy_statechart::sync_or_keep::<InlineEditMachine>(&step.state, &i2).value.value, "B");
    }

    #[test]
    fn disabling_while_editing_closes_the_field() {
        let i = input();
        let off = InlineEditInput { disabled: true, ..input() };
        let s = hydroxy_statechart::sync_or_keep::<InlineEditMachine>(&editing("x", &i), &off);
        assert!(!s.editing);
    }
}
