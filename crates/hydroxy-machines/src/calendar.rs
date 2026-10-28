//! Calendar: an inline month grid. Owns the selected date and the focused date
//! (the grid's single tab stop); the visible month is the focused date's month.
//! "Today" comes in as input, so the machine never reads a clock.

use hydroxy_statechart::{Machine, Transition};

use crate::date::Date;

/// Props for [`CalendarMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CalendarInput {
    /// Controlled value. `Some` at start fixes controlled mode.
    pub value: Option<Date>,
    /// Uncontrolled initial value.
    pub default_value: Option<Date>,
    /// Today's date: marked in the grid and focused when nothing is selected.
    pub today: Date,
    /// First day of the week, 0 = Sunday … 6 = Saturday.
    pub week_start: u8,
    /// Earliest selectable date. Earlier days can't be focused.
    pub min: Option<Date>,
    /// Latest selectable date. Later days can't be focused.
    pub max: Option<Date>,
    /// Days inside the range that can be focused but not selected.
    pub unavailable: Vec<Date>,
    /// Block every interaction.
    pub disabled: bool,
}

impl CalendarInput {
    /// Whether `date` is within `min`/`max`.
    #[must_use]
    pub fn in_range(&self, date: Date) -> bool {
        self.min.is_none_or(|m| date >= m) && self.max.is_none_or(|m| date <= m)
    }

    /// Whether `date` can be selected.
    #[must_use]
    pub fn is_selectable(&self, date: Date) -> bool {
        !self.disabled && self.in_range(date) && !self.unavailable.contains(&date)
    }

    /// `date` pulled inside `min`/`max`.
    #[must_use]
    pub fn clamp(&self, date: Date) -> Date {
        let date = self.min.map_or(date, |m| date.max(m));
        self.max.map_or(date, |m| date.min(m))
    }

    /// Whether the month `months` away from `focused` has any day in range.
    /// Drives the previous/next month buttons.
    #[must_use]
    pub fn can_shift(&self, focused: Date, months: i32) -> bool {
        let target = focused.add_months(months);
        self.min.is_none_or(|m| target.last_of_month() >= m) && self.max.is_none_or(|m| target.first_of_month() <= m)
    }
}

/// What [`CalendarMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CalendarState {
    /// The selected date.
    pub value: Option<Date>,
    /// The day that holds the tab stop. Its month is the one on show.
    pub focused: Date,
    /// Fixed at start: the owner holds the value and the machine only requests changes.
    pub controlled: bool,
}

/// A keyboard move of the focused day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarMove {
    /// This many days (arrows: ±1, ±7).
    Days(i64),
    /// This many months (Page Up / Page Down).
    Months(i32),
    /// This many years (Shift + Page Up / Page Down).
    Years(i32),
    /// First day of the week (Home).
    WeekStart,
    /// Last day of the week (End).
    WeekEnd,
}

/// Inputs to [`CalendarMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarEvent {
    /// A day was clicked (or Enter/Space on it).
    Select(Date),
    /// A day received DOM focus.
    Focus(Date),
    /// A navigation key on the focused day.
    Move(CalendarMove),
    /// The previous (-1) or next (+1) month button.
    ShiftMonth(i32),
}

/// Side effects from [`CalendarMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarCommand {
    /// Tell the owner the value changed (or should, when controlled).
    ValueChange(Date),
    /// Move DOM focus to this day, once it's rendered.
    FocusDay(Date),
}

/// Selection and roving focus for one month grid.
#[derive(Debug, Clone, Copy)]
pub struct CalendarMachine;

type Step = Transition<CalendarState, CalendarCommand>;

fn moved(date: Date, by: CalendarMove, week_start: u8) -> Date {
    match by {
        CalendarMove::Days(n) => date.add_days(n),
        CalendarMove::Months(n) => date.add_months(n),
        CalendarMove::Years(n) => date.add_years(n),
        CalendarMove::WeekStart => date.add_days(-date.days_into_week(week_start)),
        CalendarMove::WeekEnd => date.add_days(6 - date.days_into_week(week_start)),
    }
}

