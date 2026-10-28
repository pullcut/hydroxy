//! The phone field state machine.

use hydroxy_statechart::{Machine, Transition};

use super::countries::{Country, country, country_or_first, parse_e164, split_international, visible_isos};
use crate::as_you_type as ayt;
use crate::listbox::{self, Direction};

/// Props for [`PhoneFieldMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhoneFieldInput {
    /// Controlled E.164 value. `Some` at start fixes controlled mode.
    pub value: Option<String>,
    /// Uncontrolled initial E.164 value.
    pub default_value: Option<String>,
    /// ISO code of the country to start in when there is no value.
    pub default_country: String,
    /// Ignore every change.
    pub disabled: bool,
    /// Focusable and selectable, but unchangeable.
    pub read_only: bool,
}

impl Default for PhoneFieldInput {
    fn default() -> Self {
        Self { value: None, default_value: None, default_country: "US".into(), disabled: false, read_only: false }
    }
}

/// What [`PhoneFieldMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhoneFieldState {
    /// ISO code of the chosen country.
    pub country: String,
    /// Digits typed for the national number, a trunk `0` included.
    pub national: String,
    /// The input's text.
    pub text: String,
    /// Where the caret belongs after the last edit, as a char index.
    pub caret: usize,
    /// The field has been left once, so validity may show.
    pub touched: bool,
    /// The country list is open.
    pub picker_open: bool,
    /// The picker's search text.
    pub query: String,
    /// ISO code of the highlighted country in the picker.
    pub highlighted: Option<String>,
    /// Fixed at start: the owner holds the value.
    pub controlled: bool,
}

impl PhoneFieldState {
    /// The chosen country.
    #[must_use]
    pub fn country(&self) -> &'static Country {
        country_or_first(&self.country)
    }

    /// The number in E.164, or `None` while empty.
    #[must_use]
    pub fn e164(&self) -> Option<String> {
        self.country().e164(&self.national)
    }

    /// Whether the number has a plausible length.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.country().is_valid(&self.national)
    }

    /// Show the field as invalid: left once, not empty, and not plausible.
    #[must_use]
    pub fn invalid(&self) -> bool {
        self.touched && !self.national.is_empty() && !self.is_valid()
    }

    fn load(&mut self, value: Option<&str>, fallback: &str) {
        if let Some((c, national)) = value.and_then(|v| parse_e164(v, fallback)) {
            self.country = c.iso.to_string();
            self.national = national.chars().take(c.max_typed()).collect();
        } else {
            self.country = country_or_first(fallback).iso.to_string();
            self.national.clear();
        }
        self.text = self.country().display(&self.national);
        self.caret = self.text.chars().count();
    }
}

/// Inputs to [`PhoneFieldMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhoneFieldEvent {
    /// The person typed or pasted in the number. `caret` is a char index.
    SetText {
        /// The input's new text.
        text: String,
        /// Caret after the edit.
        caret: usize,
    },
    /// Focus left the number.
    Blur,
    /// Open the country list on the current country.
    OpenPicker,
    /// Close the list without choosing.
    ClosePicker {
        /// Put focus back on the country button (Escape), not when focus moved on its own.
        restore_focus: bool,
    },
    /// Toggle the list (the country button).
    TogglePicker,
    /// The picker search changed.
    Query(String),
    /// Arrow key in the picker.
    Move(Direction),
    /// Pointer over a country.
    Highlight(String),
    /// Choose a country by ISO code.
    Select(String),
    /// Choose the highlighted country.
    Enter,
}

/// Side effects from [`PhoneFieldMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhoneFieldCommand {
    /// The number changed: E.164 (`None` when empty) and whether it looks valid.
    ValueChange {
        /// `+` dial code and significant number.
        e164: Option<String>,
        /// Plausible length for the country.
        valid: bool,
    },
    /// The country changed, by ISO code.
    CountryChange(String),
    /// Focus the number input (after picking a country).
    FocusNumber,
    /// Focus the country button (after Escape in the picker).
    FocusTrigger,
}

/// A phone number with a country.
#[derive(Debug, Clone, Copy)]
pub struct PhoneFieldMachine;

type Step = Transition<PhoneFieldState, PhoneFieldCommand>;

fn is_sig(c: char) -> bool {
    c.is_ascii_digit() || c == '+'
}

