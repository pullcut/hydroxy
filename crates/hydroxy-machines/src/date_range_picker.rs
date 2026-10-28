//! Date range picker: a button that opens a two-month calendar in a popover.
//! The first click sets the anchor and the pointer or arrow keys preview the
//! range from it; the second click completes the range and closes. Availability
//! rules and the roving focus move are shared with the [`calendar`](crate::calendar).

use hydroxy_statechart::{Machine, Transition};

use crate::calendar::{CalendarInput, CalendarMove};
use crate::date::Date;

/// An inclusive span of days, `start <= end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateRange {
    /// First day.
    pub start: Date,
    /// Last day.
    pub end: Date,
}

impl DateRange {
    /// The range between two dates in either order.
    #[must_use]
    pub fn new(a: Date, b: Date) -> Self {
        if a <= b { Self { start: a, end: b } } else { Self { start: b, end: a } }
    }

    /// Whether `date` is one of the days.
    #[must_use]
    pub fn contains(self, date: Date) -> bool {
        self.start <= date && date <= self.end
    }
}

/// Props for [`DateRangePickerMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DateRangePickerInput {
    /// Controlled value. `Some` at start fixes controlled mode.
    pub value: Option<DateRange>,
    /// Uncontrolled initial value.
    pub default_value: Option<DateRange>,
    /// Availability rules shared with the calendar (`today`, `week_start`, `min`,
    /// `max`, `unavailable`, `disabled`). Its `value` fields are ignored.
    pub rules: CalendarInput,
}

/// What [`DateRangePickerMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DateRangePickerState {
    /// The chosen range.
    pub value: Option<DateRange>,
    /// The first click of a range in progress.
    pub anchor: Option<Date>,
    /// The day the range in progress would end on.
    pub hover: Option<Date>,
    /// The day that holds the tab stop.
    pub focused: Date,
    /// First day of the left-hand month. The right-hand one follows it.
    pub month: Date,
    /// Whether the popover is showing.
    pub open: bool,
    /// Fixed at start: the owner holds the value.
    pub controlled: bool,
}

impl DateRangePickerState {
    /// The range to highlight: the one in progress, else the chosen one.
    #[must_use]
    pub fn preview(&self) -> Option<DateRange> {
        match self.anchor {
            Some(anchor) => Some(DateRange::new(anchor, self.hover.unwrap_or(anchor))),
            None => self.value,
        }
    }

    /// Keep the focused day inside the two months on show.
    fn reveal_focused(&mut self) {
        if self.focused < self.month {
            self.month = self.focused.first_of_month();
        } else if self.focused > self.month.add_months(1).last_of_month() {
            self.month = self.focused.first_of_month().add_months(-1);
        }
    }
}

/// Inputs to [`DateRangePickerMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateRangePickerEvent {
    /// The trigger was activated.
    Toggle,
    /// Close the popover; `restore_focus` sends focus back to the trigger.
    Close {
        /// Whether to focus the trigger afterwards.
        restore_focus: bool,
    },
    /// A day was clicked (or Enter/Space on it).
    Select(Date),
    /// The pointer is over a day.
    Hover(Date),
    /// A day received DOM focus.
    Focus(Date),
    /// A navigation key on the focused day.
    Move(CalendarMove),
    /// The previous (-1) or next (+1) month button.
    ShiftMonth(i32),
}

/// Side effects from [`DateRangePickerMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateRangePickerCommand {
    /// The range changed (or should, when controlled).
    ValueChange(DateRange),
    /// Move DOM focus to this day, once it is rendered.
    FocusDay(Date),
    /// Move DOM focus back to the trigger.
    FocusTrigger,
}

/// Range, anchor and popover state.
#[derive(Debug, Clone, Copy)]
pub struct DateRangePickerMachine;

type Step = Transition<DateRangePickerState, DateRangePickerCommand>;

fn moved(date: Date, by: CalendarMove, week_start: u8) -> Date {
    match by {
        CalendarMove::Days(n) => date.add_days(n),
        CalendarMove::Months(n) => date.add_months(n),
        CalendarMove::Years(n) => date.add_years(n),
        CalendarMove::WeekStart => date.add_days(-date.days_into_week(week_start)),
        CalendarMove::WeekEnd => date.add_days(6 - date.days_into_week(week_start)),
    }
}

