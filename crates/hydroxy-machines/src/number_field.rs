//! Number field: a typed draft over a committed number. Typing edits the draft
//! only; Enter or blur parses, clamps and commits it. Steps commit at once,
//! snapping to the `step` grid from `min`. Uncontrolled changes commit and
//! report; controlled ones only request, and the owner's value arrives via `sync`.

use hydroxy_statechart::{Machine, Transition};

/// Props for [`NumberFieldMachine`].
#[derive(Debug, Clone, PartialEq)]
pub struct NumberFieldInput {
    /// Controlled value. `Some` at start fixes controlled mode; later `None` clears it.
    pub value: Option<f64>,
    /// Uncontrolled initial value. `None` starts empty.
    pub default_value: Option<f64>,
    /// Lowest value.
    pub min: Option<f64>,
    /// Highest value.
    pub max: Option<f64>,
    /// Arrow-key and button step. Must be positive.
    pub step: f64,
    /// PageUp/PageDown step. Defaults to ten steps.
    pub large_step: Option<f64>,
    /// Fixed decimal places for display and rounding. `None` shows as many as the value needs.
    pub fraction_digits: Option<u8>,
    /// Ignore every change.
    pub disabled: bool,
    /// Keep focusable and selectable, but ignore changes.
    pub read_only: bool,
}

impl Default for NumberFieldInput {
    fn default() -> Self {
        Self {
            value: None,
            default_value: None,
            min: None,
            max: None,
            step: 1.0,
            large_step: None,
            fraction_digits: None,
            disabled: false,
            read_only: false,
        }
    }
}

impl NumberFieldInput {
    /// Clamp into `[min, max]`.
    #[must_use]
    pub fn clamp(&self, v: f64) -> f64 {
        let v = self.max.map_or(v, |max| v.min(max));
        self.min.map_or(v, |min| v.max(min))
    }

    /// Decimal places used for rounding: `fraction_digits`, else the step's own.
    fn digits(&self) -> u8 {
        self.fraction_digits.unwrap_or_else(|| decimals(self.step))
    }

    /// Round to [`Self::digits`] places, dropping float noise like `0.30000000000000004`.
    fn round(&self, v: f64) -> f64 {
        let scale = 10f64.powi(i32::from(self.digits()));
        (v * scale).round() / scale + 0.0
    }

    /// The display text for a value.
    #[must_use]
    pub fn format(&self, value: Option<f64>) -> String {
        match (value, self.fraction_digits) {
            (None, _) => String::new(),
            (Some(v), Some(d)) => format!("{:.*}", usize::from(d), v + 0.0),
            (Some(v), None) => (v + 0.0).to_string(),
        }
    }

    /// The value one step (or one large step) up or down from `base`, snapped
    /// to the step grid and clamped. Empty starts from 0 clamped.
    #[must_use]
    pub fn stepped(&self, base: Option<f64>, up: bool, large: bool) -> f64 {
        let step = if self.step > 0.0 { self.step } else { 1.0 };
        let Some(base) = base else { return self.round(self.clamp(0.0)) };
        let n = if large { self.large_step.map_or(10.0, |l| (l / step).round().max(1.0)) } else { 1.0 };
        let origin = self.min.unwrap_or(0.0);
        let k = (base - origin) / step;
        // Off-grid values snap to the next grid line in the step's direction.
        let k = if up { (k + 1e-9).floor() + n } else { (k - 1e-9).ceil() - n };
        self.round(self.clamp(origin + k * step))
    }
}

/// Decimal places in a step, such as 2 for `0.25`.
fn decimals(step: f64) -> u8 {
    let text = step.to_string();
    text.split_once('.').map_or(0, |(_, frac)| u8::try_from(frac.len()).unwrap_or(u8::MAX).min(12))
}

/// Parse typed text. Accepts a leading `-` or `−`, and ignores spaces, `,` and `_` grouping.
/// `Ok(None)` for empty text; `Err` for anything that isn't a finite number.
fn parse(text: &str) -> Result<Option<f64>, ()> {
    let cleaned: String =
        text.chars().filter(|c| !matches!(c, ' ' | ',' | '_')).map(|c| if c == '−' { '-' } else { c }).collect();
    if cleaned.is_empty() {
        return Ok(None);
    }
    match cleaned.parse::<f64>() {
        Ok(v) if v.is_finite() => Ok(Some(v)),
        _ => Err(()),
    }
}

/// What [`NumberFieldMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NumberFieldState {
    /// The committed value.
    pub value: Option<f64>,
    /// The text in the input, which may not be committed yet.
    pub draft: String,
    /// Fixed at start: the owner holds the value and the machine only requests changes.
    pub controlled: bool,
}

