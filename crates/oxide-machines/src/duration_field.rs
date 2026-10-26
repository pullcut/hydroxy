//! Duration field: hours, minutes and seconds typed into separate segments.
//! Empty segments count as zero, so the value is known as soon as one is filled.

use oxide_statechart::{Machine, Transition};

use crate::segments::{self, Spec};

/// One part of the duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationSegment {
    /// Hours, 0–99.
    Hours,
    /// Minutes, 0–59.
    Minutes,
    /// Seconds, 0–59.
    Seconds,
}

impl DurationSegment {
    /// The range and width.
    #[must_use]
    pub const fn spec(self) -> Spec {
        match self {
            Self::Hours => Spec { min: 0, max: 99, width: 2 },
            Self::Minutes | Self::Seconds => Spec { min: 0, max: 59, width: 2 },
        }
    }
}

/// Props for [`DurationFieldMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DurationFieldInput {
    /// Controlled value in seconds. `Some` at start fixes controlled mode.
    pub value: Option<u32>,
    /// Uncontrolled initial value in seconds.
    pub default_value: Option<u32>,
    /// Show a seconds segment. Without it, seconds stay 0.
    pub show_seconds: bool,
    /// Ignore every change.
    pub disabled: bool,
    /// Focusable but not editable.
    pub read_only: bool,
}

impl DurationFieldInput {
    /// The segments on show, left to right.
    #[must_use]
    pub fn segments(&self) -> &'static [DurationSegment] {
        use DurationSegment::{Hours, Minutes, Seconds};
        if self.show_seconds { &[Hours, Minutes, Seconds] } else { &[Hours, Minutes] }
    }
}

/// What [`DurationFieldMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DurationFieldState {
    /// Hours typed so far.
    pub hours: Option<u32>,
    /// Minutes typed so far.
    pub minutes: Option<u32>,
    /// Seconds typed so far.
    pub seconds: Option<u32>,
    /// The segment being typed into, and how many digits it has.
    pub typing: Option<(DurationSegment, u8)>,
    /// Fixed at start: the owner holds the value.
    pub controlled: bool,
}

impl DurationFieldState {
    /// A segment's value.
    #[must_use]
    pub fn get(&self, segment: DurationSegment) -> Option<u32> {
        match segment {
            DurationSegment::Hours => self.hours,
            DurationSegment::Minutes => self.minutes,
            DurationSegment::Seconds => self.seconds,
        }
    }

    fn set(&mut self, segment: DurationSegment, value: Option<u32>) {
        match segment {
            DurationSegment::Hours => self.hours = value,
            DurationSegment::Minutes => self.minutes = value,
            DurationSegment::Seconds => self.seconds = value,
        }
    }

    /// Total seconds, or `None` while every segment is empty.
    #[must_use]
    pub fn value(&self) -> Option<u32> {
        if self.hours.is_none() && self.minutes.is_none() && self.seconds.is_none() {
            return None;
        }
        Some(self.hours.unwrap_or(0) * 3600 + self.minutes.unwrap_or(0) * 60 + self.seconds.unwrap_or(0))
    }

    /// A segment's text, dashes while empty.
    #[must_use]
    pub fn text(&self, segment: DurationSegment) -> String {
        segments::text(self.get(segment), segment.spec().width)
    }

    fn from_seconds(value: Option<u32>, controlled: bool) -> Self {
        let split = |v: u32| (v / 3600, v % 3600 / 60, v % 60);
        let (hours, minutes, seconds) = value.map_or((None, None, None), |v| {
            let (h, m, s) = split(v.min(99 * 3600 + 59 * 60 + 59));
            (Some(h), Some(m), Some(s))
        });
        Self { hours, minutes, seconds, typing: None, controlled }
    }
}

/// Inputs to [`DurationFieldMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationFieldEvent {
    /// A digit key in a segment.
    Digit {
        /// Which segment.
        segment: DurationSegment,
        /// 0–9.
        digit: u32,
    },
    /// Arrow Up or Down in a segment.
    Step {
        /// Which segment.
        segment: DurationSegment,
        /// Up (`true`) or down.
        up: bool,
    },
    /// Home (lowest) or End (highest) in a segment.
    Jump {
        /// Which segment.
        segment: DurationSegment,
        /// End (`true`) or Home.
        to_max: bool,
    },
    /// Backspace or Delete: empty the segment.
    Clear(DurationSegment),
    /// A segment gained focus: the next digit starts fresh.
    Focus,
}

/// Side effects from [`DurationFieldMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationFieldCommand {
    /// The total seconds changed (or should, when controlled). `None` when empty.
    ValueChange(Option<u32>),
    /// Move DOM focus to this segment.
    FocusSegment(DurationSegment),
}

/// Segmented duration entry.
#[derive(Debug, Clone, Copy)]
pub struct DurationFieldMachine;

impl Machine for DurationFieldMachine {
    type Input = DurationFieldInput;
    type State = DurationFieldState;
    type Event = DurationFieldEvent;
    type Command = DurationFieldCommand;

