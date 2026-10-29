//! Date field: day, month and year typed into separate segments. Digits fill a
//! segment and move on when no further digit could fit; arrows step and wrap.
//! "Today" arrives as input, so an empty segment starts from it without a clock.

use hydroxy_statechart::{Machine, Transition};

use crate::date::{Date, days_in_month};
use crate::list::Direction;
use crate::segments::{self, Spec};

/// One part of the date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DateSegment {
    /// Day of the month.
    Day,
    /// Month, 1–12.
    Month,
    /// Four-digit year.
    Year,
}

/// The order segments appear in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum DateOrder {
    /// Day, month, year: 26/10/2026.
    #[default]
    Dmy,
    /// Month, day, year: 10/26/2026.
    Mdy,
    /// Year, month, day: 2026/10/26.
    Ymd,
}

impl DateOrder {
    /// The segments left to right.
    #[must_use]
    pub const fn segments(self) -> [DateSegment; 3] {
        use DateSegment::{Day, Month, Year};
        match self {
            Self::Dmy => [Day, Month, Year],
            Self::Mdy => [Month, Day, Year],
            Self::Ymd => [Year, Month, Day],
        }
    }

    /// The segment after `from`, if any.
    #[must_use]
    pub fn next(self, from: DateSegment) -> Option<DateSegment> {
        let all = self.segments();
        let at = all.iter().position(|s| *s == from)?;
        all.get(at + 1).copied()
    }
}

/// Props for [`DateFieldMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateFieldInput {
    /// Controlled value. `Some` at start fixes controlled mode.
    pub value: Option<Date>,
    /// Uncontrolled initial value.
    pub default_value: Option<Date>,
    /// Earliest valid date. Earlier ones mark the field invalid.
    pub min: Option<Date>,
    /// Latest valid date.
    pub max: Option<Date>,
    /// Today: where an empty segment starts when stepped.
    pub today: Date,
    /// Segment order.
    pub order: DateOrder,
    /// Ignore every change.
    pub disabled: bool,
    /// Focusable but not editable.
    pub read_only: bool,
}

impl Default for DateFieldInput {
    fn default() -> Self {
        Self {
            value: None,
            default_value: None,
            min: None,
            max: None,
            today: Date::default(),
            order: DateOrder::Dmy,
            disabled: false,
            read_only: false,
        }
    }
}

/// What [`DateFieldMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DateFieldState {
    /// Day typed so far.
    pub day: Option<u32>,
    /// Month typed so far.
    pub month: Option<u32>,
    /// Year typed so far.
    pub year: Option<u32>,
    /// The segment being typed into, and how many digits it has.
    pub typing: Option<(DateSegment, u8)>,
    /// Fixed at start: the owner holds the value.
    pub controlled: bool,
}

impl DateFieldState {
    /// A segment's value.
    #[must_use]
    pub fn get(&self, segment: DateSegment) -> Option<u32> {
        match segment {
            DateSegment::Day => self.day,
            DateSegment::Month => self.month,
            DateSegment::Year => self.year,
        }
    }

    fn set(&mut self, segment: DateSegment, value: Option<u32>) {
        match segment {
            DateSegment::Day => self.day = value,
            DateSegment::Month => self.month = value,
            DateSegment::Year => self.year = value,
        }
    }

    /// The range of `segment` given the others (a day depends on month and year).
    #[must_use]
    pub fn spec(&self, segment: DateSegment) -> Spec {
        match segment {
            DateSegment::Day => {
                let year = self.year.and_then(|y| i32::try_from(y).ok()).unwrap_or(2000);
                let month = self.month.and_then(|m| u8::try_from(m).ok()).unwrap_or(1);
                Spec { min: 1, max: u32::from(days_in_month(year, month)), width: 2 }
            }
            DateSegment::Month => Spec { min: 1, max: 12, width: 2 },
            DateSegment::Year => Spec { min: 1, max: 9999, width: 4 },
        }
    }

    /// The date the segments spell, if it is a real one.
    #[must_use]
    pub fn date(&self) -> Option<Date> {
        Date::new(i32::try_from(self.year?).ok()?, u8::try_from(self.month?).ok()?, u8::try_from(self.day?).ok()?)
    }

    /// The date, if it is also within `min`/`max`.
    #[must_use]
    pub fn value(&self, input: &DateFieldInput) -> Option<Date> {
        self.date().filter(|d| input.min.is_none_or(|m| *d >= m) && input.max.is_none_or(|m| *d <= m))
    }

