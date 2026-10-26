//! `<ChartFrame>`: the shared shell around a chart, with a title, description, legend,
//! and the empty and loading states.

use leptos::either::EitherOf3;
use leptos::prelude::*;

/// A titled card section for any chart. While `loading` the chart is replaced by a
/// pulsing placeholder; when `empty` it is replaced by `empty_text`.
#[component]
pub fn ChartFrame(
    /// Heading, such as "Interviews read".
    #[prop(optional, into)]
    title: Option<String>,
    /// One line under the title saying what the numbers are.
    #[prop(optional, into)]
    description: Option<String>,
    /// Series names for a shared legend, in the categorical colour order. Leave empty when the chart has its own.
    #[prop(optional, into)]
    legend: Vec<String>,
    /// There is nothing to plot.
    #[prop(optional, into)]
    empty: Signal<bool>,
    /// Data is on its way. Takes precedence over `empty`.
    #[prop(optional, into)]
    loading: Signal<bool>,
    /// Message shown when empty.
    #[prop(default = "No data for this period".into(), into)]
    empty_text: String,
    /// Height in px of the placeholder shown while loading or empty.
    #[prop(default = 180)]
    placeholder_height: u16,
    /// The chart.
    children: Children,
) -> impl IntoView {
    let state = move || {
        if loading.get() {
            "loading"
        } else if empty.get() {
            "empty"
        } else {
            "ready"
        }
    };
    let has_head = title.is_some() || description.is_some();
    let legend_items = (!legend.is_empty()).then(|| {
        legend
            .into_iter()
            .enumerate()
            .map(|(i, name)| {
                view! {
                    <li data-part="legend-item">
                        <span data-part="swatch" style=format!("background: var(--chart-{});", i % 6 + 1)></span>
                        {name}
                    </li>
                }
            })
            .collect_view()
    });
    let height = format!("min-height: {placeholder_height}px;");

    view! {
        <section data-scope="chart-frame" data-part="root" data-state=state aria-busy=move || (state() == "loading").then_some("true")>
            {has_head
                .then(|| {
                    view! {
                        <header data-part="header">
                            {title.map(|t| view! { <h3 data-part="title">{t}</h3> })}
                            {description.map(|d| view! { <p data-part="description">{d}</p> })}
                        </header>
                    }
                })}
            <div data-part="body">
                // The chart stays mounted and is only hidden, so toggling states never replays its entrance.
                <div data-part="chart" hidden=move || state() != "ready">{children()}</div>
                {move || match state() {
                    "loading" => {
                        EitherOf3::A(view! {
                            <div data-part="loading" role="status" aria-label="Loading chart" style=height.clone()></div>
                        })
                    }
                    "empty" => EitherOf3::B(view! { <div data-part="empty" style=height.clone()>{empty_text.clone()}</div> }),
                    _ => EitherOf3::C(()),
                }}
            </div>
            {legend_items.map(|items| view! { <ul data-part="legend">{items}</ul> })}
        </section>
    }
}
