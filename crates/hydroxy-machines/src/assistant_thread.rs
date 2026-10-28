//! Small machines behind an assistant thread's message parts: the branch picker, the
//! per-message action bar and the quote-selection toolbar. They keep no message data;
//! the owner holds the messages and applies the commands.

use hydroxy_statechart::{Controllable, Machine, Transition};

// ---------------------------------------------------------------- branch picker

/// Props for [`BranchPickerMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BranchPickerInput {
    /// How many branches the message has.
    pub count: usize,
    /// Controlled position. `Some` at start fixes controlled mode.
    pub index: Option<usize>,
    /// A reply is running.
    pub running: bool,
    /// Allow switching while a reply runs.
    pub switch_during_run: bool,
}

impl Default for BranchPickerInput {
    fn default() -> Self {
        Self { count: 1, index: None, running: false, switch_during_run: false }
    }
}

/// What [`BranchPickerMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BranchPickerState {
    /// Zero-based position of the shown branch.
    pub index: Controllable<usize>,
}

/// Inputs to [`BranchPickerMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchPickerEvent {
    /// Show the branch before this one.
    Previous,
    /// Show the branch after this one.
    Next,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchPickerCommand {
    /// Tell the owner to show this branch (zero-based).
    Switch(usize),
}

/// Steps through the branches of one message.
#[derive(Debug, Clone, Copy)]
pub struct BranchPickerMachine;

/// Whether Previous is enabled.
#[must_use]
pub fn can_previous(state: &BranchPickerState, input: &BranchPickerInput) -> bool {
    state.index.value > 0 && (!input.running || input.switch_during_run)
}

/// Whether Next is enabled.
#[must_use]
pub fn can_next(state: &BranchPickerState, input: &BranchPickerInput) -> bool {
    state.index.value + 1 < input.count && (!input.running || input.switch_during_run)
}

impl Machine for BranchPickerMachine {
    type Input = BranchPickerInput;
    type State = BranchPickerState;
    type Event = BranchPickerEvent;
    type Command = BranchPickerCommand;

    fn initial(input: &BranchPickerInput) -> BranchPickerState {
        BranchPickerState {
            index: Controllable::new(
                input.index.unwrap_or(0).min(input.count.saturating_sub(1)),
                input.index.is_some(),
            ),
        }
    }

    fn transition(
        state: &BranchPickerState,
        event: BranchPickerEvent,
        input: &BranchPickerInput,
    ) -> Transition<BranchPickerState, BranchPickerCommand> {
        let mut next = state.clone();
        let mut commands = Vec::new();
        let target = match event {
            BranchPickerEvent::Previous if can_previous(state, input) => Some(state.index.value - 1),
            BranchPickerEvent::Next if can_next(state, input) => Some(state.index.value + 1),
            _ => None,
        };
        if let Some(i) = target {
            commands.push(BranchPickerCommand::Switch(i));
            next.index.set(i);
        }
        Transition::with(next, commands)
    }

    fn sync(state: &BranchPickerState, input: &BranchPickerInput) -> Option<BranchPickerState> {
        let mut next = state.clone();
        next.index.sync(input.index);
        next.index.value = next.index.value.min(input.count.saturating_sub(1));
        hydroxy_statechart::changed(state, next)
    }
}

// ------------------------------------------------------------------- action bar

/// A thumbs vote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feedback {
    /// Thumbs up.
    Positive,
    /// Thumbs down.
    Negative,
}

/// What [`ActionBarMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActionBarState {
    /// The copy button shows its confirmation.
    pub copied: bool,
    /// The vote already given.
    pub feedback: Option<Feedback>,
    /// The message is being read aloud.
    pub speaking: bool,
}

/// Inputs to [`ActionBarMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionBarEvent {
    /// Copy the message.
    Copy,
    /// The copy confirmation timed out.
    CopiedReset,
    /// Vote. Voting the same way again takes it back.
    Vote(Feedback),
    /// Ask for the answer again.
    Reload,
    /// Edit the message.
    Edit,
    /// Export the message as Markdown.
    Export,
    /// Start or stop reading aloud.
    ToggleSpeak,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionBarCommand {
    /// Write the message text to the clipboard, then send `CopiedReset` after a moment.
    Copy,
    /// Report the vote (`None` when taken back).
    Feedback(Option<Feedback>),
    /// Run the owner's reload.
    Reload,
    /// Run the owner's edit.
    Edit,
    /// Hand the owner the Markdown export.
    Export,
    /// Start reading aloud.
    Speak,
    /// Stop reading aloud.
    StopSpeaking,
}

/// Which messages show the action bar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Autohide {
    /// Always visible.
    #[default]
    Never,
    /// Hidden until the message is hovered or focused.
    Always,
    /// Visible only on the last message; others show on hover.
    NotLast,
}

/// Props for [`ActionBarMachine`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ActionBarInput {
    /// A reply is running.
    pub running: bool,
    /// Hide the bar while a reply runs.
    pub hide_when_running: bool,
    /// Hover-reveal rule.
    pub autohide: Autohide,
    /// This is the last message in the thread.
    pub is_last: bool,
}

