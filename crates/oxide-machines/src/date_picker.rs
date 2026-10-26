//! Date picker: a text field that accepts a typed `YYYY-MM-DD` date, plus a
//! trigger that opens a [`calendar`](crate::calendar) in a popover. Owns the
//! value, the field's text and the open flag.

use oxide_statechart::{Machine, Transition};

use crate::calendar::CalendarInput;
use crate::date::Date;

/// Props for [`DatePickerMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DatePickerInput {
    /// Controlled value. `Some` at start fixes controlled mode.
    pub value: Option<Date>,
    /// Uncontrolled initial value.
    pub default_value: Option<Date>,
    /// Availability rules shared with the calendar (`min`, `max`,
    /// `unavailable`, `disabled`). Its `value`/`default_value` are ignored.
    pub rules: CalendarInput,
}

/// What [`DatePickerMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DatePickerState {
    /// The chosen date.
    pub value: Option<Date>,
    /// The field's text, which may be a half-typed date.
    pub text: String,
    /// Whether the calendar popover is showing.
    pub open: bool,
    /// Fixed at start: the owner holds the value and the machine only requests changes.
    pub controlled: bool,
}

impl DatePickerState {
    /// The field holds text that isn't the current value (a typo, a blocked date).
    #[must_use]
    pub fn invalid(&self) -> bool {
        !self.text.is_empty() && self.text.parse::<Date>().map_or(true, |d| Some(d) != self.value)
    }
}

/// Inputs to [`DatePickerMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatePickerEvent {
    /// The field's text changed.
    Input(String),
    /// The field lost focus or Enter was pressed: tidy the text.
    Commit,
    /// The trigger was activated.
    Toggle,
    /// Close the popover; `restore_focus` sends focus back to the trigger.
    Close {
        /// Whether to focus the trigger afterwards.
        restore_focus: bool,
    },
    /// A day was chosen in the calendar.
    Select(Date),
}

/// Side effects from [`DatePickerMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatePickerCommand {
    /// Tell the owner the value changed (or should, when controlled). `None` when cleared.
    ValueChange(Option<Date>),
    /// Move DOM focus back to the trigger.
    FocusTrigger,
}

/// Value, text and popover state for one date picker.
#[derive(Debug, Clone, Copy)]
pub struct DatePickerMachine;

type Step = Transition<DatePickerState, DatePickerCommand>;

fn text_of(value: Option<Date>) -> String {
    value.map(|d| d.to_string()).unwrap_or_default()
}

/// Commit `value` when uncontrolled and request the change.
fn change(next: &mut DatePickerState, commands: &mut Vec<DatePickerCommand>, value: Option<Date>) {
    if !next.controlled {
        next.value = value;
    }
    commands.push(DatePickerCommand::ValueChange(value));
}

impl Machine for DatePickerMachine {
    type Input = DatePickerInput;
    type State = DatePickerState;
    type Event = DatePickerEvent;
    type Command = DatePickerCommand;

    fn initial(input: &DatePickerInput) -> DatePickerState {
        let value = input.value.or(input.default_value);
        DatePickerState { value, text: text_of(value), open: false, controlled: input.value.is_some() }
    }

    fn transition(state: &DatePickerState, event: DatePickerEvent, input: &DatePickerInput) -> Step {
        let rules = &input.rules;
        if rules.disabled {
            return Transition::stay(state.clone());
        }
        let mut next = state.clone();
        let mut commands = Vec::new();
        match event {
            DatePickerEvent::Input(text) => {
                let parsed = text.parse::<Date>().ok().filter(|d| rules.is_selectable(*d));
                if text.is_empty() && state.value.is_some() {
                    change(&mut next, &mut commands, None);
                } else if let Some(date) = parsed.filter(|d| state.value != Some(*d)) {
                    change(&mut next, &mut commands, Some(date));
                }
                next.text = text;
            }
            DatePickerEvent::Commit => next.text = text_of(next.value),
            DatePickerEvent::Toggle => next.open = !state.open,
            DatePickerEvent::Close { .. } if !state.open => {}
            DatePickerEvent::Close { restore_focus } => {
                next.open = false;
                if restore_focus {
                    commands.push(DatePickerCommand::FocusTrigger);
                }
            }
            DatePickerEvent::Select(date) if !rules.is_selectable(date) => {}
            DatePickerEvent::Select(date) => {
                if state.value != Some(date) {
                    change(&mut next, &mut commands, Some(date));
                }
                next.text = date.to_string();
                next.open = false;
                commands.push(DatePickerCommand::FocusTrigger);
            }
        }
        Transition { state: next, commands }
    }

