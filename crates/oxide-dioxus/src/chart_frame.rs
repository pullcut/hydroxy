//! `ChartFrame`: the shared shell around a chart: title, description, legend, and
//! the empty and loading states.

use dioxus::prelude::*;

/// A titled card section for any chart. While `loading` the chart is replaced by a
/// pulsing placeholder; when `empty` it is replaced by `empty_text`.
#[component]
pub fn ChartFrame(
    /// Heading, such as "Interviews read".
    title: Option<String>,
    /// One line under the title saying what the numbers are.
    description: Option<String>,
    /// Series names for a shared legend, in the categorical colour order. Leave empty when the chart has its own.
    #[props(default)]
    legend: Vec<String>,
    /// There is nothing to plot.
    #[props(default)]
    empty: bool,
    /// Data is on its way. Takes precedence over `empty`.
    #[props(default)]
    loading: bool,
    /// Message shown when empty.
    #[props(default = "No data for this period".to_string())]
    empty_text: String,
    /// Height in px of the placeholder shown while loading or empty.
    #[props(default = 180)]
    placeholder_height: u16,
    /// The chart.
    children: Element,
) -> Element {
    let state = if loading {
        "loading"
    } else if empty {
        "empty"
    } else {
        "ready"
    };
    rsx! {
        section {
            "data-scope": "chart-frame",
            "data-part": "root",
            "data-state": state,
            "aria-busy": loading.then_some("true"),
            if title.is_some() || description.is_some() {
                header { "data-part": "header",
                    if let Some(t) = title {
                        h3 { "data-part": "title", "{t}" }
                    }
                    if let Some(d) = description {
                        p { "data-part": "description", "{d}" }
                    }
                }
            }
            div { "data-part": "body",
                // The chart stays mounted and is only hidden, so toggling states never replays its entrance.
                div { "data-part": "chart", hidden: state != "ready", {children} }
                match state {
                    "loading" => rsx! {
                        div {
                            "data-part": "loading",
                            role: "status",
                            "aria-label": "Loading chart",
                            style: "min-height: {placeholder_height}px;",
                        }
                    },
                    "empty" => rsx! {
                        div { "data-part": "empty", style: "min-height: {placeholder_height}px;", "{empty_text}" }
                    },
                    _ => rsx! {},
                }
            }
            if !legend.is_empty() {
                ul { "data-part": "legend",
                    for (i , name) in legend.iter().enumerate() {
                        li { "data-part": "legend-item",
                            span { "data-part": "swatch", style: "background: var(--chart-{i % 6 + 1});" }
                            "{name}"
                        }
                    }
                }
            }
        }
    }
}