/// The action bar is not rendered at all (while running, if so configured).
#[must_use]
pub fn action_bar_hidden(input: &ActionBarInput) -> bool {
    input.hide_when_running && input.running
}

/// The bar is rendered but only revealed on hover or focus.
#[must_use]
pub fn action_bar_floating(input: &ActionBarInput) -> bool {
    match input.autohide {
        Autohide::Never => false,
        Autohide::Always => true,
        Autohide::NotLast => !input.is_last,
    }
}

/// Copy confirmation, votes and speech for one message.
#[derive(Debug, Clone, Copy)]
pub struct ActionBarMachine;

impl Machine for ActionBarMachine {
    type Input = ActionBarInput;
    type State = ActionBarState;
    type Event = ActionBarEvent;
    type Command = ActionBarCommand;

    fn initial(_: &ActionBarInput) -> ActionBarState {
        ActionBarState::default()
    }

    fn transition(
        state: &ActionBarState,
        event: ActionBarEvent,
        input: &ActionBarInput,
    ) -> Transition<ActionBarState, ActionBarCommand> {
        let mut next = state.clone();
        let mut commands = Vec::new();
        match event {
            ActionBarEvent::Copy => {
                if !input.running {
                    next.copied = true;
                    commands.push(ActionBarCommand::Copy);
                }
            }
            ActionBarEvent::CopiedReset => next.copied = false,
            ActionBarEvent::Vote(v) => {
                next.feedback = if state.feedback == Some(v) { None } else { Some(v) };
                commands.push(ActionBarCommand::Feedback(next.feedback));
            }
            ActionBarEvent::Reload => commands.push(ActionBarCommand::Reload),
            ActionBarEvent::Edit => commands.push(ActionBarCommand::Edit),
            ActionBarEvent::Export => commands.push(ActionBarCommand::Export),
            ActionBarEvent::ToggleSpeak => {
                next.speaking = !state.speaking;
                commands.push(if next.speaking { ActionBarCommand::Speak } else { ActionBarCommand::StopSpeaking });
            }
        }
        Transition::with(next, commands)
    }
}

// ------------------------------------------------------------ selection toolbar

/// Text the reader selected inside a message.
#[derive(Debug, Clone, PartialEq)]
pub struct Selection {
    /// The message it is in.
    pub message_id: String,
    /// The selected text.
    pub text: String,
    /// Viewport position of the selection's top-left, in px.
    pub left: f64,
    /// Viewport position of the selection's top edge, in px.
    pub top: f64,
}

/// What [`SelectionToolbarMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectionToolbarState {
    /// The current selection; the toolbar shows while this is set.
    pub selection: Option<Selection>,
}

/// Inputs to [`SelectionToolbarMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionToolbarEvent {
    /// The page selection changed. Blank text hides the toolbar.
    Selected(Selection),
    /// The selection was cleared or left the messages.
    Cleared,
    /// Quote the selection.
    Quote,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionToolbarCommand {
    /// Quote this text into the composer, then clear the page selection.
    Quote(Selection),
}

/// Shows a Quote button over selected message text.
#[derive(Debug, Clone, Copy)]
pub struct SelectionToolbarMachine;

impl Machine for SelectionToolbarMachine {
    type Input = ();
    type State = SelectionToolbarState;
    type Event = SelectionToolbarEvent;
    type Command = SelectionToolbarCommand;

    fn initial((): &()) -> SelectionToolbarState {
        SelectionToolbarState::default()
    }

