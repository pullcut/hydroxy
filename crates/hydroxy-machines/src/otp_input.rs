//! One-time-code input: `length` single-character cells over one string.
//!
//! The value is always a contiguous prefix, so it round-trips through a plain
//! `String` with no holes: typing past the end lands in the first empty cell,
//! and deleting a middle character shifts the rest left, like a text field.
//! Uncontrolled changes commit and report; controlled ones only request, and the
//! owner's value arrives via `sync`. Focus is always local.

use crate::list::Direction;
use hydroxy_statechart::{Machine, Transition};

/// Which characters a cell accepts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum OtpMode {
    /// ASCII digits.
    #[default]
    Numeric,
    /// ASCII letters and digits.
    Alphanumeric,
}

impl OtpMode {
    /// Whether `c` is allowed.
    #[must_use]
    pub fn accepts(self, c: char) -> bool {
        match self {
            OtpMode::Numeric => c.is_ascii_digit(),
            OtpMode::Alphanumeric => c.is_ascii_alphanumeric(),
        }
    }

    /// Keep only allowed characters.
    #[must_use]
    pub fn sanitize(self, text: &str) -> String {
        text.chars().filter(|&c| self.accepts(c)).collect()
    }
}

/// Props for [`OtpInputMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtpInputInput {
    /// Number of cells. At least 1.
    pub length: usize,
    /// Controlled value. `Some` at start fixes controlled mode.
    pub value: Option<String>,
    /// Uncontrolled initial value.
    pub default_value: String,
    /// Allowed characters.
    pub mode: OtpMode,
    /// Ignore every change.
    pub disabled: bool,
}

impl Default for OtpInputInput {
    fn default() -> Self {
        Self { length: 6, value: None, default_value: String::new(), mode: OtpMode::default(), disabled: false }
    }
}

impl OtpInputInput {
    fn len(&self) -> usize {
        self.length.max(1)
    }

    fn clean(&self, text: &str) -> String {
        self.mode.sanitize(text).chars().take(self.len()).collect()
    }
}

/// What [`OtpInputMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OtpInputState {
    /// The code so far. ASCII, at most `length` characters.
    pub value: String,
    /// The cell that has (or should get) focus.
    pub focus: usize,
    /// Fixed at start: the owner holds the value and the machine only requests changes.
    pub controlled: bool,
}

impl OtpInputState {
    /// The character in cell `index`, if filled.
    #[must_use]
    pub fn char_at(&self, index: usize) -> Option<char> {
        self.value.chars().nth(index)
    }
}

/// Inputs to [`OtpInputMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OtpInputEvent {
    /// Cell `index` now holds `text` (its native value after an input event).
    /// One new character fills the cell; several (autofill) fill from there on.
    Input {
        /// The cell.
        index: usize,
        /// The cell's whole new text.
        text: String,
    },
    /// Clipboard text pasted into cell `index`. A full-length code fills from the start.
    Paste {
        /// The cell.
        index: usize,
        /// The pasted text.
        text: String,
    },
    /// Clear cell `index`, or the previous cell when it is empty.
    Backspace(usize),
    /// Clear cell `index` and stay.
    Delete(usize),
    /// A cell got focus. Empty cells past the end redirect to the first empty one.
    Focus(usize),
    /// Move focus by one cell.
    Move {
        /// Right ([`Direction::Next`]) or left.
        direction: Direction,
    },
    /// Focus the first cell.
    First,
    /// Focus the last reachable cell.
    Last,
}

/// Side effects from [`OtpInputMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OtpInputCommand {
    /// Tell the owner the value changed (or should, when controlled).
    ValueChange(String),
    /// Every cell is filled. Runs after `ValueChange`.
    Complete(String),
}

/// Segmented one-time-code entry.
#[derive(Debug, Clone, Copy)]
pub struct OtpInputMachine;

/// The furthest cell focus may reach: the first empty one, or the last cell.
fn reach(value: &str, input: &OtpInputInput) -> usize {
    value.len().min(input.len() - 1)
}

/// `value` without the character at `index`.
fn remove_at(value: &str, index: usize) -> String {
    let mut value = value.to_string();
    value.remove(index);
    value
}

impl OtpInputMachine {
    /// Overwrite cells from `index` with `text` (already sanitized). A full-length
    /// code always starts at the first cell, wherever it was pasted; a longer one
    /// keeps its last `length` characters, where codes sit in "Your code is 123456".
    fn fill(
        state: &OtpInputState,
        index: usize,
        text: &str,
        input: &OtpInputInput,
    ) -> Transition<OtpInputState, OtpInputCommand> {
        let text = &text[text.len().saturating_sub(input.len())..];
        let len = state.value.len();
        let index = if text.len() >= input.len() { 0 } else { index.min(len) };
        let end = index + text.len();
        let value = format!("{}{text}{}", &state.value[..index], state.value.get(end..).unwrap_or(""));
        Self::change(state, &value, end, input)
    }

