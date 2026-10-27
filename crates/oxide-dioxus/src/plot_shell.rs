//! The frame shared by the distribution and financial charts: gridlines and ticks,
//! annotations, the focusable overlay with its marker and tooltip, legend, live region
//! and screen-reader table. Each chart passes its marks as children.

use dioxus::prelude::*;
use oxide_charts::probe::{self, Pick, Readout};
use oxide_charts::{format_value, nav};

/// `var(--chart-N)` for a colour slot.
pub(crate) fn colour(slot: usize) -> String {
    format!("var(--chart-{})", slot.min(5) + 1)
}

/// A fraction as a CSS percentage.
pub(crate) fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// The shared frame. `active` is the focused stop, so marks can highlight it too.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub(crate) fn PlotShell(
    /// `data-scope` of the root.
    scope: &'static str,
    /// Accessible name and table caption.
    label: String,
    /// How to use the keyboard, appended to the overlay's name.
    hint: &'static str,
    /// Plot height in px.
    height: u16,
    /// Ticks, stops, legend, notes and table.
    readout: Readout,
    /// Formats values.
    format: Option<Callback<f64, String>>,
    /// The focused stop.
    active: Signal<Option<usize>>,
    /// The marks.
    children: Element,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let mut size = use_signal(|| (0.0_f64, 0.0_f64));
    let current = active().and_then(|i| readout.stops.get(i).cloned());
    let announcement = current.as_ref().map(|s| probe::describe(s, fmt)).unwrap_or_default();
    let (stops, pick) = (readout.stops.clone(), readout.pick);
    let move_to = move |e: Event<PointerData>| {
        let (w, h) = size();
        if w > 0.0 && h > 0.0 {
            let c = e.data().element_coordinates();
            let near = probe::nearest(&stops, c.x / w, 1.0 - c.y / h, pick);
            if active() != near {
                active.set(near);
            }
        }
    };
    let move_to_down = move_to.clone();
    let count = readout.stops.len();
    let band = readout.x_bands.then_some("");

    rsx! {
        figure { "data-scope": scope, "data-part": "root",
            div { "data-part": "plot", style: "height: {height}px;",
                div { "data-part": "grid", "aria-hidden": "true",
                    for t in readout.y_ticks.iter() {
                        div { "data-part": "gridline", "data-axis": "y", style: "bottom: {pct(t.fraction)};",
                            span { "data-part": "tick", {t.label.show(fmt)} }
                        }
                    }
                    for t in readout.x_ticks.iter() {
                        div { "data-part": "gridline", "data-axis": "x", "data-band": band, style: "left: {pct(t.fraction)};",
                            span { "data-part": "tick", {t.label.show(fmt)} }
                        }
                    }
                }
                {children}
                for n in readout.notes.iter() {
                    div { "data-part": "annotation", "data-shape": n.shape.attr(), style: n.style(), "aria-hidden": "true",
                        if let Some(l) = n.label.clone() {
                            span { "data-part": "annotation-label", "{l}" }
                        }
                    }
                }
                div {
                    "data-part": "overlay",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. {hint}",
                    onresize: move |e| {
                        if let Ok(s) = e.data().get_border_box_size() {
                            size.set((s.width, s.height));
                        }
                    },
                    onpointermove: move_to,
                    onpointerdown: move_to_down,
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        let key = e.key().to_string();
                        let key = match (pick, key.as_str()) {
                            (Pick::Row, "ArrowDown") => "ArrowRight",
                            (Pick::Row, "ArrowUp") => "ArrowLeft",
                            (_, k) => k,
                        };
                        if let Some(next) = nav::list_key(key, active(), count) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    if let Some(s) = current {
                        span {
                            "data-part": "marker",
                            style: "left: {pct(s.x)}; bottom: {pct(s.y)}; background: {colour(s.colour.unwrap_or(0))};",
                        }
                        div {
                            "data-part": "tooltip",
                            "data-side": if s.x > 0.5 { "left" } else { "right" },
                            "data-vertical": if s.y > 0.5 { "below" } else { "above" },
                            style: "left: {pct(s.x)}; bottom: {pct(s.y)};",
                            "aria-hidden": "true",
                            div { "data-part": "tooltip-label",
                                if let Some(c) = s.colour {
                                    span { "data-part": "swatch", style: "background: {colour(c)};" }
                                }
                                {s.title.show(fmt)}
                            }
                            ul {
                                for r in s.rows.iter() {
                                    li { "data-part": "tooltip-row",
                                        span { "data-part": "tooltip-name",
                                            if let Some(c) = r.colour {
                                                span { "data-part": "swatch", style: "background: {colour(c)};" }
                                            }
                                            "{r.name}"
                                        }
                                        span { "data-part": "tooltip-value", {r.value.show(fmt)} }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !readout.legend.is_empty() {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (name , c) in readout.legend.iter() {
                        li { "data-part": "legend-item",
                            span { "data-part": "swatch", style: "background: {colour(*c)};" }
                            "{name}"
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        for h in readout.head.iter() {
                            th { scope: "col", "{h}" }
                        }
                    }
                }
                tbody {
                    for row in readout.rows.iter() {
                        tr {
                            th { scope: "row", {row.first().map(|c| c.show(fmt)).unwrap_or_default()} }
                            for c in row.iter().skip(1) {
                                td { {c.show(fmt)} }
                            }
                        }
                    }
                }
            }
        }
    }
}
