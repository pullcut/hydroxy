//! Image cropper connector. Parts, all under `data-scope="image-cropper"`:
//! root, viewport (scrolls when zoomed), stage, image, shade, box (the crop
//! area, focusable), grid, handle (eight, `data-handle`), toolbar, aspect
//! (preset toggle buttons), rotate-left, rotate-right, zoom (a native range
//! input), reset, readout and live.
//!
//! Everything on the stage is placed in percent of the rotated image, so the
//! adapter only converts pointer positions to view pixels with [`to_view_px`].

use hydroxy_machines::Modifiers;
pub use hydroxy_machines::image_cropper::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "image-cropper";

/// Aspect presets offered in the toolbar: label and width / height.
pub const PRESETS: [(&str, Option<f64>); 5] =
    [("Free", None), ("1:1", Some(1.0)), ("4:3", Some(4.0 / 3.0)), ("16:9", Some(16.0 / 9.0)), ("3:2", Some(1.5))];

/// Whether a preset matches the current lock.
#[must_use]
pub fn preset_active(preset: Option<f64>, current: Option<f64>) -> bool {
    match (preset, current) {
        (None, None) => true,
        (Some(a), Some(b)) => (a - b).abs() < 1e-6,
        _ => false,
    }
}

/// A pointer position, given the stage's client rect, in view pixels.
#[must_use]
pub fn to_view_px(state: &ImageCropperState, x: f64, y: f64, rect: (f64, f64, f64, f64)) -> (f64, f64) {
    let (left, top, width, height) = rect;
    let (vw, vh) = state.view_size();
    let sx = if width > 0.0 { vw / width } else { 0.0 };
    let sy = if height > 0.0 { vh / height } else { 0.0 };
    ((x - left) * sx, (y - top) * sy)
}

/// Ids derived from one root id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The image element, found by the drag script.
    pub img: String,
    /// The hint text, referenced by `aria-describedby`.
    pub hint: String,
    /// The zoom slider.
    pub zoom: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { img: format!("{root}-img"), hint: format!("{root}-hint"), zoom: format!("{root}-zoom") }
}

/// `aria-valuetext` for the zoom slider: the zoom as a whole percentage.
#[must_use]
pub fn zoom_valuetext(state: &ImageCropperState) -> String {
    format!("{}%", (state.zoom * 100.0).round())
}

fn pct(v: f64, of: f64) -> f64 {
    if of > 0.0 { v / of * 100.0 } else { 0.0 }
}

/// `style` for the stage: the rotated image's aspect ratio and the zoom, as
/// `--ratio` and `--zoom` for the CSS to size it with.
#[must_use]
pub fn stage_style(state: &ImageCropperState) -> String {
    let (vw, vh) = state.view_size();
    let ratio = if state.is_loaded() { vw / vh } else { 4.0 / 3.0 };
    format!("--ratio: {ratio:.6}; --zoom: {};", state.zoom)
}

/// `style` for the image: sized to the unrotated image within the stage, then turned.
#[must_use]
pub fn image_style(state: &ImageCropperState) -> String {
    let (w, h) = state.image;
    let (vw, vh) = state.view_size();
    if !state.is_loaded() {
        return "width: 100%; height: 100%;".to_string();
    }
    format!(
        "width: {:.4}%; height: {:.4}%; transform: translate(-50%, -50%) rotate({}deg);",
        pct(w, vw),
        pct(h, vh),
        u16::from(state.turns) * 90
    )
}

/// `style` for the crop box, in percent of the stage.
#[must_use]
pub fn box_style(state: &ImageCropperState) -> String {
    let (vw, vh) = state.view_size();
    let r = state.crop;
    format!(
        "left: {:.4}%; top: {:.4}%; width: {:.4}%; height: {:.4}%;",
        pct(r.x, vw),
        pct(r.y, vh),
        pct(r.w, vw),
        pct(r.h, vh)
    )
}

/// The crop in words, for the readout and live region.
#[must_use]
pub fn describe(state: &ImageCropperState) -> String {
    if !state.is_loaded() {
        return "Loading image".to_string();
    }
    let c = state.crop();
    let turn = if c.rotation == 0 { String::new() } else { format!(", rotated {}°", c.rotation) };
    format!("{} × {} at {}, {}{turn}", c.width, c.height, c.x, c.y)
}

/// Map a key on the focused crop box. Arrows move by one step (Shift: ten),
/// Alt+arrows resize from the bottom-right, `+` / `-` zoom, `[` / `]` rotate.
#[must_use]
pub fn box_key(key: &str, mods: Modifiers, state: &ImageCropperState) -> Option<ImageCropperEvent> {
    let Modifiers { shift, alt, .. } = mods;
    let step = state.step() * if shift { 10.0 } else { 1.0 };
    let (dx, dy) = match key {
        "ArrowLeft" => (-step, 0.0),
        "ArrowRight" => (step, 0.0),
        "ArrowUp" => (0.0, -step),
        "ArrowDown" => (0.0, step),
        "+" | "=" => return Some(ImageCropperEvent::SetZoom(state.zoom + 0.25)),
        "-" | "_" => return Some(ImageCropperEvent::SetZoom(state.zoom - 0.25)),
        "[" => return Some(ImageCropperEvent::Rotate(false)),
        "]" => return Some(ImageCropperEvent::Rotate(true)),
        _ => return None,
    };
    Some(if alt { ImageCropperEvent::Grow { dw: dx, dh: dy } } else { ImageCropperEvent::Nudge { dx, dy } })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loaded() -> ImageCropperState {
        ImageCropperState {
            image: (1000.0, 500.0),
            crop: Rect { x: 100.0, y: 50.0, w: 800.0, h: 400.0 },
            ..ImageCropperState::default()
        }
    }

    #[test]
    fn styles_are_in_percent_of_the_stage() {
        let s = loaded();
        assert_eq!(box_style(&s), "left: 10.0000%; top: 10.0000%; width: 80.0000%; height: 80.0000%;");
        assert_eq!(stage_style(&s), "--ratio: 2.000000; --zoom: 1;");
        let r = ImageCropperState { turns: 1, ..s };
        assert!(image_style(&r).starts_with("width: 200.0000%; height: 50.0000%;"));
    }

    #[test]
    fn pointer_maps_to_view_pixels() {
        let s = loaded();
        assert_eq!(to_view_px(&s, 150.0, 75.0, (100.0, 50.0, 500.0, 250.0)), (100.0, 50.0));
    }

    #[test]
    fn box_key_nudges_grows_and_rotates() {
        let s = loaded();
        assert_eq!(box_key("ArrowLeft", Modifiers::SHIFT, &s), Some(ImageCropperEvent::Nudge { dx: -100.0, dy: 0.0 }));
        assert_eq!(box_key("ArrowDown", Modifiers::ALT, &s), Some(ImageCropperEvent::Grow { dw: 0.0, dh: 10.0 }));
        assert_eq!(box_key("]", Modifiers::NONE, &s), Some(ImageCropperEvent::Rotate(true)));
        assert_eq!(box_key("a", Modifiers::NONE, &s), None);
    }

    #[test]
    fn readout_and_presets() {
        assert_eq!(describe(&loaded()), "800 × 400 at 100, 50");
        assert!(preset_active(Some(1.5), Some(3.0 / 2.0)));
        assert!(!preset_active(None, Some(1.0)));
    }
}