    /// Every segment is filled but they don't make an allowed date.
    #[must_use]
    pub fn invalid(&self, input: &DateFieldInput) -> bool {
        self.day.is_some() && self.month.is_some() && self.year.is_some() && self.value(input).is_none()
    }

    /// A segment's text, dashes while empty.
    #[must_use]
    pub fn text(&self, segment: DateSegment) -> String {
        segments::text(self.get(segment), self.spec(segment).width)
    }

    fn from_date(date: Option<Date>, controlled: bool) -> Self {
        Self {
            day: date.map(|d| u32::from(d.day())),
            month: date.map(|d| u32::from(d.month())),
            year: date.and_then(|d| u32::try_from(d.year()).ok()),
            typing: None,
            controlled,
        }
    }

    /// A day past the end of its month falls back to the last day.
    fn clamp_day(&mut self) {
        let max = self.spec(DateSegment::Day).max;
        self.day = self.day.map(|d| d.min(max));
    }
}

/// Inputs to [`DateFieldMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DateFieldEvent {
    /// A digit key in a segment.
    Digit {
        /// Which segment.
        segment: DateSegment,
        /// 0–9.
        digit: u32,
    },
    /// Arrow Up or Down in a segment.
    Step {
        /// Which segment.
        segment: DateSegment,
        /// Up ([`Direction::Next`]) or down.
        direction: Direction,
    },
    /// Home (lowest) or End (highest) in a segment.
    Jump {
        /// Which segment.
        segment: DateSegment,
        /// End (`true`) or Home.
        to_max: bool,
    },
    /// Backspace or Delete: empty the segment.
    Clear(DateSegment),
    /// A segment gained focus: the next digit starts fresh.
    Focus,
}

/// Side effects from [`DateFieldMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DateFieldCommand {
    /// The date changed (or should, when controlled). `None` when incomplete or not allowed.
    ValueChange(Option<Date>),
    /// Move DOM focus to this segment.
    FocusSegment(DateSegment),
}

/// Segmented date entry.
#[derive(Debug, Clone, Copy)]
pub struct DateFieldMachine;

impl Machine for DateFieldMachine {
    type Input = DateFieldInput;
    type State = DateFieldState;
    type Event = DateFieldEvent;
    type Command = DateFieldCommand;

    fn initial(input: &DateFieldInput) -> DateFieldState {
        DateFieldState::from_date(input.value.or(input.default_value), input.value.is_some())
    }

    fn transition(
        state: &DateFieldState,
        event: DateFieldEvent,
        input: &DateFieldInput,
    ) -> Transition<DateFieldState, DateFieldCommand> {
        if input.disabled || input.read_only {
            return Transition::stay();
        }
        let mut next = state.clone();
        let mut focus = None;
        match event {
            DateFieldEvent::Focus => next.typing = None,
            DateFieldEvent::Digit { segment, digit } => {
                let prev_digits = state.typing.filter(|(s, _)| *s == segment).map_or(0, |(_, n)| n);
                let typed = segments::type_digit(state.get(segment), prev_digits, digit, state.spec(segment));
                next.set(segment, Some(typed.value));
                next.typing = Some((segment, typed.digits));
                if typed.advance {
                    next.typing = None;
                    focus = input.order.next(segment);
                }
            }
            DateFieldEvent::Step { segment, direction } => {
                let start = match segment {
                    DateSegment::Day => u32::from(input.today.day()),
                    DateSegment::Month => u32::from(input.today.month()),
                    DateSegment::Year => u32::try_from(input.today.year()).unwrap_or(1),
                };
                next.set(segment, Some(segments::step(state.get(segment), direction, state.spec(segment), start)));
                next.typing = None;
            }
            DateFieldEvent::Jump { segment, to_max } => {
                let spec = state.spec(segment);
                next.set(segment, Some(if to_max { spec.max } else { spec.min }));
                next.typing = None;
            }
            DateFieldEvent::Clear(segment) => {
                next.set(segment, None);
                next.typing = None;
            }
        }
        next.clamp_day();
        let mut commands = Vec::new();
        let value = next.value(input);
        if value != state.value(input) {
            commands.push(DateFieldCommand::ValueChange(value));
        }
        if let Some(segment) = focus {
            commands.push(DateFieldCommand::FocusSegment(segment));
        }
        Transition::update(state, next, commands)
    }

