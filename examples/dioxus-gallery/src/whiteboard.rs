use dioxus::prelude::*;
use hydroxy_dioxus::whiteboard::{Point, Rect, Shape, ShapeKind, Whiteboard};

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
        Shape::new("goal-text", ShapeKind::Text, Rect::new(656.0, 240.0, 200.0, 34.0))
            .color(3)
            .text("Quarantine flaky tests"),
        Shape::arrow("a1", Point::new(644.0, 148.0), Point::new(700.0, 220.0)).color(3).bind(Some("n3"), Some("goal")),
    ]
}

#[component]
pub fn WhiteboardDemo() -> Element {
    let mut count = use_signal(|| retro().len());

    rsx! {
        div { style: "display: grid; gap: var(--space-3);",
            Whiteboard {
                default_shapes: retro(),
                label: "Retro board",
                file_name: "sprint-42-retro",
                on_change: move |shapes: Vec<Shape>| count.set(shapes.len()),
            }
            p { style: "margin: 0; font: var(--text-label); color: var(--text-3);",
                "{count} shapes. Double-click a note to edit it; drag an arrow end onto a shape to attach it."
            }
        }
    }
}