fn with_focus(state: &CalendarState, focused: Date) -> CalendarState {
    CalendarState { focused, ..state.clone() }
}

impl Machine for CalendarMachine {
    type Input = CalendarInput;
    type State = CalendarState;
    type Event = CalendarEvent;
    type Command = CalendarCommand;

    fn initial(input: &CalendarInput) -> CalendarState {
        let value = input.value.or(input.default_value);
        CalendarState { value, focused: input.clamp(value.unwrap_or(input.today)), controlled: input.value.is_some() }
    }

    fn transition(state: &CalendarState, event: CalendarEvent, input: &CalendarInput) -> Step {
        if input.disabled {
            return Transition::stay();
        }
        match event {
            CalendarEvent::Select(date) if !input.is_selectable(date) => Transition::stay(),
            CalendarEvent::Select(date) if state.value == Some(date) => Transition::to(with_focus(state, date)),
            CalendarEvent::Select(date) => {
                let mut next = with_focus(state, date);
                if !state.controlled {
                    next.value = Some(date);
                }
                Transition::with(next, vec![CalendarCommand::ValueChange(date)])
            }
            CalendarEvent::Focus(date) => Transition::to(with_focus(state, input.clamp(date))),
            CalendarEvent::Move(by) => {
                let date = input.clamp(moved(state.focused, by, input.week_start));
                if date == state.focused {
                    return Transition::stay();
                }
                Transition::with(with_focus(state, date), vec![CalendarCommand::FocusDay(date)])
            }
            CalendarEvent::ShiftMonth(n) => Transition::to(with_focus(state, input.clamp(state.focused.add_months(n)))),
        }
    }

