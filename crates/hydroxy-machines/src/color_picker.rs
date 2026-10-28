//! Colour picker: a saturation/value area, a hue slider, an optional alpha
//! slider and a hex field over one colour. The state keeps HSV so the hue
//! survives passing through grey or black; the value goes out as [`Rgba`].

#![allow(clippy::float_cmp, clippy::many_single_char_names, reason = "saturation and value are compared exactly against 0.0, where hue is undefined")]

use hydroxy_statechart::{Machine, Transition};

use crate::color::{Hsv, Rgba};

/// Props for [`ColorPickerMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ColorPickerInput {
    /// Controlled value. `Some` at start fixes controlled mode.
    pub value: Option<Rgba>,
    /// Uncontrolled initial value. Default opaque black.
    pub default_value: Option<Rgba>,
    /// Offer an alpha slider and `#rrggbbaa` hex.
    pub alpha: bool,
    /// Ignore every change.
    pub disabled: bool,
}

/// What [`ColorPickerMachine`] remembers between events.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorPickerState {
    /// Hue, saturation and value of the colour.
    pub hsv: Hsv,
    /// Opacity, 0–1.
    pub alpha: f64,
    /// The hex field's text, which may be half typed.
    pub draft: String,
    /// Fixed at start: the owner holds the value.
    pub controlled: bool,
}

impl ColorPickerState {
    /// The colour with its alpha (always opaque without the alpha slider).
    #[must_use]
    pub fn rgba(&self, input: &ColorPickerInput) -> Rgba {
        let a = if input.alpha { (self.alpha * 255.0).round().clamp(0.0, 255.0) } else { 255.0 };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "clamped to 0..=255")]
        Rgba { a: a as u8, ..self.hsv.to_rgb() }
    }

    /// Whether the hex text isn't a colour.
    #[must_use]
    pub fn invalid(&self) -> bool {
        Rgba::parse_hex(&self.draft).is_none()
    }

    fn with(hsv: Hsv, alpha: f64, input: &ColorPickerInput, controlled: bool) -> Self {
        let mut state = Self { hsv, alpha, draft: String::new(), controlled };
        state.draft = state.rgba(input).hex(input.alpha);
        state
    }

    /// Take on `color`, keeping the hue (and saturation) it can't express.
    fn adopt(&mut self, color: Rgba) {
        let hsv = color.to_hsv();
        self.hsv = Hsv {
            h: if hsv.s == 0.0 || hsv.v == 0.0 { self.hsv.h } else { hsv.h },
            s: if hsv.v == 0.0 { self.hsv.s } else { hsv.s },
            v: hsv.v,
        };
        self.alpha = f64::from(color.a) / 255.0;
    }
}

/// Inputs to [`ColorPickerMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum ColorPickerEvent {
    /// Pointer on the area: saturation and value, each 0–1.
    SetArea {
        /// Saturation, left to right.
        s: f64,
        /// Value, bottom to top.
        v: f64,
    },
    /// Arrow keys on the area, in percent points (`dv` up is positive).
    NudgeArea {
        /// Change in saturation.
        ds: f64,
        /// Change in value.
        dv: f64,
    },
    /// Set the hue in degrees.
    SetHue(f64),
    /// Change the hue by degrees.
    NudgeHue(f64),
    /// Set the opacity, 0–1.
    SetAlpha(f64),
    /// Change the opacity by percent points.
    NudgeAlpha(f64),
    /// The hex field's text changed. A valid colour applies at once.
    HexInput(String),
    /// Enter or blur in the hex field: tidy the text.
    HexCommit,
    /// Pick a whole colour, such as a swatch.
    SetColor(Rgba),
}

/// Side effects from [`ColorPickerMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPickerCommand {
    /// The colour changed (or should, when controlled).
    ValueChange(Rgba),
}

/// One colour with its hex text.
#[derive(Debug, Clone, Copy)]
pub struct ColorPickerMachine;

impl Machine for ColorPickerMachine {
    type Input = ColorPickerInput;
    type State = ColorPickerState;
    type Event = ColorPickerEvent;
    type Command = ColorPickerCommand;