/// Inputs to [`NumberFieldMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum NumberFieldEvent {
    /// The user typed. Edits the draft only.
    SetText(String),
    /// Parse, clamp and commit the draft (Enter or blur). Unparseable text reverts.
    Commit,
    /// One step up or down; `large` for PageUp/PageDown.
    Step {
        /// Up (`true`) or down.
        up: bool,
        /// Use the large step.
        large: bool,
    },
    /// Jump to `min`, if set.
    ToMin,
    /// Jump to `max`, if set.
    ToMax,
}

/// Side effects from [`NumberFieldMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberFieldCommand {
    /// Tell the owner the value changed (or should, when controlled). `None` is empty.
    ValueChange(Option<f64>),
}

/// A number with a typed draft, steps and bounds.
#[derive(Debug, Clone, Copy)]
pub struct NumberFieldMachine;

impl NumberFieldMachine {
    /// Commit `value` (already clamped and rounded): adopt when uncontrolled, request either way.
    fn commit(
        state: &NumberFieldState,
        value: Option<f64>,
        input: &NumberFieldInput,
    ) -> Transition<NumberFieldState, NumberFieldCommand> {
        let mut next = state.clone();
        if !state.controlled {
            next.value = value;
        }
        next.draft = input.format(next.value);
        let commands = if value == state.value { Vec::new() } else { vec![NumberFieldCommand::ValueChange(value)] };
        Transition::update(state, next, commands)
    }
}

impl Machine for NumberFieldMachine {
    type Input = NumberFieldInput;
    type State = NumberFieldState;
    type Event = NumberFieldEvent;
    type Command = NumberFieldCommand;

    fn initial(input: &NumberFieldInput) -> NumberFieldState {
        let value = input.value.or(input.default_value).map(|v| input.round(input.clamp(v)));
        NumberFieldState { value, draft: input.format(value), controlled: input.value.is_some() }
    }

    fn transition(
        state: &NumberFieldState,
        event: NumberFieldEvent,
        input: &NumberFieldInput,
    ) -> Transition<NumberFieldState, NumberFieldCommand> {
        if input.disabled || input.read_only {
            return Transition::stay();
        }
        match event {
            NumberFieldEvent::SetText(text) => Transition::to(NumberFieldState { draft: text, ..state.clone() }),
            NumberFieldEvent::Commit => {
                let value = parse(&state.draft).unwrap_or(state.value).map(|v| input.round(input.clamp(v)));
                Self::commit(state, value, input)
            }
            NumberFieldEvent::Step { up, large } => {
                // Step from what the user sees, so "5" then ArrowUp gives 6 before a commit.
                let base = parse(&state.draft).unwrap_or(state.value);
                Self::commit(state, Some(input.stepped(base, up, large)), input)
            }
            NumberFieldEvent::ToMin => match input.min {
                Some(min) => Self::commit(state, Some(min), input),
                None => Transition::stay(),
            },
            NumberFieldEvent::ToMax => match input.max {
                Some(max) => Self::commit(state, Some(max), input),
                None => Transition::stay(),
            },
        }
    }

