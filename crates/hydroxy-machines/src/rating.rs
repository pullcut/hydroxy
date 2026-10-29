//! Rating owns a star value from `0` (unrated) to `count`, in whole or half
//! steps, plus a hover preview. Uncontrolled picks commit and report;
//! controlled picks only request, and the owner's value arrives via `sync`.

use hydroxy_statechart::{Controllable, Machine, Transition};

pub use crate::ChangeReason;

/// Props for [`RatingMachine`].
#[allow(clippy::struct_excessive_bools, reason = "independent props, as on the component")]
#[derive(Debug, Clone, PartialEq)]
pub struct RatingInput {
    /// Number of stars.
    pub count: usize,
    /// Controlled value. `Some` at start fixes controlled mode.
    pub value: Option<f64>,
    /// Uncontrolled initial value.
    pub default_value: f64,
    /// Allow half stars.
    pub allow_half: bool,
    /// Picking the current value again, or stepping below the first star, clears to 0.
    pub clearable: bool,
    /// Show the value only. No picks, no hover.
    pub read_only: bool,
    /// Ignore picks and drop out of the tab order.
    pub disabled: bool,
}

impl Default for RatingInput {
    fn default() -> Self {
        Self {
            count: 5,
            value: None,
            default_value: 0.0,
            allow_half: false,
            clearable: false,
            read_only: false,
            disabled: false,
        }
    }
}

impl RatingInput {
    /// One keyboard step: `0.5` with half stars, else `1`.
    #[must_use]
    pub fn step(&self) -> f64 {
        if self.allow_half { 0.5 } else { 1.0 }
    }

    /// Round to the step and clamp to `0..=count`.
    #[must_use]
    pub fn normalize(&self, v: f64) -> f64 {
        let step = self.step();
        ((v / step).round() * step).clamp(0.0, count_f64(self.count))
    }

    fn interactive(&self) -> bool {
        !self.read_only && !self.disabled
    }

    /// The lowest value the keyboard can reach.
    fn floor(&self) -> f64 {
        if self.clearable { 0.0 } else { self.step() }
    }
}

#[allow(clippy::cast_precision_loss, reason = "star counts are tiny")]
fn count_f64(count: usize) -> f64 {
    count as f64
}

/// The 1-based star that holds `value`: 2.5 lives in star 3. At least 1.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "value is clamped to 0..=count")]
pub fn star_of(value: f64) -> usize {
    (value.ceil() as usize).max(1)
}

/// What [`RatingMachine`] remembers between events.
#[derive(Debug, Clone, PartialEq)]
pub struct RatingState {
    /// The committed value. Controlled when the owner holds it: the machine then only
    /// requests changes.
    pub value: Controllable<f64>,
    /// The value under the pointer, shown instead of `value` while set.
    pub hover: Option<f64>,
}

impl RatingState {
    /// What the stars should show: the hover preview, else the value.
    #[must_use]
    pub fn display(&self) -> f64 {
        self.hover.unwrap_or(self.value.value)
    }
}

/// A keyboard action on the rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RatingKey {
    /// Up one step.
    Increment,
    /// Down one step.
    Decrement,
    /// The lowest value.
    First,
    /// The highest value.
    Last,
}

/// Inputs to [`RatingMachine`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RatingEvent {
    /// Pick a value: a star (or half star) was clicked.
    Select(f64, ChangeReason),
    /// The pointer is over a value, or left (`None`).
    Hover(Option<f64>),
    /// A key on a focused star.
    Key(RatingKey),
}

/// Side effects from [`RatingMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RatingCommand {
    /// Tell the owner the value changed (or should, when controlled).
    ValueChange {
        /// The new (or proposed) value.
        value: f64,
        /// What caused the change.
        reason: ChangeReason,
    },
    /// Move focus to this 1-based star.
    FocusStar(usize),
}

/// A star rating.
#[derive(Debug, Clone, Copy)]
pub struct RatingMachine;

impl Machine for RatingMachine {
    type Input = RatingInput;
    type State = RatingState;
    type Event = RatingEvent;
    type Command = RatingCommand;

    fn initial(input: &RatingInput) -> RatingState {
        RatingState {
            value: Controllable::new(
                input.normalize(input.value.unwrap_or(input.default_value)),
                input.value.is_some(),
            ),
            hover: None,
        }
    }

    #[allow(clippy::float_cmp, reason = "exact change detection on normalized values")]
    fn transition(
        state: &RatingState,
        event: RatingEvent,
        input: &RatingInput,
    ) -> Transition<RatingState, RatingCommand> {
        if !input.interactive() {
            return Transition::to(RatingState { hover: None, ..state.clone() });
        }
        let (value, reason, focus) = match event {
            RatingEvent::Hover(v) => {
                return Transition::to(RatingState { hover: v.map(|v| input.normalize(v)), ..state.clone() });
            }
            RatingEvent::Select(v, reason) => {
                let v = input.normalize(v);
                let v = if input.clearable && v == state.value.value { 0.0 } else { v };
                (v, reason, false)
            }
            RatingEvent::Key(key) => {
                let v = match key {
                    RatingKey::Increment => state.value.value + input.step(),
                    RatingKey::Decrement => state.value.value - input.step(),
                    RatingKey::First => input.floor(),
                    RatingKey::Last => count_f64(input.count),
                };
                (input.normalize(v).max(input.floor()), ChangeReason::Keyboard, true)
            }
        };
        let mut next = state.clone();
        let mut commands = Vec::new();
        if focus {
            // Keyboard use clears a stale pointer preview.
            next.hover = None;
            commands.push(RatingCommand::FocusStar(star_of(value)));
        }
        if value != state.value.value {
            next.value.set(value);
            commands.push(RatingCommand::ValueChange { value, reason });
        }
        Transition::update(state, next, commands)
    }