    fn initial(input: &ColorPickerInput) -> ColorPickerState {
        let color = input.value.or(input.default_value).unwrap_or(Rgba::rgb(0, 0, 0));
        ColorPickerState::with(color.to_hsv(), f64::from(color.a) / 255.0, input, input.value.is_some())
    }

    fn transition(
        state: &ColorPickerState,
        event: ColorPickerEvent,
        input: &ColorPickerInput,
    ) -> Transition<ColorPickerState, ColorPickerCommand> {
        if input.disabled {
            return Transition::stay();
        }
        let mut next = state.clone();
        match event {
            ColorPickerEvent::SetArea { s, v } => {
                next.hsv.s = s.clamp(0.0, 1.0);
                next.hsv.v = v.clamp(0.0, 1.0);
            }
            ColorPickerEvent::NudgeArea { ds, dv } => {
                next.hsv.s = (state.hsv.s + ds / 100.0).clamp(0.0, 1.0);
                next.hsv.v = (state.hsv.v + dv / 100.0).clamp(0.0, 1.0);
            }
            ColorPickerEvent::SetHue(h) => next.hsv.h = h.clamp(0.0, 360.0),
            ColorPickerEvent::NudgeHue(dh) => next.hsv.h = (state.hsv.h + dh).clamp(0.0, 360.0),
            ColorPickerEvent::SetAlpha(a) => next.alpha = a.clamp(0.0, 1.0),
            ColorPickerEvent::NudgeAlpha(da) => next.alpha = (state.alpha + da / 100.0).clamp(0.0, 1.0),
            ColorPickerEvent::HexInput(text) => {
                if let Some(color) = Rgba::parse_hex(&text) {
                    // Typing the colour already shown must not disturb the hue.
                    if color != state.rgba(input) {
                        next.adopt(color);
                    }
                }
                next.draft = text;
                return Self::finish(state, next, input, false);
            }
            ColorPickerEvent::HexCommit => {}
            ColorPickerEvent::SetColor(color) => next.adopt(color),
        }
        Self::finish(state, next, input, true)
    }

    fn sync(state: &ColorPickerState, input: &ColorPickerInput) -> Option<ColorPickerState> {
        Some(match input.value {
            Some(color) if state.controlled && color != state.rgba(input) => {
                let mut next = state.clone();
                next.adopt(color);
                next.draft = next.rgba(input).hex(input.alpha);
                next
            }
            _ => state.clone(),
        })
    }
}

