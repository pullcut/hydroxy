//! Colour picker connector. Anatomy: root, area (with its thumb), hue and alpha
//! sliders (each a track with a thumb), preview, hex input, swatches, swatch,
//! hidden input. Every part carries `data-scope="color-picker"` and its `data-part`.
//!
//! The area and sliders are `role="slider"` thumbs on tracks the adapter turns
//! pointer positions into events for; arrows work on the focused thumb. Colours
//! reach the CSS as custom properties on the root (`--picker-hue`,
//! `--picker-color`), so gradients need no JS.

pub use hydroxy_machines::color::{Hsv, Rgba};
pub use hydroxy_machines::color_picker::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "color-picker";

/// A neutral starter palette for the swatches.
pub const DEFAULT_SWATCHES: [Rgba; 8] = [
    Rgba::rgb(0xef, 0x44, 0x44),
    Rgba::rgb(0xf9, 0x73, 0x16),
    Rgba::rgb(0xea, 0xb3, 0x08),
    Rgba::rgb(0x22, 0xc5, 0x5e),
    Rgba::rgb(0x06, 0xb6, 0xd4),
    Rgba::rgb(0x3b, 0x82, 0xf6),
    Rgba::rgb(0x8b, 0x5c, 0xf6),
    Rgba::rgb(0xec, 0x48, 0x99),
];

/// Ids for the parts the adapter needs to find.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The hex input.
    pub hex: String,
    /// The area thumb.
    pub area: String,
    /// The hue thumb.
    pub hue: String,
    /// The alpha thumb.
    pub alpha: String,
}

/// Ids from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids {
        hex: format!("{root}-hex"),
        area: format!("{root}-area"),
        hue: format!("{root}-hue"),
        alpha: format!("{root}-alpha"),
    }
}

/// `style` for the root: the hue and current colour as custom properties.
#[must_use]
pub fn root_style(state: &ColorPickerState, input: &ColorPickerInput) -> String {
    let c = state.rgba(input);
    let hue = Hsv { s: 1.0, v: 1.0, ..state.hsv }.to_rgb();
    format!(
        "--picker-hue: rgb({} {} {}); --picker-color: rgb({} {} {}); --picker-alpha: {};",
        hue.r,
        hue.g,
        hue.b,
        c.r,
        c.g,
        c.b,
        (state.alpha * 100.0).round() / 100.0
    )
}

/// `style` for the area thumb: saturation across, value up.
#[must_use]
pub fn area_thumb_style(state: &ColorPickerState) -> String {
    format!("left: {:.2}%; top: {:.2}%;", state.hsv.s * 100.0, (1.0 - state.hsv.v) * 100.0)
}

/// `style` for the hue thumb.
#[must_use]
pub fn hue_thumb_style(state: &ColorPickerState) -> String {
    format!("left: {:.2}%;", state.hsv.h / 360.0 * 100.0)
}

/// `style` for the alpha thumb.
#[must_use]
pub fn alpha_thumb_style(state: &ColorPickerState) -> String {
    format!("left: {:.2}%;", state.alpha * 100.0)
}

/// `aria-valuetext` for the area thumb.
#[must_use]
pub fn area_valuetext(state: &ColorPickerState) -> String {
    format!("Saturation {}%, brightness {}%", percent(state.hsv.s), percent(state.hsv.v))
}

/// A 0–1 fraction as a whole percentage.
#[must_use]
pub fn percent(fraction: f64) -> u32 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "clamped to 0..=100")]
    let p = (fraction.clamp(0.0, 1.0) * 100.0).round() as u32;
    p
}

/// `aria-valuenow` for the hue thumb: whole degrees.
#[must_use]
pub fn hue_degrees(state: &ColorPickerState) -> u32 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "clamped to 0..=360")]
    let h = state.hsv.h.clamp(0.0, 360.0).round() as u32;
    h
}

/// A pointer position as a 0–1 fraction along a track starting at `origin` and `length` long.
#[must_use]
pub fn fraction(pointer: f64, origin: f64, length: f64) -> f64 {
    if length <= 0.0 { 0.0 } else { ((pointer - origin) / length).clamp(0.0, 1.0) }
}

/// A pointer position on the area as an event: saturation across, value up.
#[must_use]
pub fn area_event(x: f64, y: f64, left: f64, top: f64, width: f64, height: f64) -> ColorPickerEvent {
    ColorPickerEvent::SetArea { s: fraction(x, left, width), v: 1.0 - fraction(y, top, height) }
}

/// A pointer position on the hue slider as an event.
#[must_use]
pub fn hue_event(x: f64, left: f64, width: f64) -> ColorPickerEvent {
    ColorPickerEvent::SetHue(fraction(x, left, width) * 360.0)
}

/// A pointer position on the alpha slider as an event.
#[must_use]
pub fn alpha_event(x: f64, left: f64, width: f64) -> ColorPickerEvent {
    ColorPickerEvent::SetAlpha(fraction(x, left, width))
}

fn amount(shift: bool, page: bool) -> f64 {
    if page || shift { 10.0 } else { 1.0 }
}