impl Machine for DateRangePickerMachine {
    type Input = DateRangePickerInput;
    type State = DateRangePickerState;
    type Event = DateRangePickerEvent;
    type Command = DateRangePickerCommand;

    fn initial(input: &DateRangePickerInput) -> DateRangePickerState {
        let value = input.value.or(input.default_value);
        let focused = input.rules.clamp(value.map_or(input.rules.today, |r| r.start));
        DateRangePickerState {
            value,
            focused,
            month: focused.first_of_month(),
            controlled: input.value.is_some(),
            ..DateRangePickerState::default()
        }
    }

    fn transition(state: &DateRangePickerState, event: DateRangePickerEvent, input: &DateRangePickerInput) -> Step {
        let rules = &input.rules;
        if rules.disabled {
            return Transition::stay();
        }
        let mut next = state.clone();
        let mut commands = Vec::new();
        match event {
            DateRangePickerEvent::Toggle if state.open => {
                return Self::transition(state, DateRangePickerEvent::Close { restore_focus: false }, input);
            }
            DateRangePickerEvent::Toggle => {
                next.open = true;
                next.anchor = None;
                next.hover = None;
                next.focused = rules.clamp(state.value.map_or(rules.today, |r| r.start));
                next.month = next.focused.first_of_month();
            }
            DateRangePickerEvent::Close { restore_focus } => {
                next.open = false;
                next.anchor = None;
                next.hover = None;
                if restore_focus && state.open {
                    commands.push(DateRangePickerCommand::FocusTrigger);
                }
            }
            DateRangePickerEvent::Select(date) if !rules.is_selectable(date) => {}
            DateRangePickerEvent::Select(date) => match state.anchor {
                None => {
                    next.anchor = Some(date);
                    next.hover = Some(date);
                    next.focused = date;
                }
                Some(anchor) => {
                    let range = DateRange::new(anchor, date);
                    if !state.controlled {
                        next.value = Some(range);
                    }
                    next.open = false;
                    next.anchor = None;
                    next.hover = None;
                    next.focused = date;
                    commands.push(DateRangePickerCommand::ValueChange(range));
                    commands.push(DateRangePickerCommand::FocusTrigger);
                }
            },
            DateRangePickerEvent::Hover(date) => {
                if state.anchor.is_some() {
                    next.hover = Some(date);
                }
            }
            DateRangePickerEvent::Focus(date) => {
                next.focused = rules.clamp(date);
                if state.anchor.is_some() {
                    next.hover = Some(next.focused);
                }
                next.reveal_focused();
            }
            DateRangePickerEvent::Move(by) => {
                let date = rules.clamp(moved(state.focused, by, rules.week_start));
                if date != state.focused {
                    next.focused = date;
                    if state.anchor.is_some() {
                        next.hover = Some(date);
                    }
                    next.reveal_focused();
                    commands.push(DateRangePickerCommand::FocusDay(date));
                }
            }
            DateRangePickerEvent::ShiftMonth(n) => {
                next.month = state.month.add_months(n);
                next.focused = rules.clamp(state.focused.add_months(n));
                next.reveal_focused();
            }
        }
        Transition::with(next, commands)
    }

