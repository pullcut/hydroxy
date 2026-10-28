//! Signature pad connector. Parts, all under `data-scope="signature-pad"`:
//! root, modes (with a `mode` button each), surface (the drawing `<svg>`),
//! stroke, guide, typed-input, typed-preview, actions (undo, clear), live and
//! the hidden form input.
//!
//! Drawing is pointer-only, so typing a name is offered as the keyboard
//! alternative: the two modes are a pair of toggle buttons, and a polite live
//! region says what the pad holds after each change.

pub use oxide_machines::signature_pad::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "signature-pad";

/// The `viewBox` of the drawing surface.
#[must_use]
pub fn view_box() -> String {
    format!("0 0 {VIEW_WIDTH} {VIEW_HEIGHT}")
}

/// A pointer position on the surface, given its client rect, as a sample in view units.
#[must_use]
pub fn to_point(x: f64, y: f64, rect: (f64, f64, f64, f64), t: f64) -> Point {
    let (left, top, width, height) = rect;
    let fx = if width > 0.0 { (x - left) / width } else { 0.0 };
    let fy = if height > 0.0 { (y - top) / height } else { 0.0 };
    Point { x: fx * VIEW_WIDTH, y: fy * VIEW_HEIGHT, t }
}

/// Path data for the stroke being drawn.
#[must_use]
pub fn live_path(state: &SignaturePadState, input: &SignaturePadInput) -> String {
    state.live.as_deref().map(|p| stroke_path(p, input.min_width, input.max_width)).unwrap_or_default()
}

/// What the live region says about the pad.
#[must_use]
pub fn status(state: &SignaturePadState) -> String {
    match state.mode {
        Mode::Draw => match state.strokes.len() {
            0 => "Signature empty".to_string(),
            1 => "Signature drawn, 1 stroke".to_string(),
            n => format!("Signature drawn, {n} strokes"),
        },
        Mode::Type if state.typed.trim().is_empty() => "Signature empty".to_string(),
        Mode::Type => format!("Signed as {}", state.typed.trim()),
    }
}

/// The text submitted with a form: `drawn:` plus path data, or `typed:` plus the name.
#[must_use]
pub fn form_value(state: &SignaturePadState) -> String {
    match state.value() {
        Some(SignatureValue::Drawn(d)) => format!("drawn:{d}"),
        Some(SignatureValue::Typed(name)) => format!("typed:{name}"),
        None => String::new(),
    }
}

/// Escape text for an SVG/XML text node or attribute.
fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// A standalone SVG document for a signature, drawn in `color`.
#[must_use]
pub fn to_svg(value: &SignatureValue, color: &str) -> String {
    let body = match value {
        SignatureValue::Drawn(d) => format!(r#"<path d="{}" fill="{}"/>"#, escape(d), escape(color)),
        SignatureValue::Typed(name) => format!(
            r#"<text x="{}" y="{}" text-anchor="middle" dominant-baseline="middle" font-family="{}" font-size="64" fill="{}">{}</text>"#,
            VIEW_WIDTH / 2.0,
            VIEW_HEIGHT / 2.0,
            escape(SCRIPT_FONTS),
            escape(color),
            escape(name)
        ),
    };
    format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{}">{body}</svg>"#, view_box())
}

/// Script faces for the typed signature, falling back to any cursive font.
pub const SCRIPT_FONTS: &str = "'Segoe Script', 'Snell Roundhand', 'Brush Script MT', 'Apple Chancery', 'URW Chancery L', cursive";

/// Map a key on the root: Ctrl/Cmd+Z undoes the last stroke.
#[must_use]
pub fn root_key(key: &str, mod_key: bool) -> Option<SignaturePadEvent> {
    (mod_key && key.eq_ignore_ascii_case("z")).then_some(SignaturePadEvent::Undo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_maps_into_view_units() {
        let p = to_point(150.0, 60.0, (100.0, 50.0, 300.0, 100.0), 5.0);
        assert!((p.x - 100.0).abs() < 1e-9 && (p.y - 20.0).abs() < 1e-9);
    }

    #[test]
    fn status_and_form_value() {
        let mut s = SignaturePadState::default();
        assert_eq!(status(&s), "Signature empty");
        assert_eq!(form_value(&s), "");
        s.mode = Mode::Type;
        s.typed = "Grace Hopper".into();
        assert_eq!(status(&s), "Signed as Grace Hopper");
        assert_eq!(form_value(&s), "typed:Grace Hopper");
    }

    #[test]
    fn svg_export_escapes_text() {
        let svg = to_svg(&SignatureValue::Typed("A & B".into()), "#111");
        assert!(svg.contains("A &amp; B") && svg.starts_with("<svg"));
        assert!(to_svg(&SignatureValue::Drawn("M0 0Z".into()), "#111").contains(r#"d="M0 0Z""#));
    }

    #[test]
    fn mod_z_undoes() {
        assert_eq!(root_key("z", true), Some(SignaturePadEvent::Undo));
        assert_eq!(root_key("z", false), None);
    }
}