    fn sync(state: &DatePickerState, input: &DatePickerInput) -> DatePickerState {
        let mut next = state.clone();
        if state.controlled && input.value.is_some() && input.value != state.value {
            next.value = input.value;
            next.text = text_of(input.value);
        }
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Date {
        s.parse().unwrap()
    }

    fn run(state: &DatePickerState, event: DatePickerEvent, input: &DatePickerInput) -> Step {
        DatePickerMachine::transition(state, event, input)
    }

    #[test]
    fn typing_a_full_date_commits_it() {
        let i = DatePickerInput::default();
        let s = DatePickerMachine::initial(&i);
        let half = run(&s, DatePickerEvent::Input("2026-10".into()), &i);
        assert!(half.commands.is_empty());
        assert!(half.state.invalid());
        let full = run(&half.state, DatePickerEvent::Input("2026-10-30".into()), &i);
        assert_eq!(full.state.value, Some(d("2026-10-30")));
        assert_eq!(full.commands, vec![DatePickerCommand::ValueChange(Some(d("2026-10-30")))]);
        assert!(!full.state.invalid());
    }

    #[test]
    fn commit_reverts_bad_text_and_clearing_clears() {
        let i = DatePickerInput { default_value: Some(d("2026-10-01")), ..Default::default() };
        let s = DatePickerMachine::initial(&i);
        let typo = run(&s, DatePickerEvent::Input("2026-13-01".into()), &i);
        assert!(typo.commands.is_empty());
        assert_eq!(run(&typo.state, DatePickerEvent::Commit, &i).state.text, "2026-10-01");
        let cleared = run(&s, DatePickerEvent::Input(String::new()), &i);
        assert_eq!(cleared.state.value, None);
        assert_eq!(cleared.commands, vec![DatePickerCommand::ValueChange(None)]);
    }

    #[test]
    fn typed_dates_respect_the_rules() {
        let rules = CalendarInput { max: Some(d("2026-10-31")), unavailable: vec![d("2026-10-10")], ..Default::default() };
        let i = DatePickerInput { rules, ..Default::default() };
        let s = DatePickerMachine::initial(&i);
        assert!(run(&s, DatePickerEvent::Input("2026-11-01".into()), &i).commands.is_empty());
        assert!(run(&s, DatePickerEvent::Input("2026-10-10".into()), &i).state.invalid());
    }

    #[test]
    fn selecting_closes_and_returns_focus() {
        let i = DatePickerInput::default();
        let open = run(&DatePickerMachine::initial(&i), DatePickerEvent::Toggle, &i).state;
        assert!(open.open);
        let step = run(&open, DatePickerEvent::Select(d("2026-10-30")), &i);
        assert!(!step.state.open);
        assert_eq!(step.state.text, "2026-10-30");
        assert_eq!(
            step.commands,
            vec![DatePickerCommand::ValueChange(Some(d("2026-10-30"))), DatePickerCommand::FocusTrigger]
        );
    }

    #[test]
    fn escape_restores_focus_but_outside_click_does_not() {
        let i = DatePickerInput::default();
        let open = DatePickerState { open: true, ..Default::default() };
        assert_eq!(run(&open, DatePickerEvent::Close { restore_focus: true }, &i).commands, vec![DatePickerCommand::FocusTrigger]);
        assert!(run(&open, DatePickerEvent::Close { restore_focus: false }, &i).commands.is_empty());
        assert!(run(&DatePickerState::default(), DatePickerEvent::Close { restore_focus: true }, &i).commands.is_empty());
    }

    #[test]
    fn controlled_requests_and_follows_sync() {
        let mut i = DatePickerInput { value: Some(d("2026-10-01")), ..Default::default() };
        let step = run(&DatePickerMachine::initial(&i), DatePickerEvent::Select(d("2026-10-02")), &i);
        assert_eq!(step.state.value, Some(d("2026-10-01")));
        i.value = Some(d("2026-10-02"));
        let synced = DatePickerMachine::sync(&step.state, &i);
        assert_eq!((synced.value, synced.text.as_str()), (Some(d("2026-10-02")), "2026-10-02"));
    }

    #[test]
    fn disabled_ignores_everything() {
        let i = DatePickerInput { rules: CalendarInput { disabled: true, ..Default::default() }, ..Default::default() };
        let s = DatePickerMachine::initial(&i);
        assert_eq!(run(&s, DatePickerEvent::Toggle, &i).state, s);
    }
}