    fn transition(
        state: &SelectionToolbarState,
        event: SelectionToolbarEvent,
        (): &(),
    ) -> Transition<SelectionToolbarState, SelectionToolbarCommand> {
        let mut commands = Vec::new();
        let selection = match event {
            SelectionToolbarEvent::Selected(s) if !s.text.trim().is_empty() => Some(s),
            SelectionToolbarEvent::Selected(_) | SelectionToolbarEvent::Cleared => None,
            SelectionToolbarEvent::Quote => {
                if let Some(s) = state.selection.clone() {
                    commands.push(SelectionToolbarCommand::Quote(s));
                }
                None
            }
        };
        Transition::with(SelectionToolbarState { selection }, commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step as advance;

    fn pick(count: usize, running: bool) -> (BranchPickerInput, BranchPickerState) {
        let input = BranchPickerInput { count, running, ..BranchPickerInput::default() };
        (input, BranchPickerMachine::initial(&input))
    }

    #[test]
    fn branches_step_and_stop_at_the_ends() {
        let (input, s) = pick(3, false);
        assert!(!can_previous(&s, &input));
        let t = advance::<BranchPickerMachine>(&s, BranchPickerEvent::Next, &input);
        assert_eq!(t.state.index.value, 1);
        assert_eq!(t.commands, vec![BranchPickerCommand::Switch(1)]);
        let s = advance::<BranchPickerMachine>(&t.state, BranchPickerEvent::Next, &input).state;
        assert!(!can_next(&s, &input));
        assert!(advance::<BranchPickerMachine>(&s, BranchPickerEvent::Next, &input).commands.is_empty());
    }

    #[test]
    fn branches_lock_while_running_unless_allowed() {
        let (mut input, s) = pick(3, true);
        assert!(!can_next(&s, &input));
        input.switch_during_run = true;
        assert!(can_next(&s, &input));
    }

    #[test]
    fn controlled_branch_only_requests() {
        let input = BranchPickerInput { count: 3, index: Some(0), ..BranchPickerInput::default() };
        let s = BranchPickerMachine::initial(&input);
        let t = advance::<BranchPickerMachine>(&s, BranchPickerEvent::Next, &input);
        assert_eq!(t.state.index.value, 0);
        assert_eq!(t.commands, vec![BranchPickerCommand::Switch(1)]);
        let input = BranchPickerInput { index: Some(1), ..input };
        assert_eq!(hydroxy_statechart::sync_or_keep::<BranchPickerMachine>(&t.state, &input).index.value, 1);
    }

    #[test]
    fn branch_index_clamps_when_branches_shrink() {
        let input = BranchPickerInput { count: 2, ..BranchPickerInput::default() };
        let s = BranchPickerState { index: Controllable::new(4, false) };
        assert_eq!(hydroxy_statechart::sync_or_keep::<BranchPickerMachine>(&s, &input).index.value, 1);
    }

    fn bar(
        e: ActionBarEvent,
        s: &ActionBarState,
        input: ActionBarInput,
    ) -> hydroxy_statechart::Step<ActionBarState, ActionBarCommand> {
        advance::<ActionBarMachine>(s, e, &input)
    }

    #[test]
    fn copy_confirms_then_resets() {
        let input = ActionBarInput::default();
        let t = bar(ActionBarEvent::Copy, &ActionBarState::default(), input);
        assert!(t.state.copied);
        assert_eq!(t.commands, vec![ActionBarCommand::Copy]);
        assert!(!bar(ActionBarEvent::CopiedReset, &t.state, input).state.copied);
        let running = ActionBarInput { running: true, ..input };
        assert!(bar(ActionBarEvent::Copy, &ActionBarState::default(), running).commands.is_empty());
    }

    #[test]
    fn voting_twice_takes_the_vote_back() {
        let input = ActionBarInput::default();
        let t = bar(ActionBarEvent::Vote(Feedback::Positive), &ActionBarState::default(), input);
        assert_eq!(t.commands, vec![ActionBarCommand::Feedback(Some(Feedback::Positive))]);
        let t = bar(ActionBarEvent::Vote(Feedback::Negative), &t.state, input);
        assert_eq!(t.state.feedback, Some(Feedback::Negative));
        let t = bar(ActionBarEvent::Vote(Feedback::Negative), &t.state, input);
        assert_eq!(t.commands, vec![ActionBarCommand::Feedback(None)]);
    }

    #[test]
    fn speak_toggles() {
        let input = ActionBarInput::default();
        let t = bar(ActionBarEvent::ToggleSpeak, &ActionBarState::default(), input);
        assert_eq!(t.commands, vec![ActionBarCommand::Speak]);
        let t = bar(ActionBarEvent::ToggleSpeak, &t.state, input);
        assert_eq!(t.commands, vec![ActionBarCommand::StopSpeaking]);
    }

    #[test]
    fn visibility_rules() {
        let mut i = ActionBarInput { running: true, hide_when_running: true, ..ActionBarInput::default() };
        assert!(action_bar_hidden(&i));
        i.running = false;
        assert!(!action_bar_hidden(&i));
        i.autohide = Autohide::NotLast;
        assert!(action_bar_floating(&i));
        i.is_last = true;
        assert!(!action_bar_floating(&i));
        i.autohide = Autohide::Always;
        assert!(action_bar_floating(&i));
    }

    fn sel(text: &str) -> Selection {
        Selection { message_id: "m1".into(), text: text.into(), left: 10.0, top: 20.0 }
    }

    #[test]
    fn toolbar_shows_for_text_and_hides_for_blank() {
        let s = SelectionToolbarMachine::initial(&());
        let s = advance::<SelectionToolbarMachine>(&s, SelectionToolbarEvent::Selected(sel("hello")), &()).state;
        assert!(s.selection.is_some());
        let s = advance::<SelectionToolbarMachine>(&s, SelectionToolbarEvent::Selected(sel("  ")), &()).state;
        assert!(s.selection.is_none());
    }

    #[test]
    fn quote_reports_and_hides() {
        let s = SelectionToolbarState { selection: Some(sel("hello")) };
        let t = advance::<SelectionToolbarMachine>(&s, SelectionToolbarEvent::Quote, &());
        assert_eq!(t.commands, vec![SelectionToolbarCommand::Quote(sel("hello"))]);
        assert!(t.state.selection.is_none());
        assert!(advance::<SelectionToolbarMachine>(&t.state, SelectionToolbarEvent::Quote, &()).commands.is_empty());
    }
}
