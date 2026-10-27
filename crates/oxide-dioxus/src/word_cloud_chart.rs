//! `WordCloudChart`: weighted words placed along a spiral from the centre, heaviest
//! first and largest. Placement is deterministic, from `oxide_charts::wordcloud`.

use dioxus::prelude::*;
use oxide_charts::{format_value, funnel, nav, wordcloud};

pub use oxide_charts::wordcloud::Word;

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// A word cloud. Hover, touch or focus a word to read its weight; focus the cloud and
/// use the arrow keys to step through words from heaviest to lightest, Home and End to
/// jump, Escape to clear. Words that do not fit are left out of the picture but kept
/// in the table.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn WordCloudChart(
    /// The words and their weights, in any order.
    words: ReadSignal<Vec<Word>>,
    /// What the chart shows, such as "Themes in feedback". Used as the accessible name.
    label: String,
    /// Layout width in px. The cloud scales down to fit narrower containers.
    #[props(default = 520)]
    width: u16,
    /// Layout height in px.
    #[props(default = 300)]
    height: u16,
    /// Name of the weight, shown in the tooltip and table.
    #[props(default = "Weight".to_string())]
    weight_label: String,
    /// Formats weights for the tooltip and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let (w, h) = (f64::from(width), f64::from(height));
    let all = words();
    let placed = wordcloud::layout(&all, wordcloud::Options { width: w, height: h, ..Default::default() });
    let n = placed.len();
    let heavy = n.div_ceil(3);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let announcement = current
        .map(|i| {
            let x = &all[placed[i].index];
            format!("{}: {} {}", x.text, weight_label.to_lowercase(), fmt(x.weight))
        })
        .unwrap_or_default();

    rsx! {
        figure { "data-scope": "word-cloud-chart", "data-part": "root",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through words, heaviest first.",
                style: "max-width: {width}px; aspect-ratio: {width} / {height};",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    let key = e.key().to_string();
                    if let Some(next) = nav::list_key(funnel::arrow_alias(&key), active(), n) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg { view_box: "0 0 {width} {height}", "aria-hidden": "true",
                    for (i , p) in placed.iter().enumerate() {
                        text {
                            key: "{p.index}",
                            "data-part": "word",
                            "data-heavy": (i < heavy).then_some(""),
                            "data-active": (current == Some(i)).then_some(""),
                            x: "{p.x:.1}",
                            y: "{p.y:.1}",
                            font_size: "{p.font_size:.1}",
                            text_anchor: "middle",
                            dominant_baseline: "central",
                            style: "--i: {i.min(40)};",
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerdown: move |_| active.set(Some(i)),
                            "{all[p.index].text}"
                        }
                    }
                }
                if let Some(i) = current {
                    div {
                        "data-part": "tooltip",
                        "data-side": if placed[i].x / w > 0.5 { "left" } else { "right" },
                        style: "left: {pct(placed[i].x / w)}; top: {pct(placed[i].y / h)};",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label", "{all[placed[i].index].text}" }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "{weight_label}" }
                                span { "data-part": "tooltip-value", {fmt(all[placed[i].index].weight)} }
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
                        th { scope: "col", "Word" }
                        th { scope: "col", "{weight_label}" }
                    }
                }
                tbody {
                    for x in all.iter() {
                        tr {
                            th { scope: "row", "{x.text}" }
                            td { {fmt(x.weight)} }
                        }
                    }
                }
            }
        }
    }
}
