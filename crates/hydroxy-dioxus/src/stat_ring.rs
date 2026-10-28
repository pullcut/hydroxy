//! `StatRing`: a compact ring showing progress towards a goal, with the figure in the
//! middle. Progress comes from `hydroxy_charts::metric::progress`.

use dioxus::prelude::*;
use hydroxy_charts::{format_value, metric};

/// A progress ring. Hover, touch or focus it to read the value against the goal;
/// Escape hides the reading.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn StatRing(
    /// The current value.
    value: ReadSignal<f64>,
    /// The goal the ring fills towards.
    goal: ReadSignal<f64>,
    /// What the ring measures, such as "Weekly active". Shown under the figure.
    label: String,
    /// Ring diameter in px.
    #[props(default = 112)]
    size: u16,
    /// Ring width in px.
    #[props(default = 10)]
    thickness: u16,
    /// Formats the value and goal. The centre shows the share of the goal.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let (v, g) = (value(), goal());
    let progress = metric::progress(v, g);
    // The centre shows the true share, so 112% reads as over the goal while the ring stays full.
    let share = if g > 0.0 { format!("{}%", format_value((v / g * 100.0).round())) } else { "—".into() };
    let mut active = use_signal(|| false);
    let r = (f64::from(size) - f64::from(thickness)) / 2.0;
    let c = f64::from(size) / 2.0;
    let announcement = if active() { format!("{label}: {} of {}, {share}", fmt(v), fmt(g)) } else { String::new() };

    rsx! {
        figure { "data-scope": "stat-ring", "data-part": "root", "data-over": (v > g && g > 0.0).then_some(""),
            div {
                "data-part": "ring",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Focus to read the value against the goal.",
                style: "width: {size}px; height: {size}px;",
                onpointerenter: move |_| active.set(true),
                onpointerdown: move |_| active.set(true),
                onpointerleave: move |_| active.set(false),
                onfocus: move |_| active.set(true),
                onblur: move |_| active.set(false),
                onkeydown: move |e| {
                    if e.key() == Key::Escape && active() {
                        e.prevent_default();
                        active.set(false);
                    }
                },
                svg { view_box: "0 0 {size} {size}", "aria-hidden": "true",
                    circle { "data-part": "track", cx: "{c}", cy: "{c}", r: "{r}", stroke_width: "{thickness}" }
                    circle {
                        "data-part": "value",
                        cx: "{c}",
                        cy: "{c}",
                        r: "{r}",
                        stroke_width: "{thickness}",
                        "pathLength": "1",
                        transform: "rotate(-90 {c} {c})",
                        style: "stroke-dasharray: {progress:.4} 1;",
                    }
                }
                div { "data-part": "center", "aria-hidden": "true",
                    span { "data-part": "center-value", "{share}" }
                    span { "data-part": "center-label", "{label}" }
                }
                if active() {
                    div { "data-part": "tooltip", style: "left: 100%; top: 50%;", "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: var(--chart-1);" }
                            "{label}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Value" }
                                span { "data-part": "tooltip-value", {fmt(v)} }
                            }
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Goal" }
                                span { "data-part": "tooltip-value", {fmt(g)} }
                            }
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Measure" }
                        th { scope: "col", "Value" }
                        th { scope: "col", "Goal" }
                        th { scope: "col", "Progress" }
                    }
                }
                tbody {
                    tr {
                        th { scope: "row", "{label}" }
                        td { {fmt(v)} }
                        td { {fmt(g)} }
                        td { "{share}" }
                    }
                }
            }
        }
    }
}
