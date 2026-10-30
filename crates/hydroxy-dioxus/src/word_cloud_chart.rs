//! `WordCloudChart`: weighted words placed along a spiral from the centre, heaviest
//! first and largest. Placement is deterministic, from `hydroxy_charts::wordcloud`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, funnel, nav, wordcloud};

use crate::callbacks::{select_active, use_hover};
use crate::plot_kit::ChartTip;
use crate::sparkline::use_datum;

pub use hydroxy_charts::wordcloud::Word;

/// A word cloud. Hover, touch or focus a word to read its weight; focus the cloud and
/// use the arrow keys to step through words from heaviest to lightest, Home and End to
/// jump, Escape to clear. Words that do not fit are left out of the picture but kept
/// in the table.
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
    /// Called when the hovered or focused word changes; `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a word.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let (w, h) = (f64::from(width), f64::from(height));
    let all = words();
    let placed = wordcloud::layout(&all, wordcloud::Options { width: w, height: h, ..Default::default() });
    let n = placed.len();
    let heavy = n.div_ceil(3);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let datum = use_datum(current.map(|i| {
        let x = &all[placed[i].index];
        EventDatum::category(placed[i].index, x.text.clone(), x.weight)
    }));
    use_hover(datum, on_hover);
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
                onclick: move |_| select_active(datum, on_select),
                onkeydown: move |e| {
                    if e.key() == Key::Enter {
                        select_active(datum, on_select);
                    }
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
                            style: "--i: {i}; --ox-n: {placed.len()};",
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerdown: move |_| active.set(Some(i)),
                            "{all[p.index].text}"
                        }
                    }
                }
                if let Some(i) = current {
                    ChartTip {
                        tip: Tip {
                            x: placed[i].x / w,
                            y: 1.0 - placed[i].y / h,
                            title: all[placed[i].index].text.clone(),
                            rows: vec![TipRow::new(None, weight_label.clone(), fmt(all[placed[i].index].weight))],
                        },
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
