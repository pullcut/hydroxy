//! `SpiralChart`: a time series wound onto a spiral, one turn per period, each value a
//! short arc shaded by value. Geometry comes from `hydroxy_charts::spiral`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{format_value, raster, spiral};

use crate::callbacks::{select_active, use_hover};
use crate::chart_frame::use_export;
use crate::line_chart::plot_fractions;
use crate::plot_kit::pct;
use crate::sparkline::use_datum;

/// A spiral that turns once per `period` values, so the same moment in every period lines
/// up along a spoke and seasonal patterns show as wedges of colour. The spiral draws out
/// from the centre. Hover or touch an arc to read it; focus the chart and use Left/Right to
/// step along the spiral, Up/Down to move a whole turn, Home/End to jump, Escape to clear.
#[component]
pub fn SpiralChart(
    /// The values in time order. Non-finite values leave a gap.
    values: ReadSignal<Vec<f64>>,
    /// A name for each value, such as its date, for the tooltip and table.
    labels: ReadSignal<Vec<String>>,
    /// Values per turn: 365 for daily values over years, 24 × 7 for hourly values over weeks.
    period: f64,
    /// Names spaced evenly around the dial, clockwise from 12 o'clock, such as the months.
    #[props(default)]
    spokes: ReadSignal<Vec<String>>,
    /// A name for each turn, from the centre out, such as the years. Shown at 12 o'clock.
    #[props(default)]
    turns: ReadSignal<Vec<String>>,
    /// What the chart shows, such as "Daily cycling distance". Used as the accessible name.
    label: String,
    /// Plot diameter in px.
    #[props(default = 360)]
    size: u16,
    /// Formats values for the tooltip, legend and table.
    format: Option<Callback<f64, String>>,
    /// Called when the hovered or focused arc changes; `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on an arc.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let layout = use_memo(move || spiral::layout(&values.read(), period));
    let mut active = use_signal(|| None::<usize>);
    let mut box_size = use_signal(|| (0.0_f64, 0.0_f64));
    let name = move |i: usize| labels.read().get(i).cloned().unwrap_or_else(|| (i + 1).to_string());
    use_export(move |()| {
        let l = layout.read();
        std::iter::once(vec![String::new(), "Value".to_string()])
            .chain(l.segments.iter().enumerate().map(|(i, s)| vec![name(i), s.value.map_or_else(String::new, fmt)]))
            .collect()
    });
    let l = layout.read();
    let len = l.segments.len();
    let current = active().filter(|&i| i < len);
    let datum = use_datum(current.and_then(|i| Some(EventDatum::category(i, name(i), l.segments[i].value?))));
    use_hover(datum, on_hover);
    let value_text = |v: Option<f64>| v.map_or_else(|| "No data".to_string(), fmt);
    let announcement = current.map(|i| format!("{}: {}", name(i), value_text(l.segments[i].value))).unwrap_or_default();
    let hover = move |e: Event<PointerData>| {
        let Some((x, y)) = plot_fractions(e.data().element_coordinates(), box_size()) else { return };
        let next = spiral::index_at(&layout.read(), period, x, 1.0 - y);
        if active() != next {
            active.set(next);
        }
    };
    let width = format!("{:.3}", l.spacing * 0.78);
    let spoke_names = spokes();
    let turn_names = turns();
    let shown = raster::table_rows(len, 2);

    rsx! {
        figure { "data-scope": "spiral-chart", "data-part": "root",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the left and right arrow keys to step along the spiral, up and down to move a turn.",
                style: "width: {size}px;",
                onresize: move |e| {
                    if let Ok(s) = e.data().get_border_box_size() {
                        box_size.set((s.width, s.height));
                    }
                },
                onpointermove: hover,
                onpointerdown: hover,
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onclick: move |_| select_active(datum, on_select),
                onkeydown: move |e| {
                    if e.key() == Key::Enter {
                        select_active(datum, on_select);
                    }
                    if let Some(next) = spiral::key(&e.key().to_string(), active(), len, period) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg { view_box: "0 0 100 100", "aria-hidden": "true",
                    for j in 0..spoke_names.len() {
                        path { "data-part": "spoke", d: spiral::spoke(&l, j, spoke_names.len()).0 }
                    }
                    for (i , s) in l.segments.iter().enumerate() {
                        path {
                            key: "{i}",
                            "data-part": "segment",
                            "data-empty": s.value.is_none().then_some(""),
                            "pathLength": "1",
                            stroke_width: "{width}",
                            style: "stroke: {spiral::shade(s.level)}; --t0: {s.start:.4}; --dt: {s.span:.4};",
                            d: "{s.path}",
                        }
                    }
                    if let Some(i) = current {
                        path { "data-part": "highlight", stroke_width: "{width}", d: "{l.segments[i].path}" }
                    }
                }
                for (j , text) in spoke_names.iter().enumerate() {
                    {
                        let (_, (x, y)) = spiral::spoke(&l, j, spoke_names.len());
                        rsx! {
                            span {
                                "data-part": "spoke-label",
                                "data-side": spiral::side(x),
                                "aria-hidden": "true",
                                style: "left: {pct(x)}; top: {pct(y)};",
                                "{text}"
                            }
                        }
                    }
                }
                for (k , text) in turn_names.iter().take(l.turns).enumerate() {
                    {
                        let (x, y) = spiral::turn_anchor(&l, k);
                        rsx! {
                            span { "data-part": "turn-label", "aria-hidden": "true", style: "left: {pct(x)}; top: {pct(y)};", "{text}" }
                        }
                    }
                }
                if let Some(i) = current {
                    {
                        let (x, y) = spiral::anchor(&l, period, i);
                        rsx! {
                            div {
                                "data-part": "tooltip",
                                "data-side": if x > 0.5 { "left" } else { "right" },
                                style: "left: {pct(x)}; top: {pct(y)};",
                                "aria-hidden": "true",
                                div { "data-part": "tooltip-label", {name(i)} }
                                div { "data-part": "tooltip-value", {value_text(l.segments[i].value)} }
                            }
                        }
                    }
                }
            }
            div { "data-part": "scale", "aria-hidden": "true",
                span { "data-part": "scale-value", {fmt(l.min)} }
                span { "data-part": "ramp" }
                span { "data-part": "scale-value", {fmt(l.max)} }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption {
                    "{label}"
                    if shown < len {
                        " (first {shown} of {len} values)"
                    }
                }
                thead {
                    tr {
                        th { scope: "col" }
                        th { scope: "col", "Value" }
                    }
                }
                tbody {
                    for (i , s) in l.segments.iter().take(shown).enumerate() {
                        tr {
                            th { scope: "row", {name(i)} }
                            td { {s.value.map_or_else(|| "–".to_string(), fmt)} }
                        }
                    }
                }
            }
        }
    }
}
