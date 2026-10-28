//! `<WordCloudChart>`: weighted words placed along a spiral from the centre, heaviest
//! first and largest. Placement is deterministic, from `hydroxy_charts::wordcloud`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, nav, wordcloud};
use leptos::prelude::*;

use crate::plot_kit::ChartTip;

pub use hydroxy_charts::wordcloud::Word;

/// A word cloud. Hover, touch or focus a word to read its weight; focus the cloud and
/// use the arrow keys to step through words from heaviest to lightest, Home and End to
/// jump, Escape to clear. Words that do not fit are left out of the picture but kept
/// in the table.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn WordCloudChart(
    /// The words and their weights, in any order.
    #[prop(into)]
    words: Signal<Vec<Word>>,
    /// What the chart shows, such as "Themes in feedback". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Layout width in px. The cloud scales down to fit narrower containers.
    #[prop(default = 520)]
    width: u16,
    /// Layout height in px.
    #[prop(default = 300)]
    height: u16,
    /// Name of the weight, shown in the tooltip and table.
    #[prop(default = "Weight".into(), into)]
    weight_label: String,
    /// Formats weights for the tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called when the hovered or focused word changes; `None` when it clears.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a word.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let (w, h) = (f64::from(width), f64::from(height));
    let placed = Memo::new(move |_| {
        words.with(|ws| wordcloud::layout(ws, wordcloud::Options { width: w, height: h, ..Default::default() }))
    });
    let active = RwSignal::new(None::<usize>);
    let len = move || placed.with(Vec::len);
    let heavy = move || len().div_ceil(3);
    let word = move |i: usize| {
        placed.with(|p| p.get(i).copied()).and_then(|p| words.with(|ws| ws.get(p.index).cloned()).map(|x| (p, x)))
    };
    let weight_name = StoredValue::new(weight_label);
    let describe = move |i: usize| {
        word(i).map(|(_, x)| format!("{}: {} {}", x.text, weight_name.with_value(|n| n.to_lowercase()), fmt(x.weight)))
    };

    let datum = Signal::derive(move || {
        let (spot, x) = word(active.get()?)?;
        Some(EventDatum::category(spot.index, x.text, x.weight))
    });
    crate::callbacks::watch_hover(Some(datum), on_hover);

    view! {
        <figure data-scope="word-cloud-chart" data-part="root">
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=format!("{label}. Use the arrow keys to step through words, heaviest first.")
                style=format!("max-width: {width}px; aspect-ratio: {width} / {height};")
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:click=move |_| crate::callbacks::select_active(Some(datum), on_select)
                on:keydown=move |ev| {
                    if ev.key() == "Enter" {
                            crate::callbacks::select_active(Some(datum), on_select);
                        }
                    let key = ev.key();
                    if let Some(next) = nav::list_key(nav::linear_key(&key), active.get_untracked(), len()) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <svg viewBox=format!("0 0 {width} {height}") aria-hidden="true">
                    {move || {
                        let heavy = heavy();
                        placed
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, p)| {
                                let text = words.with(|ws| ws.get(p.index).map(|x| x.text.clone())).unwrap_or_default();
                                view! {
                                    <text
                                        data-part="word"
                                        data-heavy=(i < heavy).then_some("")
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        x=format!("{:.1}", p.x)
                                        y=format!("{:.1}", p.y)
                                        font-size=format!("{:.1}", p.font_size)
                                        text-anchor="middle"
                                        dominant-baseline="central"
                                        style=format!("--i: {};", i.min(40))
                                        on:pointerenter=move |_| active.set(Some(i))
                                        on:pointerdown=move |_| active.set(Some(i))
                                    >
                                        {text}
                                    </text>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                {move || {
                    let i = active.get()?;
                    let (spot, x) = word(i)?;
                    let (fx, fy) = (spot.x / w, spot.y / h);
                    let tip = Tip {
                        x: fx,
                        y: 1.0 - fy,
                        title: x.text,
                        rows: vec![TipRow::new(None, weight_name.get_value(), fmt(x.weight))],
                    };
                    Some(view! { <ChartTip tip /> })
                }}
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label.clone()}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Word"</th>
                        <th scope="col">{weight_name.get_value()}</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        words
                            .get()
                            .into_iter()
                            .map(|x| {
                                view! {
                                    <tr>
                                        <th scope="row">{x.text}</th>
                                        <td>{fmt(x.weight)}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