    fn sync(state: &DateRangePickerState, input: &DateRangePickerInput) -> Option<DateRangePickerState> {
        let mut next = state.clone();
        if state.controlled && input.value != state.value {
            next.value = input.value;
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Date {
        s.parse().unwrap()
    }

    fn input() -> DateRangePickerInput {
        DateRangePickerInput {
            rules: CalendarInput { today: d("2026-10-26"), week_start: 1, ..Default::default() },
            ..Default::default()
        }
    }

    fn send(
        state: &DateRangePickerState,
        event: DateRangePickerEvent,
        input: &DateRangePickerInput,
    ) -> hydroxy_statechart::Step<DateRangePickerState, DateRangePickerCommand> {
        DateRangePickerMachine::transition(state, event, input).resolve(state)
    }

    #[test]
    fn two_clicks_make_a_range_in_either_order() {
        let input = input();
        let s = DateRangePickerMachine::initial(&input);
        let s = send(&s, DateRangePickerEvent::Toggle, &input).state;
        assert!(s.open && s.month == d("2026-10-01"));
        let t = send(&s, DateRangePickerEvent::Select(d("2026-11-05")), &input);
        assert_eq!(t.state.anchor, Some(d("2026-11-05")));
        assert!(t.commands.is_empty() && t.state.open);
        let t = send(&t.state, DateRangePickerEvent::Select(d("2026-10-30")), &input);
        let range = DateRange::new(d("2026-10-30"), d("2026-11-05"));
        assert_eq!(t.state.value, Some(range));
        assert!(!t.state.open && t.state.anchor.is_none());
        assert_eq!(t.commands, [DateRangePickerCommand::ValueChange(range), DateRangePickerCommand::FocusTrigger]);
    }

    #[test]
    fn hover_previews_only_while_anchored() {
        let input = input();
        let s = send(&DateRangePickerMachine::initial(&input), DateRangePickerEvent::Toggle, &input).state;
        assert_eq!(send(&s, DateRangePickerEvent::Hover(d("2026-10-30")), &input).state.hover, None);
        let s = send(&s, DateRangePickerEvent::Select(d("2026-10-10")), &input).state;
        let s = send(&s, DateRangePickerEvent::Hover(d("2026-10-05")), &input).state;
        assert_eq!(s.preview(), Some(DateRange::new(d("2026-10-05"), d("2026-10-10"))));
    }

    #[test]
    fn keyboard_extends_the_preview_and_scrolls_months() {
        let input = input();
        let s = send(&DateRangePickerMachine::initial(&input), DateRangePickerEvent::Toggle, &input).state;
        let s = send(&s, DateRangePickerEvent::Select(d("2026-10-26")), &input).state;
        // Two months on show: October and November. Moving into December scrolls by one.
        let t = send(&s, DateRangePickerEvent::Move(CalendarMove::Months(2)), &input);
        assert_eq!(t.state.focused, d("2026-12-26"));
        assert_eq!(t.state.month, d("2026-11-01"));
        assert_eq!(t.state.hover, Some(d("2026-12-26")));
        assert_eq!(t.commands, [DateRangePickerCommand::FocusDay(d("2026-12-26"))]);
        let t = send(&t.state, DateRangePickerEvent::ShiftMonth(-1), &input);
        assert_eq!(t.state.month, d("2026-10-01"));
    }

    #[test]
    fn unavailable_days_cannot_be_picked_and_close_discards_the_anchor() {
        let mut input = input();
        input.rules.unavailable = vec![d("2026-10-27")];
        let s = send(&DateRangePickerMachine::initial(&input), DateRangePickerEvent::Toggle, &input).state;
        assert_eq!(send(&s, DateRangePickerEvent::Select(d("2026-10-27")), &input).state.anchor, None);
        let s = send(&s, DateRangePickerEvent::Select(d("2026-10-26")), &input).state;
        let t = send(&s, DateRangePickerEvent::Close { restore_focus: true }, &input);
        assert!(!t.state.open && t.state.anchor.is_none() && t.state.value.is_none());
        assert_eq!(t.commands, [DateRangePickerCommand::FocusTrigger]);
    }

    #[test]
    fn controlled_only_requests() {
        let range = DateRange::new(d("2026-10-01"), d("2026-10-05"));
        let input = DateRangePickerInput { value: Some(range), ..input() };
        let s = DateRangePickerMachine::initial(&input);
        assert!(s.controlled);
        let s = send(&s, DateRangePickerEvent::Toggle, &input).state;
        let s = send(&s, DateRangePickerEvent::Select(d("2026-10-10")), &input).state;
        let t = send(&s, DateRangePickerEvent::Select(d("2026-10-12")), &input);
        assert_eq!(t.state.value, Some(range));
        let moved = DateRangePickerInput { value: Some(DateRange::new(d("2026-10-10"), d("2026-10-12"))), ..input };
        assert_eq!(hydroxy_statechart::sync_or_keep::<DateRangePickerMachine>(&t.state, &moved).value, moved.value);
    }

    #[test]
    fn disabled_ignores_events() {
        let mut input = input();
        input.rules.disabled = true;
        let s = DateRangePickerMachine::initial(&input);
        assert!(!send(&s, DateRangePickerEvent::Toggle, &input).state.open);
    }
}