    fn initial(input: &DurationFieldInput) -> DurationFieldState {
        DurationFieldState::from_seconds(input.value.or(input.default_value), input.value.is_some())
    }

    fn transition(
        state: &DurationFieldState,
        event: DurationFieldEvent,
        input: &DurationFieldInput,
    ) -> Transition<DurationFieldState, DurationFieldCommand> {
        if input.disabled || input.read_only {
            return Transition::stay(state.clone());
        }
        let mut next = state.clone();
        let mut focus = None;
        match event {
            DurationFieldEvent::Focus => next.typing = None,
            DurationFieldEvent::Digit { segment, digit } => {
                let prev_digits = state.typing.filter(|(s, _)| *s == segment).map_or(0, |(_, n)| n);
                let typed = segments::type_digit(state.get(segment), prev_digits, digit, segment.spec());
                next.set(segment, Some(typed.value));
                next.typing = Some((segment, typed.digits));
                if typed.advance {
                    next.typing = None;
                    let all = input.segments();
                    focus = all.iter().position(|s| *s == segment).and_then(|i| all.get(i + 1)).copied();
                }
            }
            DurationFieldEvent::Step { segment, up } => {
                next.set(segment, Some(segments::step(state.get(segment), up, segment.spec(), 0)));
                next.typing = None;
            }
            DurationFieldEvent::Jump { segment, to_max } => {
                let spec = segment.spec();
                next.set(segment, Some(if to_max { spec.max } else { spec.min }));
                next.typing = None;
            }
            DurationFieldEvent::Clear(segment) => {
                next.set(segment, None);
                next.typing = None;
            }
        }
        let mut commands = Vec::new();
        if next.value() != state.value() {
            commands.push(DurationFieldCommand::ValueChange(next.value()));
        }
        if let Some(segment) = focus {
            commands.push(DurationFieldCommand::FocusSegment(segment));
        }
        Transition { state: next, commands }
    }

    fn sync(state: &DurationFieldState, input: &DurationFieldInput) -> DurationFieldState {
        if state.controlled && input.value != state.value() {
            DurationFieldState::from_seconds(input.value, true)
        } else {
            state.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use DurationSegment::{Hours, Minutes, Seconds};

    fn run(input: &DurationFieldInput, events: &[DurationFieldEvent]) -> (DurationFieldState, Vec<DurationFieldCommand>) {
        let mut state = DurationFieldMachine::initial(input);
        let mut all = Vec::new();
        for e in events {
            let t = DurationFieldMachine::transition(&state, *e, input);
            state = t.state;
            all.extend(t.commands);
        }
        (state, all)
    }

    fn digits(segment: DurationSegment, s: &str) -> Vec<DurationFieldEvent> {
        s.chars().map(|c| DurationFieldEvent::Digit { segment, digit: c.to_digit(10).unwrap() }).collect()
    }

    #[test]
    fn typing_totals_seconds() {
        let input = DurationFieldInput { show_seconds: true, ..Default::default() };
        let mut events = digits(Hours, "01");
        events.extend(digits(Minutes, "30"));
        events.extend(digits(Seconds, "15"));
        let (state, commands) = run(&input, &events);
        assert_eq!(state.value(), Some(5415));
        assert_eq!(commands.last(), Some(&DurationFieldCommand::ValueChange(Some(5415))));
        assert!(commands.contains(&DurationFieldCommand::FocusSegment(Minutes)));
    }

    #[test]
    fn empty_segments_count_as_zero() {
        let input = DurationFieldInput::default();
        let (state, commands) = run(&input, &digits(Minutes, "7"));
        assert_eq!(state.value(), Some(420));
        // Minutes is the last segment without seconds, so focus stays.
        assert_eq!(commands, [DurationFieldCommand::ValueChange(Some(420))]);
        assert_eq!(state.text(Hours), "––");
    }

    #[test]
    fn minutes_wrap_and_clear() {
        let input = DurationFieldInput { default_value: Some(0), ..Default::default() };
        let (state, _) = run(&input, &[DurationFieldEvent::Step { segment: Minutes, up: false }]);
        assert_eq!((state.minutes, state.value()), (Some(59), Some(3540)));
        let (state, commands) = run(&input, &[DurationFieldEvent::Clear(Hours), DurationFieldEvent::Clear(Minutes), DurationFieldEvent::Clear(Seconds)]);
        assert_eq!(state.value(), None);
        assert_eq!(commands.last(), Some(&DurationFieldCommand::ValueChange(None)));
    }

    #[test]
    fn controlled_follows_owner() {
        let input = DurationFieldInput { value: Some(90), ..Default::default() };
        let state = DurationFieldMachine::initial(&input);
        let moved = DurationFieldInput { value: Some(3600), ..input };
        assert_eq!(DurationFieldMachine::sync(&state, &moved).value(), Some(3600));
    }
}
