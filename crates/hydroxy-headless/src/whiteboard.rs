//! Whiteboard connector. Anatomy: root, toolbar, tool, swatch, action,
//! viewport, canvas (the `<svg>`), world, shape, selection, handle, marquee,
//! editor (the text `<textarea>`), instructions, live. Every part carries
//! `data-scope="whiteboard"` and its `data-part`.
//!
//! The viewport is one focusable `role="application"` element that receives
//! every pointer and key event. Shapes are drawn from [`shape_svg`], the same
//! markup the SVG export uses, so what you see is what you export.

use std::fmt::Write as _;

pub use hydroxy_machines::whiteboard::*;

/// The icon name (see [`crate::node_editor::icon`]) for a tool's toolbar button.
#[must_use]
pub const fn tool_icon(tool: Tool) -> &'static str {
    match tool {
        Tool::Select => "select",
        Tool::Hand => "hand",
        Tool::Rect => "rect",
        Tool::Ellipse => "ellipse",
        Tool::Arrow => "arrow",
        Tool::Pen => "pen",
        Tool::Sticky => "sticky",
        Tool::Text => "text",
    }
}

/// The `data-scope` value on every part.
pub const SCOPE: &str = "whiteboard";

/// Read by screen readers on the canvas, through `aria-describedby`.
pub const INSTRUCTIONS: &str = "Tools: V select, H hand, R rectangle, O ellipse, A arrow, P pen, N sticky note, T text. With a drawing tool, press Enter to add a shape in the middle of the view. Right and left square brackets select the next or previous shape. Arrow keys move the selection, Alt with the arrow keys resizes it, comma and full stop rotate it; Shift moves further. Enter edits a selected note, Escape stops editing. Delete removes the selection; Control or Command with D duplicates, with Z undoes, with Shift and Z redoes, with A selects everything, with square brackets changes the stacking order. Plus and minus zoom, 0 resets the zoom and 1 fits everything.";

/// The instructions' id.
#[must_use]
pub fn instructions_id(root: &str) -> String {
    format!("{root}-instructions")
}

/// Inline style for the viewport: grid dots that follow the view.
#[must_use]
pub fn viewport_style(v: Viewport) -> String {
    let mut step = 24.0 * v.zoom;
    while step < 12.0 {
        step *= 2.0;
    }
    format!(
        "--ox-grid: {step:.2}px; --ox-grid-x: {:.2}px; --ox-grid-y: {:.2}px;",
        v.x.rem_euclid(step),
        v.y.rem_euclid(step)
    )
}

/// SVG `transform` for the world group.
#[must_use]
pub fn world_transform(v: Viewport) -> String {
    format!("translate({:.2} {:.2}) scale({:.4})", v.x, v.y, v.zoom)
}

/// One handle to draw: which, its centre (world) and radius (world).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandleDraw {
    /// Which handle.
    pub handle: Handle,
    /// Centre, world.
    pub at: Point,
}

/// The selection outline and its handles, in world units.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionDraw {
    /// `transform` for an outline rect at the origin.
    pub transform: String,
    /// Outline width.
    pub w: f64,
    /// Outline height.
    pub h: f64,
    /// The rotate stalk from the top edge to the rotate handle, when shown.
    pub stalk: Option<(Point, Point)>,
    /// Handles.
    pub handles: Vec<HandleDraw>,
}

/// What to draw around the selection. `None` when nothing is selected or while drawing.
#[must_use]
pub fn selection_draw(state: &WhiteboardState) -> Option<SelectionDraw> {
    if matches!(state.gesture, Some(Gesture::Create { .. })) || state.editing.is_some() {
        return None;
    }
    let zoom = state.viewport.zoom;
    if let Some(a) = state.lone_arrow() {
        let (s, e) = arrow_ends(a, &state.shapes);
        return Some(SelectionDraw {
            transform: String::new(),
            w: 0.0,
            h: 0.0,
            stalk: None,
            handles: vec![HandleDraw { handle: Handle::Start, at: s }, HandleDraw { handle: Handle::End, at: e }],
        });
    }
    let f = state.frame()?;
    let mut handles: Vec<HandleDraw> =
        Handle::RESIZE.iter().map(|h| HandleDraw { handle: *h, at: f.handle(*h, zoom) }).collect();
    let top = f.handle(Handle::N, zoom);
    let rot = f.handle(Handle::Rotate, zoom);
    handles.push(HandleDraw { handle: Handle::Rotate, at: rot });
    Some(SelectionDraw { transform: f.transform(), w: f.w, h: f.h, stalk: Some((top, rot)), handles })
}

/// Screen-space style for the text editor over the shape being edited.
#[must_use]
pub fn editor_style(state: &WhiteboardState) -> Option<String> {
    let s = state.editing()?;
    let v = state.viewport;
    let at = v.to_screen(Point::new(s.x, s.y));
    let (font, pad) = if s.kind == ShapeKind::Text { (TEXT_FONT, 4.0) } else { (STICKY_FONT, STICKY_PAD) };
    let colour = if s.kind == ShapeKind::Text { swatch(s.color).css() } else { "var(--text-1)".to_string() };
    Some(format!(
        "left: {:.1}px; top: {:.1}px; width: {:.1}px; height: {:.1}px; padding: {pad:.1}px; font-size: {font:.1}px; line-height: {:.3}; color: {colour}; transform-origin: 0 0; transform: scale({:.4}) translate({:.1}px, {:.1}px) rotate({:.2}deg) translate({:.1}px, {:.1}px);",
        at.x,
        at.y,
        s.w,
        s.h,
        line_height(s.kind) / font,
        v.zoom,
        s.w / 2.0,
        s.h / 2.0,
        s.rotation.to_degrees(),
        -s.w / 2.0,
        -s.h / 2.0,
    ))
}

/// `aria-label` for the canvas: a summary of the scene.
#[must_use]
pub fn canvas_label(label: &str, state: &WhiteboardState) -> String {
    let n = state.shapes.len();
    let sel = state.selection.len();
    let mut out = format!("{label}, {n} shape{}", if n == 1 { "" } else { "s" });
    if sel == 1 {
        if let Some(s) = state.selected().next() {
            let _ = write!(out, ", selected: {}", describe(s));
        }
    } else if sel > 1 {
        let _ = write!(out, ", {sel} selected");
    }
    out
}

/// A `data:` URL for a download link of `body` with `mime`.
#[must_use]
pub fn data_url(mime: &str, body: &str) -> String {
    let mut out = format!("data:{mime};charset=utf-8,");
    for b in body.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            out.push(char::from(b));
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::Machine;

    #[test]
    fn selection_has_eight_handles_and_a_rotator() {
        let mut s = WhiteboardMachine::initial(&WhiteboardInput {
            default_shapes: vec![Shape::new("a", ShapeKind::Rect, Rect::new(0.0, 0.0, 100.0, 50.0))],
        });
        assert!(selection_draw(&s).is_none());
        s.selection.insert("a".into());
        let d = selection_draw(&s).unwrap();
        assert_eq!(d.handles.len(), 9);
        assert_eq!(d.transform, "translate(0.00 0.00)");
    }

    #[test]
    fn data_urls_escape_markup() {
        assert_eq!(
            data_url("image/svg+xml", "<a href=\"#\">"),
            "data:image/svg+xml;charset=utf-8,%3Ca%20href%3D%22%23%22%3E"
        );
    }
}
