//! The overview strip under a zoomable [`LineChart`](super::LineChart).

use dioxus::prelude::*;
use hydroxy_charts::brush::{Brush, Drag, Grab};

use crate::plot_kit::colour;

/// The overview strip under a zoomable `LineChart`: a faint copy of every line over the
/// full range, with a draggable window and two keyboard-operable edges.
#[component]
#[allow(clippy::too_many_arguments, reason = "component props")]
pub(super) fn BrushStrip(
    window: Signal<Brush>,
    overview: Memo<Vec<String>>,
    len: usize,
    start_label: String,
    end_label: String,
    lo: usize,
    hi: usize,
    label: String,
) -> Element {
    let mut window = window;
    let mut drag = use_signal(|| None::<Drag>);
    let mut width = use_signal(|| 0.0_f64);
    #[allow(clippy::cast_precision_loss, reason = "point counts are small")]
    let step = 1.0 / len.saturating_sub(1).max(1) as f64;
    let fraction = move |e: &Event<PointerData>| {
        let w = width();
        (w > 0.0).then(|| (e.data().element_coordinates().x / w).clamp(0.0, 1.0))
    };
    let b = window();
    let edge = |grab: Grab, name: &'static str, index: usize, text: &str| {
        let at = if grab == Grab::Start { b.start } else { b.end };
        rsx! {
            div {
                "data-part": "brush-handle",
                "data-edge": name,
                role: "slider",
                tabindex: "0",
                "aria-label": "{label} range {name}",
                "aria-valuemin": "0",
                "aria-valuemax": "{len.saturating_sub(1)}",
                "aria-valuenow": "{index}",
                "aria-valuetext": "{text}",
                style: "left: {at * 100.0:.3}%;",
                onkeydown: move |e| {
                    let shift = e.modifiers().shift();
                    if let Some(next) = window().key(grab, &e.key().to_string(), shift, step, step) {
                        e.prevent_default();
                        window.set(next);
                    }
                },
            }
        }
    };

    rsx! {
        div {
            "data-part": "brush",
            role: "group",
            "aria-label": "{label}: zoom range",
            onresize: move |e| {
                if let Ok(size) = e.data().get_border_box_size() {
                    width.set(size.width);
                }
            },
            onpointerdown: move |e| {
                if let Some(f) = fraction(&e) {
                    let (next, d) = window().begin(f, 0.025);
                    window.set(next);
                    drag.set(Some(d));
                }
            },
            onpointermove: move |e| {
                if let (Some(d), Some(f)) = (drag(), fraction(&e)) {
                    window.set(d.apply(f, step));
                }
            },
            onpointerup: move |_| drag.set(None),
            onpointercancel: move |_| drag.set(None),
            onpointerleave: move |_| drag.set(None),
            svg { "data-part": "brush-marks", view_box: "0 0 100 100", "preserveAspectRatio": "none", "aria-hidden": "true",
                for (i , d) in overview.read().iter().enumerate() {
                    path { key: "{i}", "data-part": "brush-line", d: "{d}", stroke: colour(i) }
                }
            }
            div { "data-part": "brush-window", style: "left: {b.start * 100.0:.3}%; width: {b.span() * 100.0:.3}%;" }
            {edge(Grab::Start, "start", lo, &start_label)}
            {edge(Grab::End, "end", hi.saturating_sub(1), &end_label)}
        }
    }
}