/// Keys on the area thumb: arrows move one point (ten with Shift), PageUp/PageDown change brightness by ten.
#[must_use]
pub fn area_keydown(key: &str, shift: bool) -> Option<ColorPickerEvent> {
    let n = amount(shift, false);
    let (ds, dv) = match key {
        "ArrowLeft" => (-n, 0.0),
        "ArrowRight" => (n, 0.0),
        "ArrowUp" => (0.0, n),
        "ArrowDown" => (0.0, -n),
        "PageUp" => (0.0, 10.0),
        "PageDown" => (0.0, -10.0),
        _ => return None,
    };
    Some(ColorPickerEvent::NudgeArea { ds, dv })
}

/// Keys on the hue thumb: arrows one degree (ten with Shift or Page keys), Home and End the ends.
#[must_use]
pub fn hue_keydown(key: &str, shift: bool) -> Option<ColorPickerEvent> {
    match key {
        "ArrowRight" | "ArrowUp" => Some(ColorPickerEvent::NudgeHue(amount(shift, false))),
        "ArrowLeft" | "ArrowDown" => Some(ColorPickerEvent::NudgeHue(-amount(shift, false))),
        "PageUp" => Some(ColorPickerEvent::NudgeHue(10.0)),
        "PageDown" => Some(ColorPickerEvent::NudgeHue(-10.0)),
        "Home" => Some(ColorPickerEvent::SetHue(0.0)),
        "End" => Some(ColorPickerEvent::SetHue(360.0)),
        _ => None,
    }
}

/// Keys on the alpha thumb: arrows one point (ten with Shift or Page keys), Home and End the ends.
#[must_use]
pub fn alpha_keydown(key: &str, shift: bool) -> Option<ColorPickerEvent> {
    match key {
        "ArrowRight" | "ArrowUp" => Some(ColorPickerEvent::NudgeAlpha(amount(shift, false))),
        "ArrowLeft" | "ArrowDown" => Some(ColorPickerEvent::NudgeAlpha(-amount(shift, false))),
        "PageUp" => Some(ColorPickerEvent::NudgeAlpha(10.0)),
        "PageDown" => Some(ColorPickerEvent::NudgeAlpha(-10.0)),
        "Home" => Some(ColorPickerEvent::SetAlpha(0.0)),
        "End" => Some(ColorPickerEvent::SetAlpha(1.0)),
        _ => None,
    }
}

/// Keys in the hex field. Enter tidies the text without submitting a form.
#[must_use]
pub fn hex_keydown(key: &str) -> Option<ColorPickerEvent> {
    (key == "Enter").then_some(ColorPickerEvent::HexCommit)
}

/// The hidden input's value: the hex the field shows.
#[must_use]
pub fn form_value(state: &ColorPickerState, input: &ColorPickerInput) -> String {
    state.rgba(input).hex(input.alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_follow_the_state() {
        let input = ColorPickerInput::default();
        let state = ColorPickerState {
            hsv: Rgba::rgb(255, 0, 0).to_hsv(),
            alpha: 1.0,
            draft: "#ff0000".into(),
            controlled: false,
        };
        let style = root_style(&state, &input);
        assert!(style.contains("--picker-hue: rgb(255 0 0)") && style.contains("--picker-color: rgb(255 0 0)"));
        assert_eq!(area_thumb_style(&state), "left: 100.00%; top: 0.00%;");
        assert_eq!(hue_thumb_style(&state), "left: 0.00%;");
        assert_eq!(area_valuetext(&state), "Saturation 100%, brightness 100%");
        assert_eq!(form_value(&state, &input), "#ff0000");
    }

    #[test]
    fn pointer_positions_become_events() {
        // Top-right corner is full saturation and value; bottom-left is neither.
        assert_eq!(area_event(200.0, 0.0, 0.0, 0.0, 200.0, 100.0), ColorPickerEvent::SetArea { s: 1.0, v: 1.0 });
        assert_eq!(area_event(-5.0, 150.0, 0.0, 0.0, 200.0, 100.0), ColorPickerEvent::SetArea { s: 0.0, v: 0.0 });
        assert_eq!(hue_event(50.0, 0.0, 200.0), ColorPickerEvent::SetHue(90.0));
        assert_eq!(alpha_event(10.0, 0.0, 0.0), ColorPickerEvent::SetAlpha(0.0));
    }

    #[test]
    fn keys() {
        assert_eq!(area_keydown("ArrowRight", false), Some(ColorPickerEvent::NudgeArea { ds: 1.0, dv: 0.0 }));
        assert_eq!(area_keydown("ArrowUp", true), Some(ColorPickerEvent::NudgeArea { ds: 0.0, dv: 10.0 }));
        assert_eq!(hue_keydown("ArrowLeft", true), Some(ColorPickerEvent::NudgeHue(-10.0)));
        assert_eq!(hue_keydown("End", false), Some(ColorPickerEvent::SetHue(360.0)));
        assert_eq!(alpha_keydown("Home", false), Some(ColorPickerEvent::SetAlpha(0.0)));
        assert_eq!(hex_keydown("Enter"), Some(ColorPickerEvent::HexCommit));
        assert_eq!(hue_keydown("a", false), None);
    }
}