    fn sync(state: &DateFieldState, input: &DateFieldInput) -> Option<DateFieldState> {
        Some(if state.controlled && input.value != state.value(input) {
            DateFieldState::from_date(input.value, true)
        } else {
            state.clone()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::d;
    use DateSegment::{Day, Month, Year};
    use hydroxy_statechart::step as advance;

    fn run(input: &DateFieldInput, events: &[DateFieldEvent]) -> (DateFieldState, Vec<DateFieldCommand>) {
        let mut state = DateFieldMachine::initial(input);
        let mut all = Vec::new();
        for e in events {
            let t = advance::<DateFieldMachine>(&state, *e, input);
            state = t.state;
            all.extend(t.commands);
        }
        (state, all)
    }

    fn digits(segment: DateSegment, s: &str) -> Vec<DateFieldEvent> {
        s.chars().map(|c| DateFieldEvent::Digit { segment, digit: c.to_digit(10).unwrap() }).collect()
    }

    #[test]
    fn typing_fills_and_advances() {
        let input = DateFieldInput::default();
        let mut events = digits(Day, "31");
        events.extend(digits(Month, "12"));
        events.extend(digits(Year, "2026"));
        let (state, commands) = run(&input, &events);
        assert_eq!(state.value(&input), Some(d("2026-12-31")));
        assert_eq!(commands.first(), Some(&DateFieldCommand::FocusSegment(Month)));
        assert_eq!(commands.last(), Some(&DateFieldCommand::ValueChange(Some(d("2026-12-31")))));
        // A single large digit moves on at once.
        let (_, commands) = run(&input, &digits(Day, "4"));
        assert_eq!(commands, [DateFieldCommand::FocusSegment(Month)]);
    }

    #[test]
    fn order_decides_where_focus_goes() {
        let input = DateFieldInput { order: DateOrder::Mdy, ..Default::default() };
        let (_, commands) = run(&input, &digits(Month, "5"));
        assert_eq!(commands, [DateFieldCommand::FocusSegment(Day)]);
        let ymd = DateFieldInput { order: DateOrder::Ymd, ..Default::default() };
        assert_eq!(ymd.order.next(Day), None);
    }

    #[test]
    fn changing_month_clamps_the_day() {
        let input = DateFieldInput { default_value: Some(d("2026-03-31")), ..Default::default() };
        let (state, commands) = run(&input, &[DateFieldEvent::Step { segment: Month, direction: Direction::Prev }]);
        assert_eq!(state.value(&input), Some(d("2026-02-28")));
        assert_eq!(commands, [DateFieldCommand::ValueChange(Some(d("2026-02-28")))]);
    }

    #[test]
    fn empty_segments_start_from_today_and_wrap() {
        let input = DateFieldInput { today: d("2026-10-26"), ..Default::default() };
        let (state, _) = run(&input, &[DateFieldEvent::Step { segment: Year, direction: Direction::Next }]);
        assert_eq!(state.year, Some(2026));
        let (state, _) = run(
            &input,
            &[
                DateFieldEvent::Jump { segment: Month, to_max: true },
                DateFieldEvent::Step { segment: Month, direction: Direction::Next },
            ],
        );
        assert_eq!(state.month, Some(1));
    }

    #[test]
    fn clearing_reports_none_and_range_marks_invalid() {
        let input =
            DateFieldInput { default_value: Some(d("2026-03-04")), max: Some(d("2026-12-31")), ..Default::default() };
        let (state, commands) = run(&input, &[DateFieldEvent::Clear(Day)]);
        assert_eq!(commands, [DateFieldCommand::ValueChange(None)]);
        assert!(!state.invalid(&input));
        let (state, _) = run(&input, &digits(Year, "2027"));
        assert!(state.invalid(&input));
        assert_eq!(state.value(&input), None);
    }

    #[test]
    fn controlled_follows_owner() {
        let input = DateFieldInput { value: Some(d("2026-03-04")), ..Default::default() };
        let state = DateFieldMachine::initial(&input);
        assert!(state.controlled);
        let moved = DateFieldInput { value: Some(d("2027-01-02")), ..input.clone() };
        assert_eq!(
            hydroxy_statechart::sync_or_keep::<DateFieldMachine>(&state, &moved).value(&moved),
            Some(d("2027-01-02"))
        );
        // Half-typed text survives while the owner's value is still None.
        let empty = DateFieldInput { value: None, ..Default::default() };
        let typed = DateFieldState { day: Some(3), controlled: true, ..Default::default() };
        assert_eq!(hydroxy_statechart::sync_or_keep::<DateFieldMachine>(&typed, &empty).day, Some(3));
    }

    #[test]
    fn disabled_and_read_only_ignore_input() {
        let input = DateFieldInput { read_only: true, ..Default::default() };
        let (state, commands) = run(&input, &digits(Day, "1"));
        assert_eq!(state.day, None);
        assert!(commands.is_empty());
    }
}