    fn change(
        state: &OtpInputState,
        value: &str,
        focus: usize,
        input: &OtpInputInput,
    ) -> Transition<OtpInputState, OtpInputCommand> {
        let value: String = value.chars().take(input.len()).collect();
        let focus = focus.min(reach(&value, input));
        if value == state.value {
            return Transition::to(OtpInputState { focus, ..state.clone() });
        }
        let mut commands = vec![OtpInputCommand::ValueChange(value.clone())];
        if value.len() == input.len() {
            commands.push(OtpInputCommand::Complete(value.clone()));
        }
        let value = if state.controlled { state.value.clone() } else { value };
        Transition::with(OtpInputState { value, focus, controlled: state.controlled }, commands)
    }
}

impl Machine for OtpInputMachine {
    type Input = OtpInputInput;
    type State = OtpInputState;
    type Event = OtpInputEvent;
    type Command = OtpInputCommand;

    fn initial(input: &OtpInputInput) -> OtpInputState {
        let value = input.clean(input.value.as_deref().unwrap_or(&input.default_value));
        OtpInputState { focus: reach(&value, input), value, controlled: input.value.is_some() }
    }

    fn transition(
        state: &OtpInputState,
        event: OtpInputEvent,
        input: &OtpInputInput,
    ) -> Transition<OtpInputState, OtpInputCommand> {
        let len = state.value.len();
        let focus = |focus: usize| Transition::to(OtpInputState { focus, ..state.clone() });
        match event {
            OtpInputEvent::Focus(index) => focus(index.min(reach(&state.value, input))),
            OtpInputEvent::Move { direction: Direction::Next } => {
                focus((state.focus + 1).min(reach(&state.value, input)))
            }
            OtpInputEvent::Move { direction: Direction::Prev } => focus(state.focus.saturating_sub(1)),
            OtpInputEvent::First => focus(0),
            OtpInputEvent::Last => focus(reach(&state.value, input)),
            _ if input.disabled => Transition::stay(),
            OtpInputEvent::Input { index, text } => {
                let index = index.min(len);
                // One keystroke beside the cell's old character: drop the old one,
                // wherever the caret put the new one.
                let mut text = text;
                if let Some(old) = state.char_at(index)
                    && text.chars().count() == 2
                    && let Some(at) = text.find(old)
                {
                    text.remove(at);
                }
                let typed = input.mode.sanitize(&text);
                if typed.is_empty() {
                    return focus(index);
                }
                Self::fill(state, index, &typed, input)
            }
            OtpInputEvent::Paste { index, text } => {
                let pasted = input.mode.sanitize(&text);
                if pasted.is_empty() {
                    return Transition::stay();
                }
                Self::fill(state, index, &pasted, input)
            }
            OtpInputEvent::Backspace(index) | OtpInputEvent::Delete(index) if index < len => {
                Self::change(state, &remove_at(&state.value, index), index, input)
            }
            OtpInputEvent::Backspace(_) if len > 0 => {
                Self::change(state, &remove_at(&state.value, len - 1), len - 1, input)
            }
            OtpInputEvent::Backspace(_) | OtpInputEvent::Delete(_) => Transition::stay(),
        }
    }