/// Report what changed between `before` and `next`.
fn diff(before: &PhoneFieldState, next: PhoneFieldState, mut commands: Vec<PhoneFieldCommand>) -> Step {
    if next.country != before.country {
        commands.insert(0, PhoneFieldCommand::CountryChange(next.country.clone()));
    }
    let (e164, valid) = (next.e164(), next.is_valid());
    if e164 != before.e164() || valid != before.is_valid() {
        commands.push(PhoneFieldCommand::ValueChange { e164, valid });
    }
    // Focus commands go last so they run after the value is reported.
    commands.sort_by_key(|c| matches!(c, PhoneFieldCommand::FocusNumber | PhoneFieldCommand::FocusTrigger));
    Transition::with(next, commands)
}

fn choose(state: &PhoneFieldState, iso: &str) -> Step {
    let Some(c) = country(iso) else { return Transition::stay() };
    let mut next = state.clone();
    next.country = c.iso.to_string();
    next.national = state.national.chars().take(c.max_typed()).collect();
    next.text = c.display(&next.national);
    next.caret = next.text.chars().count();
    next.picker_open = false;
    diff(state, next, vec![PhoneFieldCommand::FocusNumber])
}

impl Machine for PhoneFieldMachine {
    type Input = PhoneFieldInput;
    type State = PhoneFieldState;
    type Event = PhoneFieldEvent;
    type Command = PhoneFieldCommand;

    fn initial(input: &PhoneFieldInput) -> PhoneFieldState {
        let mut state = PhoneFieldState { controlled: input.value.is_some(), ..PhoneFieldState::default() };
        state.load(input.value.as_deref().or(input.default_value.as_deref()), &input.default_country);
        state
    }

    fn transition(state: &PhoneFieldState, event: PhoneFieldEvent, input: &PhoneFieldInput) -> Step {
        use PhoneFieldEvent as E;
        if input.disabled || input.read_only {
            return Transition::stay();
        }
        let mut next = state.clone();
        match event {
            E::SetText { mut text, mut caret } => {
                if ayt::deleted_literal(&state.text, &text, is_sig) {
                    (text, caret) = ayt::drop_sig_before(&text, caret, is_sig);
                }
                let digits: String = text.chars().filter(char::is_ascii_digit).collect();
                if text.trim_start().starts_with('+') {
                    // International entry: find the country from the dial code.
                    if let Some((c, rest)) = split_international(&digits, &state.country) {
                        next.country = c.iso.to_string();
                        next.national = rest.chars().take(c.max_typed()).collect();
                        next.text = c.display(&next.national);
                    } else {
                        next.national.clear();
                        next.text = format!("+{digits}");
                    }
                    next.caret = next.text.chars().count();
                } else {
                    let c = state.country();
                    let at_end = caret >= text.chars().count();
                    next.national = digits.chars().take(c.max_typed()).collect();
                    next.text = c.display(&next.national);
                    next.caret = if at_end {
                        next.text.chars().count()
                    } else {
                        ayt::caret_after(&next.text, ayt::sig_before(&text, caret, is_sig), is_sig)
                    };
                }
                diff(state, next, Vec::new())
            }
            E::Blur => {
                next.touched = true;
                Transition::to(next)
            }
            E::OpenPicker => {
                next.picker_open = true;
                next.query.clear();
                next.highlighted = Some(state.country.clone());
                Transition::to(next)
            }
            E::TogglePicker if state.picker_open => {
                next.picker_open = false;
                Transition::to(next)
            }
            E::TogglePicker => Self::transition(state, E::OpenPicker, input),
            E::ClosePicker { restore_focus } => {
                if !state.picker_open {
                    return Transition::to(next);
                }
                next.picker_open = false;
                let commands = if restore_focus { vec![PhoneFieldCommand::FocusTrigger] } else { Vec::new() };
                Transition::with(next, commands)
            }
            E::Query(query) => {
                next.highlighted = listbox::keep(&visible_isos(&query), None);
                next.query = query;
                Transition::to(next)
            }
            E::Move(direction) => {
                next.highlighted = listbox::step(&visible_isos(&state.query), state.highlighted.as_deref(), direction);
                Transition::to(next)
            }
            E::Highlight(iso) => {
                if visible_isos(&state.query).contains(&iso.as_str()) {
                    next.highlighted = Some(iso);
                }
                Transition::to(next)
            }
            E::Select(iso) => choose(state, &iso),
            E::Enter => match state.highlighted.as_deref().filter(|_| state.picker_open) {
                Some(iso) if visible_isos(&state.query).contains(&iso) => choose(state, iso),
                _ => Transition::to(next),
            },
        }
    }

    fn sync(state: &PhoneFieldState, input: &PhoneFieldInput) -> Option<PhoneFieldState> {
        if !state.controlled || input.value == state.e164() {
            return None;
        }
        let mut next = state.clone();
        next.load(input.value.as_deref(), &state.country);
        hydroxy_statechart::changed(state, next)
    }
}
