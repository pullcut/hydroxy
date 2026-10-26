//! Range slider owns a `[low, high]` pair. Each thumb is bounded by the range
//! and by the other thumb plus `min_gap`, so the pair never crosses.
//! Pointer geometry is the adapter's job; it sends values, the machine snaps.

use oxide_statechart::{Machine, Transition};

pub use crate::ChangeReason;

/// Which thumb: `0` is low, `1` is high.
pub type Thumb = usize;

/// Props for [`RangeSliderMachine`].
#[derive(Debug, Clone, PartialEq)]
pub struct RangeSliderInput {
    /// Lowest value.
    pub min: f64,
    /// Highest value.
    pub max: f64,
    /// Step between values. Values snap to `min + n * step`.
    pub step: f64,
    /// Smallest distance kept between the two thumbs.
    pub min_gap: f64,
    /// Controlled pair. `Some` at start fixes controlled mode.
    pub value: Option<[f64; 2]>,
    /// Uncontrolled initial pair. `None` means `[min, max]`.
    pub default_value: Option<[f64; 2]>,
    /// Ignore all input.
    pub disabled: bool,
}

impl Default for RangeSliderInput {
    fn default() -> Self {
        Self { min: 0.0, max: 100.0, step: 1.0, min_gap: 0.0, value: None, default_value: None, disabled: false }
    }
}

impl RangeSliderInput {
    /// Snap `v` to the step grid and clamp it to `[lo, hi]`.
    fn snap(&self, v: f64, lo: f64, hi: f64) -> f64 {
        let snapped = if self.step > 0.0 { self.min + ((v - self.min) / self.step).round() * self.step } else { v };
        // Trim float noise such as 0.30000000000000004.
        let snapped = (snapped * 1e9).round() / 1e9;
        snapped.clamp(lo, hi.max(lo))
    }

    /// The bounds thumb `thumb` may move within, given the other thumb's value.
    #[must_use]
    pub fn bounds(&self, value: [f64; 2], thumb: Thumb) -> (f64, f64) {
        if thumb == 0 { (self.min, value[1] - self.min_gap) } else { (value[0] + self.min_gap, self.max) }
    }

    /// Order, snap and clamp a pair so it is valid for these props.
    #[must_use]
    pub fn normalize(&self, [a, b]: [f64; 2]) -> [f64; 2] {
        let (a, b) = if a <= b { (a, b) } else { (b, a) };
        let low = self.snap(a, self.min, self.max - self.min_gap);
        let high = self.snap(b, low + self.min_gap, self.max);
        [low, high]
    }

    /// Move one thumb to `v`, bounded by the other.
    fn set(&self, value: [f64; 2], thumb: Thumb, v: f64) -> [f64; 2] {
        let (lo, hi) = self.bounds(value, thumb);
        let mut next = value;
        next[thumb] = self.snap(v, lo, hi);
        next
    }
}

/// What [`RangeSliderMachine`] remembers between events.
#[derive(Debug, Clone, PartialEq)]
pub struct RangeSliderState {
    /// The current `[low, high]` pair.
    pub value: [f64; 2],
    /// The thumb being dragged, if any.
    pub dragging: Option<Thumb>,
    /// Fixed at start: the owner holds the value and the machine only requests changes.
    pub controlled: bool,
}

/// Inputs to [`RangeSliderMachine`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RangeSliderEvent {
    /// Move one thumb to a value (keyboard or code).
    Set(Thumb, f64, ChangeReason),
    /// Pointer down on the track at a value: the nearest thumb jumps there and starts dragging.
    PointerDown(f64),
    /// Pointer moved while dragging, at a value.
    PointerMove(f64),
    /// Pointer released or cancelled.
    PointerUp,
}

/// Side effects from [`RangeSliderMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RangeSliderCommand {
    /// Tell the owner the pair changed (or should, when controlled).
    ValueChange {
        /// The new (or proposed) pair.
        value: [f64; 2],
        /// What caused the change.
        reason: ChangeReason,
    },
    /// Move focus to this thumb.
    FocusThumb(Thumb),
}