    fn sync(state: &OtpInputState, input: &OtpInputInput) -> Option<OtpInputState> {
        let value = input.value.as_deref().filter(|_| state.controlled)?;
        let value = input.clean(value);
        if value == state.value {
            return None;
        }
        Some(OtpInputState { focus: state.focus.min(reach(&value, input)), value, controlled: true })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step;

    fn st(value: &str, focus: usize) -> OtpInputState {
        OtpInputState { value: value.into(), focus, controlled: false }
    }

    fn typed(index: usize, text: &str) -> OtpInputEvent {
        OtpInputEvent::Input { index, text: text.into() }
    }

    #[test]
    fn typing_advances_and_lands_in_the_first_empty_cell() {
        let input = OtpInputInput { length: 4, ..OtpInputInput::default() };
        let s = step::<OtpInputMachine>(&st("", 0), typed(0, "1"), &input);
        assert_eq!((s.state.value.as_str(), s.state.focus), ("1", 1));
        assert_eq!(s.commands, vec![OtpInputCommand::ValueChange("1".into())]);
        let s = step::<OtpInputMachine>(&s.state, typed(3, "2"), &input);
        assert_eq!((s.state.value.as_str(), s.state.focus), ("12", 2));
    }

    #[test]
    fn numeric_mode_rejects_letters() {
        let input = OtpInputInput::default();
        let s = step::<OtpInputMachine>(&st("1", 1), typed(1, "a"), &input);
        assert_eq!(s.state, st("1", 1));
        assert!(s.commands.is_empty());
        let alnum = OtpInputInput { mode: OtpMode::Alphanumeric, ..OtpInputInput::default() };
        assert_eq!(step::<OtpInputMachine>(&st("1", 1), typed(1, "a"), &alnum).state.value, "1a");
    }

    #[test]
    fn typing_over_a_filled_cell_replaces_it() {
        let input = OtpInputInput { length: 4, ..OtpInputInput::default() };
        for text in ["25", "52"] {
            let s = step::<OtpInputMachine>(&st("123", 1), typed(1, text), &input);
            assert_eq!((s.state.value.as_str(), s.state.focus), ("153", 2));
        }
    }

    #[test]
    fn completion_fires_once_after_value_change() {
        let input = OtpInputInput { length: 2, ..OtpInputInput::default() };
        let s = step::<OtpInputMachine>(&st("1", 1), typed(1, "2"), &input);
        assert_eq!(s.commands, vec![OtpInputCommand::ValueChange("12".into()), OtpInputCommand::Complete("12".into())]);
        assert_eq!(s.state.focus, 1);
        assert!(step::<OtpInputMachine>(&s.state, typed(1, "2"), &input).commands.is_empty());
    }

    #[test]
    fn paste_and_autofill_fill_the_cells() {
        let input = OtpInputInput::default();
        let s = step::<OtpInputMachine>(
            &st("", 3),
            OtpInputEvent::Paste { index: 3, text: "Code: 123-456".into() },
            &input,
        );
        assert_eq!((s.state.value.as_str(), s.state.focus), ("123456", 5));
        assert_eq!(s.commands.len(), 2);
        let s = step::<OtpInputMachine>(&st("12", 2), OtpInputEvent::Paste { index: 2, text: "34".into() }, &input);
        assert_eq!(s.state.value, "1234");
        // iOS SMS autofill arrives as one input event on the first cell.
        assert_eq!(step::<OtpInputMachine>(&st("", 0), typed(0, "987654"), &input).state.value, "987654");
        // Autofill into a filled cell arrives with the old character in front.
        assert_eq!(step::<OtpInputMachine>(&st("1", 0), typed(0, "1987654"), &input).state.value, "987654");
    }

    #[test]
    fn backspace_clears_then_moves_back() {
        let input = OtpInputInput { length: 4, ..OtpInputInput::default() };
        let s = step::<OtpInputMachine>(&st("123", 3), OtpInputEvent::Backspace(3), &input);
        assert_eq!((s.state.value.as_str(), s.state.focus), ("12", 2));
        let s = step::<OtpInputMachine>(&st("123", 1), OtpInputEvent::Backspace(1), &input);
        assert_eq!((s.state.value.as_str(), s.state.focus), ("13", 1));
        assert!(step::<OtpInputMachine>(&st("", 0), OtpInputEvent::Backspace(0), &input).commands.is_empty());
        let s = step::<OtpInputMachine>(&st("123", 0), OtpInputEvent::Delete(0), &input);
        assert_eq!((s.state.value.as_str(), s.state.focus), ("23", 0));
    }

    #[test]
    fn focus_moves_are_clamped_to_the_filled_prefix() {
        let input = OtpInputInput { length: 4, ..OtpInputInput::default() };
        assert_eq!(step::<OtpInputMachine>(&st("1", 0), OtpInputEvent::Focus(3), &input).state.focus, 1);
        assert_eq!(
            step::<OtpInputMachine>(&st("1", 1), OtpInputEvent::Move { direction: Direction::Next }, &input)
                .state
                .focus,
            1
        );
        assert_eq!(step::<OtpInputMachine>(&st("1234", 0), OtpInputEvent::Last, &input).state.focus, 3);
        assert_eq!(
            step::<OtpInputMachine>(&st("12", 2), OtpInputEvent::Move { direction: Direction::Prev }, &input)
                .state
                .focus,
            1
        );
    }

    #[test]
    fn controlled_requests_then_syncs() {
        let mut input = OtpInputInput { length: 4, value: Some(String::new()), ..OtpInputInput::default() };
        let s = OtpInputMachine::initial(&input);
        assert!(s.controlled);
        let step = hydroxy_statechart::step::<OtpInputMachine>(&s, typed(0, "7"), &input);
        assert_eq!((step.state.value.as_str(), step.state.focus), ("", 1));
        assert_eq!(step.commands, vec![OtpInputCommand::ValueChange("7".into())]);
        input.value = Some("7".into());
        assert_eq!(hydroxy_statechart::sync_or_keep::<OtpInputMachine>(&step.state, &input).value, "7");
    }

    #[test]
    fn disabled_blocks_edits_and_initial_is_sanitized() {
        let input = OtpInputInput { length: 3, default_value: "1a2345".into(), ..OtpInputInput::default() };
        assert_eq!(OtpInputMachine::initial(&input), st("123", 2));
        let off = OtpInputInput { disabled: true, ..input };
        assert!(step::<OtpInputMachine>(&st("", 0), typed(0, "1"), &off).commands.is_empty());
    }
}
