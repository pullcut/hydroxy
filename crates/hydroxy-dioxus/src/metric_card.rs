//! `MetricCard`: one KPI with its change against the previous value and an optional
//! trend sparkline. The change is shown by sign, arrow and colour together, never
//! colour alone. Maths from `hydroxy_charts::metric` and `hydroxy_charts::sparkline`.

use crate::icon::{Icon, IconName};
use dioxus::prelude::*;
use hydroxy_charts::line::Curve;
use hydroxy_charts::{format_value, metric, nav, sparkline};

const SPARK_W: f64 = 160.0;
const SPARK_H: f64 = 40.0;
const SPARK_PAD: f64 = 3.0;

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// A KPI tile. With a trend, hover or touch the sparkline to read a point, or focus it
/// and use the arrow keys to step through points, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn MetricCard(
    /// What the figure measures, in sentence case.
    label: String,
    /// The current value.
    value: ReadSignal<f64>,
    /// The value to compare against, such as last month's.
    #[props(default)]
    previous: ReadSignal<Option<f64>>,
    /// What the comparison is against, such as "vs last month".
    comparison: Option<String>,
    /// Unit after the figure, such as "ms".
    unit: Option<String>,
    /// Whether a rise is good news (default) or bad, such as for error rates.
    #[props(default = true)]
    higher_is_better: bool,
    /// Recent values, oldest first, drawn as a sparkline.
    #[props(default)]
    trend: ReadSignal<Vec<f64>>,
    /// A name for each trend point, such as its date. Default "Point n".
    #[props(default)]
    trend_labels: ReadSignal<Vec<String>>,
    /// Formats the figure, tooltip and table values.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let v = value();
    let prev = previous();
    let delta = prev.map(|p| metric::delta(v, p, higher_is_better));
    let points = trend();
    let names = trend_labels();
    let n = points.len();
    let spark = sparkline::layout(&points, SPARK_W, SPARK_H, SPARK_PAD, Curve::Linear);
    let mut active = use_signal(|| None::<usize>);
    let mut width = use_signal(|| 0.0_f64);
    let current = active().filter(|&i| i < n);
    let point_name = |i: usize| names.get(i).cloned().unwrap_or_else(|| format!("Point {}", i + 1));
    let point_value = |i: usize| points.get(i).copied().filter(|v| v.is_finite());
    let announcement = current
        .map(|i| format!("{}: {}", point_name(i), point_value(i).map_or_else(|| "no value".into(), fmt)))
        .unwrap_or_default();
    let move_to = move |e: Event<PointerData>| {
        let w = width();
        if w > 0.0 {
            let near = metric::spark_index(e.data().element_coordinates().x / w, n, SPARK_W, SPARK_PAD);
            if active() != near {
                active.set(near);
            }
        }
    };
    let tip = current.and_then(|i| {
        metric::spark_point(&points, i, SPARK_W, SPARK_H, SPARK_PAD).map(|(x, y)| (i, x / SPARK_W, y / SPARK_H))
    });
    let unit_head = unit.clone().map(|u| format!(" ({u})")).unwrap_or_default();

    rsx! {
        div { "data-scope": "metric-card", "data-part": "root",
            span { "data-part": "label", "{label}" }
            p { "data-part": "figure",
                span { "data-part": "value", {fmt(v)} }
                if let Some(u) = unit.clone() {
                    span { "data-part": "unit", "{u}" }
                }
            }
            if let Some(d) = delta {
                div { "data-part": "delta", "data-trend": d.trend.attr(), "data-tone": d.tone,
                    span { "data-part": "delta-icon",
                        Icon { name: if d.trend == metric::Trend::Flat { IconName::ArrowRight } else { IconName::ArrowUp }, size: 14 }
                    }
                    span { "data-part": "delta-value",
                        {d.ratio.map_or_else(|| format!("{}{}", if d.change > 0.0 { "+" } else { "" }, fmt(d.change)), metric::signed_percent)}
                    }
                    if let Some(c) = comparison.clone() {
                        span { "data-part": "comparison", "{c}" }
                    }
                }
            }
            if n > 1 {
                div {
                    "data-part": "spark",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label} trend. Use the arrow keys to step through points.",
                    style: "aspect-ratio: {SPARK_W} / {SPARK_H};",
                    onresize: move |e| {
                        if let Ok(s) = e.data().get_border_box_size() {
                            width.set(s.width);
                        }
                    },
                    onpointermove: move_to,
                    onpointerdown: move_to,
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if let Some(next) = nav::list_key(&e.key().to_string(), active(), n) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    svg { view_box: "0 0 {SPARK_W} {SPARK_H}", "preserveAspectRatio": "none", "aria-hidden": "true",
                        path { "data-part": "area", d: "{spark.area}" }
                        path { "data-part": "line", d: "{spark.line}", "pathLength": "1" }
                    }
                    if let Some((i , fx , fy)) = tip {
                        span { "data-part": "marker", style: "left: {pct(fx)}; top: {pct(fy)};" }
                        div {
                            "data-part": "tooltip",
                            "data-side": if fx > 0.5 { "left" } else { "right" },
                            style: "left: {pct(fx)}; top: {pct(fy)};",
                            "aria-hidden": "true",
                            div { "data-part": "tooltip-label", {point_name(i)} }
                            ul {
                                li { "data-part": "tooltip-row",
                                    span { "data-part": "tooltip-name", "Value" }
                                    span { "data-part": "tooltip-value", {point_value(i).map(fmt).unwrap_or_default()} }
                                }
                            }
                        }
                    }
                    if let Some((x , y)) = spark.last {
                        span { "data-part": "last", style: "left: {pct(x / SPARK_W)}; top: {pct(y / SPARK_H)};" }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Point" }
                        th { scope: "col", "Value{unit_head}" }
                    }
                }
                tbody {
                    tr {
                        th { scope: "row", "Current" }
                        td { {fmt(v)} }
                    }
                    if let Some(p) = prev {
                        tr {
                            th { scope: "row", {comparison.clone().unwrap_or_else(|| "Previous".into())} }
                            td { {fmt(p)} }
                        }
                    }
                    for i in 0..n {
                        tr {
                            th { scope: "row", {point_name(i)} }
                            td { {point_value(i).map(fmt).unwrap_or_default()} }
                        }
                    }
                }
            }
        }
    }
}