    fn sync(state: &CalendarState, input: &CalendarInput) -> Option<CalendarState> {
        let mut next = state.clone();
        if state.controlled && input.value.is_some() && input.value != state.value {
            next.value = input.value;
            next.focused = input.value.unwrap_or(state.focused);
        }
        next.focused = input.clamp(next.focused);
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Date {
        s.parse().unwrap()
    }

    fn input() -> CalendarInput {
        CalendarInput { today: d("2026-10-26"), week_start: 1, ..Default::default() }
    }

    fn run(
        state: &CalendarState,
        event: CalendarEvent,
        input: &CalendarInput,
    ) -> hydroxy_statechart::Step<CalendarState, CalendarCommand> {
        CalendarMachine::transition(state, event, input).resolve(state)
    }

    #[test]
    fn starts_on_value_else_today() {
        assert_eq!(CalendarMachine::initial(&input()).focused, d("2026-10-26"));
        let i = CalendarInput { default_value: Some(d("2026-03-04")), ..input() };
        let s = CalendarMachine::initial(&i);
        assert_eq!((s.value, s.focused, s.controlled), (Some(d("2026-03-04")), d("2026-03-04"), false));
    }

    #[test]
    fn select_commits_when_uncontrolled() {
        let i = input();
        let step = run(&CalendarMachine::initial(&i), CalendarEvent::Select(d("2026-10-30")), &i);
        assert_eq!(step.state.value, Some(d("2026-10-30")));
        assert_eq!(step.commands, vec![CalendarCommand::ValueChange(d("2026-10-30"))]);
        let again = run(&step.state, CalendarEvent::Select(d("2026-10-30")), &i);
        assert!(again.commands.is_empty());
    }

    #[test]
    fn controlled_requests_and_follows_sync() {
        let mut i = CalendarInput { value: Some(d("2026-10-01")), ..input() };
        let step = run(&CalendarMachine::initial(&i), CalendarEvent::Select(d("2026-10-30")), &i);
        assert_eq!(step.state.value, Some(d("2026-10-01")));
        assert_eq!(step.commands, vec![CalendarCommand::ValueChange(d("2026-10-30"))]);
        i.value = Some(d("2027-01-05"));
        let synced = hydroxy_statechart::sync_or_keep::<CalendarMachine>(&step.state, &i);
        assert_eq!((synced.value, synced.focused), (Some(d("2027-01-05")), d("2027-01-05")));
    }

    #[test]
    fn unavailable_and_out_of_range_days_are_not_selectable() {
        let i = CalendarInput { unavailable: vec![d("2026-10-27")], max: Some(d("2026-10-28")), ..input() };
        let s = CalendarMachine::initial(&i);
        assert!(run(&s, CalendarEvent::Select(d("2026-10-27")), &i).commands.is_empty());
        assert!(run(&s, CalendarEvent::Select(d("2026-10-29")), &i).commands.is_empty());
        let disabled = CalendarInput { disabled: true, ..input() };
        assert!(run(&s, CalendarEvent::Select(d("2026-10-26")), &disabled).commands.is_empty());
    }

    #[test]
    fn keys_move_focus_and_request_dom_focus() {
        let i = input();
        let s = CalendarMachine::initial(&i);
        let cases = [
            (CalendarMove::Days(1), "2026-10-27"),
            (CalendarMove::Days(-7), "2026-10-19"),
            (CalendarMove::Months(1), "2026-11-26"),
            (CalendarMove::Years(-1), "2025-10-26"),
            (CalendarMove::WeekStart, "2026-10-26"),
            (CalendarMove::WeekEnd, "2026-11-01"),
        ];
        for (by, want) in cases {
            let step = run(&s, CalendarEvent::Move(by), &i);
            if d(want) == s.focused {
                assert!(step.commands.is_empty());
            } else {
                assert_eq!(step.state.focused, d(want), "{by:?}");
                assert_eq!(step.commands, vec![CalendarCommand::FocusDay(d(want))]);
            }
        }
        let sunday_start = CalendarInput { week_start: 0, ..input() };
        assert_eq!(run(&s, CalendarEvent::Move(CalendarMove::WeekStart), &sunday_start).state.focused, d("2026-10-25"));
    }

    #[test]
    fn moves_clamp_to_min_and_max() {
        let i = CalendarInput { min: Some(d("2026-10-20")), max: Some(d("2026-11-10")), ..input() };
        let s = CalendarMachine::initial(&i);
        assert_eq!(run(&s, CalendarEvent::Move(CalendarMove::Months(1)), &i).state.focused, d("2026-11-10"));
        assert_eq!(run(&s, CalendarEvent::Move(CalendarMove::Days(-7)), &i).state.focused, d("2026-10-20"));
        let at_min = CalendarState { focused: d("2026-10-20"), ..s };
        assert!(run(&at_min, CalendarEvent::Move(CalendarMove::Days(-1)), &i).commands.is_empty());
    }

    #[test]
    fn month_buttons_shift_the_view_without_moving_dom_focus() {
        let i = CalendarInput { min: Some(d("2026-09-15")), ..input() };
        let s = CalendarState { focused: d("2026-10-31"), ..CalendarMachine::initial(&i) };
        let next = run(&s, CalendarEvent::ShiftMonth(1), &i);
        assert_eq!(next.state.focused, d("2026-11-30"));
        assert!(next.commands.is_empty());
        assert!(i.can_shift(s.focused, -1));
        assert!(!i.can_shift(s.focused, -2));
        assert_eq!(run(&s, CalendarEvent::ShiftMonth(-1), &i).state.focused, d("2026-09-30"));
    }

    #[test]
    fn sync_clamps_focus_into_a_new_range() {
        let s = CalendarMachine::initial(&input());
        let i = CalendarInput { max: Some(d("2026-10-01")), ..input() };
        assert_eq!(hydroxy_statechart::sync_or_keep::<CalendarMachine>(&s, &i).focused, d("2026-10-01"));
    }
}