    fn sync(state: &RatingState, input: &RatingInput) -> Option<RatingState> {
        let mut next = state.clone();
        next.value.sync(input.value.map(|v| input.normalize(v)));
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_commits_and_reports() {
        let input = RatingInput::default();
        let step = hydroxy_statechart::step::<RatingMachine>(
            &RatingMachine::initial(&input),
            RatingEvent::Select(3.0, ChangeReason::Pointer),
            &input,
        );
        assert!((step.state.value.value - 3.0).abs() < f64::EPSILON);
        assert_eq!(step.commands, vec![RatingCommand::ValueChange { value: 3.0, reason: ChangeReason::Pointer }]);
    }

    #[test]
    fn half_steps_round_and_whole_steps_round_up() {
        let half = RatingInput { allow_half: true, ..RatingInput::default() };
        assert!((half.normalize(2.4) - 2.5).abs() < f64::EPSILON);
        assert!((RatingInput::default().normalize(2.5) - 3.0).abs() < f64::EPSILON);
        assert!((half.normalize(9.0) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn clearable_reselect_clears() {
        let input = RatingInput { clearable: true, default_value: 3.0, ..RatingInput::default() };
        let step = hydroxy_statechart::step::<RatingMachine>(
            &RatingMachine::initial(&input),
            RatingEvent::Select(3.0, ChangeReason::Pointer),
            &input,
        );
        assert!(step.state.value.value.abs() < f64::EPSILON);
        let input = RatingInput { default_value: 3.0, ..RatingInput::default() };
        let step = hydroxy_statechart::step::<RatingMachine>(
            &RatingMachine::initial(&input),
            RatingEvent::Select(3.0, ChangeReason::Pointer),
            &input,
        );
        assert!(step.commands.is_empty());
    }

    #[test]
    fn keys_step_and_focus_the_holding_star() {
        let input = RatingInput { allow_half: true, default_value: 2.0, ..RatingInput::default() };
        let step = hydroxy_statechart::step::<RatingMachine>(
            &RatingMachine::initial(&input),
            RatingEvent::Key(RatingKey::Increment),
            &input,
        );
        assert!((step.state.value.value - 2.5).abs() < f64::EPSILON);
        assert_eq!(step.commands[0], RatingCommand::FocusStar(3));
        let step = hydroxy_statechart::step::<RatingMachine>(&step.state, RatingEvent::Key(RatingKey::Last), &input);
        assert!((step.state.value.value - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn keyboard_floor_depends_on_clearable() {
        let input = RatingInput { default_value: 1.0, ..RatingInput::default() };
        let step = hydroxy_statechart::step::<RatingMachine>(
            &RatingMachine::initial(&input),
            RatingEvent::Key(RatingKey::Decrement),
            &input,
        );
        assert!((step.state.value.value - 1.0).abs() < f64::EPSILON);
        let input = RatingInput { clearable: true, ..input };
        let step = hydroxy_statechart::step::<RatingMachine>(
            &RatingMachine::initial(&input),
            RatingEvent::Key(RatingKey::First),
            &input,
        );
        assert!(step.state.value.value.abs() < f64::EPSILON);
        assert_eq!(step.commands[0], RatingCommand::FocusStar(1));
    }

    #[test]
    fn hover_previews_without_committing() {
        let input = RatingInput::default();
        let step = hydroxy_statechart::step::<RatingMachine>(
            &RatingMachine::initial(&input),
            RatingEvent::Hover(Some(4.0)),
            &input,
        );
        assert!((step.state.display() - 4.0).abs() < f64::EPSILON);
        assert!(step.state.value.value.abs() < f64::EPSILON);
        assert!(step.commands.is_empty());
        assert_eq!(
            hydroxy_statechart::step::<RatingMachine>(&step.state, RatingEvent::Hover(None), &input).state.hover,
            None
        );
    }

    #[test]
    fn read_only_and_disabled_ignore_input() {
        for input in [
            RatingInput { read_only: true, ..RatingInput::default() },
            RatingInput { disabled: true, ..RatingInput::default() },
        ] {
            let state = RatingMachine::initial(&input);
            let step = hydroxy_statechart::step::<RatingMachine>(
                &state,
                RatingEvent::Select(2.0, ChangeReason::Pointer),
                &input,
            );
            assert_eq!(step.state, state);
            assert!(step.commands.is_empty());
            assert_eq!(
                hydroxy_statechart::step::<RatingMachine>(&state, RatingEvent::Hover(Some(2.0)), &input).state.hover,
                None
            );
        }
    }

    #[test]
    fn controlled_requests_and_waits_for_sync() {
        let mut input = RatingInput { value: Some(1.0), ..RatingInput::default() };
        let step = hydroxy_statechart::step::<RatingMachine>(
            &RatingMachine::initial(&input),
            RatingEvent::Select(4.0, ChangeReason::Pointer),
            &input,
        );
        assert!((step.state.value.value - 1.0).abs() < f64::EPSILON);
        input.value = Some(4.0);
        assert!(
            (hydroxy_statechart::sync_or_keep::<RatingMachine>(&step.state, &input).value.value - 4.0).abs()
                < f64::EPSILON
        );
    }

    #[test]
    fn star_of_rounds_up() {
        assert_eq!(star_of(0.0), 1);
        assert_eq!(star_of(2.5), 3);
        assert_eq!(star_of(3.0), 3);
    }
}
