use leptos::prelude::*;
use oxide_leptos::whiteboard::{Point, Rect, Shape, ShapeKind, Whiteboard};

fn retro() -> Vec<Shape> {
    let note = |id: &str, x: f64, y: f64, colour: usize, text: &str| {
        Shape::new(id, ShapeKind::Sticky, Rect::new(x, y, 200.0, 140.0)).color(colour).text(text)
    };
    vec![
        Shape::new("title", ShapeKind::Text, Rect::new(0.0, -72.0, 520.0, 34.0)).color(0).text("Sprint 42 retro"),
        Shape::new("went-well", ShapeKind::Rect, Rect::new(-16.0, -16.0, 464.0, 360.0)).color(4),
        Shape::new("to-fix", ShapeKind::Rect, Rect::new(512.0, -16.0, 464.0, 360.0)).color(2),
        note("n1", 8.0, 8.0, 4, "Checkout redesign shipped two days early"),
        note("n2", 224.0, 8.0, 4, "Pairing on the billing migration worked well"),
        note("n3", 536.0, 8.0, 2, "Flaky search tests blocked three deploys"),
        note("n4", 752.0, 8.0, 5, "Too many meetings on Thursday"),
        Shape::new("goal", ShapeKind::Ellipse, Rect::new(616.0, 200.0, 256.0, 112.0)).color(3),
        Shape::new("goal-text", ShapeKind::Text, Rect::new(656.0, 240.0, 200.0, 34.0)).color(3).text("Quarantine flaky tests"),
        Shape::arrow("a1", Point::new(644.0, 148.0), Point::new(700.0, 220.0)).color(3).bind(Some("n3"), Some("goal")),
    ]
}

pub fn demo() -> AnyView {
    let (count, set_count) = signal(retro().len());
    let on_change = move |shapes: Vec<Shape>| set_count.set(shapes.len());
    view! {
        <div style="display: grid; gap: var(--space-3);">
            <Whiteboard
                default_shapes=retro()
                label="Retro board"
                file_name="sprint-42-retro"
                on_change=on_change
            />
            <p style="margin: 0; font: var(--text-label); color: var(--text-3);">
                {move || format!("{} shapes. Double-click a note to edit it; drag an arrow end onto a shape to attach it.", count.get())}
            </p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "whiteboard",
    name: "Whiteboard",
    group: "Data",
    summary: "An infinite canvas with rectangles, ellipses, arrows that stay attached to shapes, smoothed freehand ink, sticky notes and text. Resize, rotate, restack, duplicate, undo, and export as JSON or SVG.",
    demo,
    leptos: include_str!("whiteboard.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/whiteboard.rs"),
    parts: &[
        &["Whiteboard", "root", "Wrapper for the toolbar, canvas and screen reader text"],
        &["", "toolbar", "role=toolbar with groups for tools, colours, editing, zoom and export"],
        &["", "tool", "Tool button, aria-pressed when active, data-tool"],
        &["", "swatch", "Colour button, --ox-swatch from the chart tokens; recolours the selection"],
        &["", "action", "Undo, redo, duplicate, restack, delete, zoom and export buttons"],
        &["", "viewport", "role=application canvas that takes every pointer and key event. aria-label summarises the scene. data-tool, data-panning"],
        &["", "canvas / world", "The <svg> and its pan and zoom group"],
        &["", "shape", "<g> per shape with data-kind; the same markup the SVG export writes. data-hint on an arrow's bind target"],
        &["", "selection / handle / stalk", "Selection outline, eight resize handles, the rotate handle and its stalk; arrow ends for a lone arrow"],
        &["", "marquee", "Drag-select box"],
        &["", "editor", "<textarea> over a sticky note or text while editing"],
        &["", "instructions / live", "Visually hidden keyboard help and a role=status region"],
    ],
    keys: &[
        &["V H R O A P N T", "Select, hand, rectangle, ellipse, arrow, pen, sticky note, text"],
        &["Enter", "With a drawing tool, adds a shape in the middle of the view; on a selected note, edits it"],
        &["] / [", "Select the next or previous shape (announced)"],
        &["Arrow keys", "Move the selection 1px, Shift for 10px. Pan when nothing is selected"],
        &["Alt + Arrow keys", "Resize the selection"],
        &[", / .", "Rotate the selection 15°"],
        &["Delete / Backspace", "Delete the selection"],
        &["Ctrl/Cmd + D", "Duplicate"],
        &["Ctrl/Cmd + Z, Shift + Z, Y", "Undo and redo"],
        &["Ctrl/Cmd + A", "Select everything"],
        &["Ctrl/Cmd + ] / [, with Shift", "Bring forward or send backward; to front or back"],
        &["+ / − / 0 / 1", "Zoom in, zoom out, back to 100%, fit everything"],
        &["Escape", "Stop editing, cancel a drag, clear the selection, then back to the select tool"],
        &["Wheel, Ctrl + wheel, pinch", "Pan, zoom, zoom"],
        &["Shift while dragging", "Square shapes, keep proportions, snap rotation to 15°"],
    ],
    props: &[
        &["default_shapes", "Vec<Shape>", "Starting scene. Shape::new(id, kind, rect).color(i).text(..); Shape::arrow(id, a, b).bind(start, end)"],
        &["on_change", "Callback<Vec<Shape>>", "Called with every shape after each committed change"],
        &["label", "String", "Accessible name for the canvas"],
        &["height", "String", "CSS height of the canvas. Default 520px"],
        &["file_name", "String", "Export file name without extension"],
    ],
};