impl ColorPickerMachine {
    /// Report a changed colour; `retext` rewrites the hex text to match.
    fn finish(
        state: &ColorPickerState,
        mut next: ColorPickerState,
        input: &ColorPickerInput,
        retext: bool,
    ) -> Transition<ColorPickerState, ColorPickerCommand> {
        let (before, after) = (state.rgba(input), next.rgba(input));
        if retext {
            next.draft = after.hex(input.alpha);
        }
        let commands = if before == after { Vec::new() } else { vec![ColorPickerCommand::ValueChange(after)] };
        Transition::with(next, commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(input: &ColorPickerInput, events: Vec<ColorPickerEvent>) -> (ColorPickerState, Vec<ColorPickerCommand>) {
        let mut state = ColorPickerMachine::initial(input);
        let mut all = Vec::new();
        for e in events {
            let t = ColorPickerMachine::transition(&state, e, input).resolve(&state);
            state = t.state;
            all.extend(t.commands);
        }
        (state, all)
    }

    #[test]
    fn area_and_hue_drive_the_colour() {
        let input = ColorPickerInput::default();
        let (state, commands) = run(&input, vec![ColorPickerEvent::SetHue(0.0), ColorPickerEvent::SetArea { s: 1.0, v: 1.0 }]);
        assert_eq!(state.rgba(&input), Rgba::rgb(255, 0, 0));
        assert_eq!(state.draft, "#ff0000");
        assert_eq!(commands, [ColorPickerCommand::ValueChange(Rgba::rgb(255, 0, 0))]);
        let (state, _) = run(&input, vec![ColorPickerEvent::SetHue(240.0), ColorPickerEvent::SetArea { s: 1.0, v: 1.0 }]);
        assert_eq!(state.draft, "#0000ff");
        // Pointer overshoot clamps.
        let (state, _) = run(&input, vec![ColorPickerEvent::SetArea { s: 3.0, v: -1.0 }]);
        assert_eq!((state.hsv.s, state.hsv.v), (1.0, 0.0));
    }

    #[test]
    fn nudges_move_by_percent_points_and_clamp() {
        let input = ColorPickerInput { alpha: true, default_value: Some(Rgba::rgb(255, 0, 0)), ..Default::default() };
        let (state, _) = run(&input, vec![ColorPickerEvent::NudgeArea { ds: -10.0, dv: 0.0 }, ColorPickerEvent::NudgeHue(-1.0)]);
        assert!((state.hsv.s - 0.9).abs() < 1e-9);
        assert_eq!(state.hsv.h, 0.0);
        let (state, _) = run(&input, vec![ColorPickerEvent::NudgeAlpha(-1.0)]);
        assert_eq!(state.rgba(&input).a, 252);
        assert_eq!(state.draft, "#ff0000fc");
    }

    #[test]
    fn hex_applies_while_typing_and_reverts_on_commit() {
        let input = ColorPickerInput::default();
        let (state, commands) = run(&input, vec![ColorPickerEvent::HexInput("#12".into())]);
        assert!(state.invalid() && commands.is_empty());
        let (state, commands) = run(&input, vec![ColorPickerEvent::HexInput("#336699".into())]);
        assert_eq!(commands, [ColorPickerCommand::ValueChange(Rgba::rgb(51, 102, 153))]);
        assert_eq!(state.draft, "#336699");
        let (state, _) = run(&input, vec![ColorPickerEvent::HexInput("#336699".into()), ColorPickerEvent::HexInput("zz".into()), ColorPickerEvent::HexCommit]);
        assert_eq!(state.draft, "#336699");
        // Shorthand is kept as typed until commit, then expanded.
        let (state, _) = run(&input, vec![ColorPickerEvent::HexInput("#f80".into())]);
        assert_eq!(state.draft, "#f80");
        let (state, _) = run(&input, vec![ColorPickerEvent::HexInput("#f80".into()), ColorPickerEvent::HexCommit]);
        assert_eq!(state.draft, "#ff8800");
    }

    #[test]
    fn hue_survives_grey_and_black() {
        let input = ColorPickerInput::default();
        let (state, _) = run(&input, vec![ColorPickerEvent::SetHue(200.0), ColorPickerEvent::SetColor(Rgba::rgb(128, 128, 128))]);
        assert_eq!(state.hsv.h, 200.0);
        let (state, _) = run(&input, vec![ColorPickerEvent::SetHue(200.0), ColorPickerEvent::SetArea { s: 0.0, v: 0.0 }, ColorPickerEvent::HexInput("#000000".into())]);
        assert_eq!(state.hsv.h, 200.0);
    }

    #[test]
    fn swatch_sets_the_colour_and_controlled_follows_owner() {
        let input = ColorPickerInput { value: Some(Rgba::rgb(255, 0, 0)), ..Default::default() };
        let state = ColorPickerMachine::initial(&input);
        assert!(state.controlled);
        let moved = ColorPickerInput { value: Some(Rgba::rgb(0, 255, 0)), ..input.clone() };
        let synced = hydroxy_statechart::sync_or_keep::<ColorPickerMachine>(&state, &moved);
        assert_eq!(synced.rgba(&moved), Rgba::rgb(0, 255, 0));
        assert_eq!(synced.draft, "#00ff00");
        let t = ColorPickerMachine::transition(&state, ColorPickerEvent::SetColor(Rgba::rgb(0, 0, 255)), &input).resolve(&state);
        assert_eq!(t.commands, [ColorPickerCommand::ValueChange(Rgba::rgb(0, 0, 255))]);
    }

    #[test]
    fn disabled_ignores_events() {
        let input = ColorPickerInput { disabled: true, ..Default::default() };
        let (state, commands) = run(&input, vec![ColorPickerEvent::SetHue(90.0)]);
        assert_eq!(state.hsv.h, 0.0);
        assert!(commands.is_empty());
    }
}