    fn sync(state: &NumberFieldState, input: &NumberFieldInput) -> Option<NumberFieldState> {
        // Only a changed controlled value rewrites the draft, so typing survives unrelated prop changes.
        if !state.controlled || input.value == state.value {
            return None;
        }
        let value = input.value.map(|v| input.round(input.clamp(v)));
        Some(NumberFieldState { value, draft: input.format(value), controlled: true })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step;

    const UP: NumberFieldEvent = NumberFieldEvent::Step { up: true, large: false };
    const DOWN: NumberFieldEvent = NumberFieldEvent::Step { up: false, large: false };

    #[test]
    fn typing_edits_draft_and_commit_clamps() {
        let input = NumberFieldInput { max: Some(10.0), ..NumberFieldInput::default() };
        let s = NumberFieldMachine::initial(&input);
        let s = step::<NumberFieldMachine>(&s, NumberFieldEvent::SetText("42".into()), &input);
        assert!(s.commands.is_empty());
        assert_eq!(s.state.value, None);
        let s = step::<NumberFieldMachine>(&s.state, NumberFieldEvent::Commit, &input);
        assert_eq!(s.state.value, Some(10.0));
        assert_eq!(s.state.draft, "10");
        assert_eq!(s.commands, vec![NumberFieldCommand::ValueChange(Some(10.0))]);
    }

    #[test]
    fn garbage_reverts_and_empty_clears() {
        let input = NumberFieldInput { default_value: Some(3.0), ..NumberFieldInput::default() };
        let s = NumberFieldMachine::initial(&input);
        let bad = step::<NumberFieldMachine>(
            &step::<NumberFieldMachine>(&s, NumberFieldEvent::SetText("abc".into()), &input).state,
            NumberFieldEvent::Commit,
            &input,
        );
        assert_eq!(bad.state, s);
        assert!(bad.commands.is_empty());
        let empty = step::<NumberFieldMachine>(
            &step::<NumberFieldMachine>(&s, NumberFieldEvent::SetText(String::new()), &input).state,
            NumberFieldEvent::Commit,
            &input,
        );
        assert_eq!(empty.state.value, None);
        assert_eq!(empty.commands, vec![NumberFieldCommand::ValueChange(None)]);
    }

    #[test]
    fn steps_snap_to_grid_and_stop_at_bounds() {
        let input = NumberFieldInput { min: Some(0.0), max: Some(1.0), step: 0.1, ..NumberFieldInput::default() };
        let s = NumberFieldState { value: Some(0.2), draft: "0.2".into(), controlled: false };
        assert_eq!(step::<NumberFieldMachine>(&s, UP, &input).state.value, Some(0.3));
        let off = NumberFieldState { value: Some(0.25), draft: "0.25".into(), controlled: false };
        assert_eq!(step::<NumberFieldMachine>(&off, UP, &input).state.value, Some(0.3));
        assert_eq!(step::<NumberFieldMachine>(&off, DOWN, &input).state.value, Some(0.2));
        let top = NumberFieldState { value: Some(1.0), draft: "1".into(), controlled: false };
        assert!(step::<NumberFieldMachine>(&top, UP, &input).commands.is_empty());
        assert_eq!(
            step::<NumberFieldMachine>(&top, NumberFieldEvent::Step { up: false, large: true }, &input).state.value,
            Some(0.0)
        );
    }

    #[test]
    fn step_uses_the_uncommitted_draft_and_empty_starts_at_zero() {
        let input = NumberFieldInput::default();
        let s = NumberFieldState { value: Some(1.0), draft: "5".into(), controlled: false };
        assert_eq!(step::<NumberFieldMachine>(&s, UP, &input).state.value, Some(6.0));
        let empty = NumberFieldMachine::initial(&input);
        assert_eq!(step::<NumberFieldMachine>(&empty, UP, &input).state.value, Some(0.0));
    }

    #[test]
    fn home_end_need_bounds() {
        let input = NumberFieldInput { min: Some(-5.0), ..NumberFieldInput::default() };
        let s = NumberFieldMachine::initial(&input);
        assert_eq!(step::<NumberFieldMachine>(&s, NumberFieldEvent::ToMin, &input).state.value, Some(-5.0));
        assert!(step::<NumberFieldMachine>(&s, NumberFieldEvent::ToMax, &input).commands.is_empty());
    }

    #[test]
    fn fraction_digits_format_and_round() {
        let input =
            NumberFieldInput { fraction_digits: Some(2), default_value: Some(1.005), ..NumberFieldInput::default() };
        let s = NumberFieldMachine::initial(&input);
        assert_eq!(s.draft, "1.00");
        let s = step::<NumberFieldMachine>(
            &step::<NumberFieldMachine>(&s, NumberFieldEvent::SetText("1,234.567".into()), &input).state,
            NumberFieldEvent::Commit,
            &input,
        );
        assert_eq!(s.state.value, Some(1234.57));
        assert_eq!(s.state.draft, "1234.57");
    }

    #[test]
    fn controlled_requests_then_syncs() {
        let mut input = NumberFieldInput { value: Some(1.0), ..NumberFieldInput::default() };
        let s = NumberFieldMachine::initial(&input);
        assert!(s.controlled);
        let step = hydroxy_statechart::step::<NumberFieldMachine>(&s, UP, &input);
        assert_eq!(step.state.value, Some(1.0));
        assert_eq!(step.commands, vec![NumberFieldCommand::ValueChange(Some(2.0))]);
        input.value = Some(2.0);
        let synced = hydroxy_statechart::sync_or_keep::<NumberFieldMachine>(&step.state, &input);
        assert_eq!((synced.value, synced.draft.as_str()), (Some(2.0), "2"));
        input.value = None;
        assert_eq!(hydroxy_statechart::sync_or_keep::<NumberFieldMachine>(&synced, &input).value, None);
    }

    #[test]
    fn sync_keeps_an_in_progress_draft() {
        let input = NumberFieldInput { value: Some(1.0), ..NumberFieldInput::default() };
        let s = NumberFieldState { value: Some(1.0), draft: "1.".into(), controlled: true };
        assert_eq!(hydroxy_statechart::sync_or_keep::<NumberFieldMachine>(&s, &input), s);
    }

    #[test]
    fn disabled_and_read_only_ignore_everything() {
        for input in [
            NumberFieldInput { disabled: true, ..NumberFieldInput::default() },
            NumberFieldInput { read_only: true, ..NumberFieldInput::default() },
        ] {
            let s = NumberFieldMachine::initial(&input);
            assert_eq!(step::<NumberFieldMachine>(&s, UP, &input).state, s);
        }
    }

    #[test]
    fn parse_accepts_signs_and_grouping_only() {
        assert_eq!(parse("−1 000"), Ok(Some(-1000.0)));
        assert_eq!(parse("  "), Ok(None));
        assert!(parse("inf").is_err());
        assert!(parse("1.2.3").is_err());
    }
}