/// A two-thumb range.
#[derive(Debug, Clone, Copy)]
pub struct RangeSliderMachine;

impl RangeSliderMachine {
    /// The thumb closest to `v`. Ties go to the side `v` lies on.
    #[must_use]
    pub fn nearest(value: [f64; 2], v: f64) -> Thumb {
        let (d0, d1) = ((v - value[0]).abs(), (v - value[1]).abs());
        match d0.partial_cmp(&d1) {
            Some(std::cmp::Ordering::Less) => 0,
            Some(std::cmp::Ordering::Greater) => 1,
            _ => usize::from(v >= value[0]),
        }
    }

    #[allow(clippy::float_cmp, reason = "exact change detection on snapped values")]
    fn change(state: &RangeSliderState, next: RangeSliderState, value: [f64; 2], reason: ChangeReason, focus: Option<Thumb>) -> Transition<RangeSliderState, RangeSliderCommand> {
        let mut commands: Vec<_> = focus.map(RangeSliderCommand::FocusThumb).into_iter().collect();
        let mut next = next;
        if value != state.value {
            commands.push(RangeSliderCommand::ValueChange { value, reason });
            if !state.controlled {
                next.value = value;
            }
        }
        Transition { state: next, commands }
    }
}

impl Machine for RangeSliderMachine {
    type Input = RangeSliderInput;
    type State = RangeSliderState;
    type Event = RangeSliderEvent;
    type Command = RangeSliderCommand;

    fn initial(input: &RangeSliderInput) -> RangeSliderState {
        let pair = input.value.or(input.default_value).unwrap_or([input.min, input.max]);
        RangeSliderState { value: input.normalize(pair), dragging: None, controlled: input.value.is_some() }
    }

    fn transition(
        state: &RangeSliderState,
        event: RangeSliderEvent,
        input: &RangeSliderInput,
    ) -> Transition<RangeSliderState, RangeSliderCommand> {
        if input.disabled {
            let mut next = state.clone();
            next.dragging = None;
            return Transition::stay(next);
        }
        match event {
            RangeSliderEvent::Set(thumb, v, reason) => {
                let thumb = thumb.min(1);
                Self::change(state, state.clone(), input.set(state.value, thumb, v), reason, None)
            }
            RangeSliderEvent::PointerDown(v) => {
                let thumb = Self::nearest(state.value, v);
                let next = RangeSliderState { dragging: Some(thumb), ..state.clone() };
                Self::change(state, next, input.set(state.value, thumb, v), ChangeReason::Pointer, Some(thumb))
            }
            RangeSliderEvent::PointerMove(v) => match state.dragging {
                Some(thumb) => Self::change(state, state.clone(), input.set(state.value, thumb, v), ChangeReason::Pointer, None),
                None => Transition::stay(state.clone()),
            },
            RangeSliderEvent::PointerUp => Transition::stay(RangeSliderState { dragging: None, ..state.clone() }),
        }
    }

    fn sync(state: &RangeSliderState, input: &RangeSliderInput) -> RangeSliderState {
        let mut next = state.clone();
        next.value = match (state.controlled, input.value) {
            (true, Some(value)) => input.normalize(value),
            // Props such as min, max or min_gap may have moved under an uncontrolled pair.
            _ => input.normalize(state.value),
        };
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn send(state: &RangeSliderState, event: RangeSliderEvent, input: &RangeSliderInput) -> Transition<RangeSliderState, RangeSliderCommand> {
        RangeSliderMachine::transition(state, event, input)
    }

    #[test]
    fn initial_defaults_to_full_range_and_orders_pairs() {
        let input = RangeSliderInput::default();
        assert_eq!(RangeSliderMachine::initial(&input).value, [0.0, 100.0]);
        let input = RangeSliderInput { default_value: Some([80.0, 20.0]), ..input };
        assert_eq!(RangeSliderMachine::initial(&input).value, [20.0, 80.0]);
    }

    #[test]
    fn thumbs_cannot_cross_or_break_the_gap() {
        let input = RangeSliderInput { default_value: Some([20.0, 60.0]), min_gap: 10.0, ..RangeSliderInput::default() };
        let state = RangeSliderMachine::initial(&input);
        let step = send(&state, RangeSliderEvent::Set(0, 90.0, ChangeReason::Keyboard), &input);
        assert_eq!(step.state.value, [50.0, 60.0]);
        let step = send(&step.state, RangeSliderEvent::Set(1, 0.0, ChangeReason::Keyboard), &input);
        assert_eq!(step.state.value, [50.0, 60.0]);
        assert!(step.commands.is_empty());
        assert_eq!(input.bounds([20.0, 60.0], 0), (0.0, 50.0));
        assert_eq!(input.bounds([20.0, 60.0], 1), (30.0, 100.0));
    }

    #[test]
    fn values_snap_to_step() {
        let input = RangeSliderInput { step: 0.1, max: 1.0, ..RangeSliderInput::default() };
        let state = RangeSliderMachine::initial(&input);
        let step = send(&state, RangeSliderEvent::Set(0, 0.3333, ChangeReason::Keyboard), &input);
        assert_eq!(step.state.value, [0.3, 1.0]);
        let input = RangeSliderInput { step: 5.0, ..RangeSliderInput::default() };
        let step = send(&RangeSliderMachine::initial(&input), RangeSliderEvent::Set(1, 42.0, ChangeReason::Keyboard), &input);
        assert_eq!(step.state.value, [0.0, 40.0]);
    }

    #[test]
    fn pointer_down_moves_nearest_thumb_and_drags_it() {
        let input = RangeSliderInput { default_value: Some([20.0, 80.0]), ..RangeSliderInput::default() };
        let state = RangeSliderMachine::initial(&input);
        let step = send(&state, RangeSliderEvent::PointerDown(70.0), &input);
        assert_eq!(step.state.value, [20.0, 70.0]);
        assert_eq!(step.state.dragging, Some(1));
        assert_eq!(
            step.commands,
            vec![
                RangeSliderCommand::FocusThumb(1),
                RangeSliderCommand::ValueChange { value: [20.0, 70.0], reason: ChangeReason::Pointer }
            ]
        );
        let step = send(&step.state, RangeSliderEvent::PointerMove(10.0), &input);
        assert_eq!(step.state.value, [20.0, 20.0]);
        let step = send(&step.state, RangeSliderEvent::PointerUp, &input);
        assert_eq!(step.state.dragging, None);
        assert!(send(&step.state, RangeSliderEvent::PointerMove(90.0), &input).commands.is_empty());
    }

    #[test]
    fn nearest_breaks_ties_by_side() {
        assert_eq!(RangeSliderMachine::nearest([50.0, 50.0], 40.0), 0);
        assert_eq!(RangeSliderMachine::nearest([50.0, 50.0], 60.0), 1);
        assert_eq!(RangeSliderMachine::nearest([0.0, 100.0], 49.0), 0);
    }

    #[test]
    fn controlled_requests_and_waits_for_sync() {
        let mut input = RangeSliderInput { value: Some([10.0, 90.0]), ..RangeSliderInput::default() };
        let state = RangeSliderMachine::initial(&input);
        let step = send(&state, RangeSliderEvent::Set(0, 30.0, ChangeReason::Keyboard), &input);
        assert_eq!(step.state.value, [10.0, 90.0]);
        assert_eq!(step.commands, vec![RangeSliderCommand::ValueChange { value: [30.0, 90.0], reason: ChangeReason::Keyboard }]);
        input.value = Some([30.0, 90.0]);
        assert_eq!(RangeSliderMachine::sync(&step.state, &input).value, [30.0, 90.0]);
    }

    #[test]
    fn disabled_ignores_everything() {
        let input = RangeSliderInput { disabled: true, ..RangeSliderInput::default() };
        let state = RangeSliderMachine::initial(&input);
        let step = send(&state, RangeSliderEvent::PointerDown(50.0), &input);
        assert_eq!(step.state, state);
        assert!(step.commands.is_empty());
    }
}
