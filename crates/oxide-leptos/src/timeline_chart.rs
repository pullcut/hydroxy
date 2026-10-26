//! `<TimelineChart>`: spans and events in lanes on a date axis. Placement and calendar
//! ticks come from `oxide_charts::timeline`; layout is plain CSS percentages.

use leptos::prelude::*;
use oxide_charts::timeline;

pub use oxide_charts::timeline::{DAY, HOUR, Item, Lane, ms_from_civil};

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// Time ranges and moments, one row group per lane. Overlapping items in a lane stack.
/// Times are milliseconds since the Unix epoch (UTC); see [`ms_from_civil`].
/// Items are focusable and show their dates on hover or focus.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn TimelineChart(
    /// The lanes, top to bottom.
    #[prop(into)]
    lanes: Signal<Vec<Lane>>,
    /// What the chart shows, such as "Release plan". Used as the accessible name.
    #[prop(into)]
    label: String,
) -> impl IntoView {
    let layout = Memo::new(move |_| lanes.with(|l| timeline::layout_lanes(l, 6)));
    let when = |i: &Item| {
        if i.is_event() { timeline::format_date(i.start) } else { format!("{} to {}", timeline::format_date(i.start), timeline::format_date(i.end)) }
    };

    view! {
        <figure data-scope="timeline-chart" data-part="root">
            <div data-part="body" role="group" aria-label=label.clone()>
                <div data-part="grid" aria-hidden="true">
                    {move || {
                        layout
                            .get()
                            .ticks
                            .into_iter()
                            .map(|t| view! { <div data-part="gridline" style=format!("left: {};", pct(t.fraction))></div> })
                            .collect_view()
                    }}
                </div>
                {move || {
                    let l = layout.get();
                    let mut offset = 0;
                    lanes
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(li, lane)| {
                            let rows = l.lane_rows.get(li).copied().unwrap_or(1);
                            let placed = &l.items[offset..offset + lane.items.len()];
                            offset += lane.items.len();
                            let items = lane
                                .items
                                .iter()
                                .zip(placed)
                                .map(|(item, p)| {
                                    let kind = if item.is_event() { "event" } else { "span" };
                                    let text = format!("{}, {}", item.label, when(item));
                                    view! {
                                        <div
                                            data-part="item"
                                            data-kind=kind
                                            data-flip=(p.left > 0.6).then_some("")
                                            role="img"
                                            aria-label=text
                                            tabindex="0"
                                            style=format!(
                                                "left: {}; width: {}; --row: {}; --lane: {li}; --ox-colour: var(--chart-{});",
                                                pct(p.left),
                                                pct(p.width),
                                                p.row,
                                                li % 6 + 1,
                                            )
                                        >
                                            {item.is_event().then(|| view! { <span data-part="marker"></span> })}
                                            <span data-part="item-label">{item.label.clone()}</span>
                                            <div data-part="tooltip" aria-hidden="true">
                                                <div data-part="tooltip-label">{lane.name.clone()}</div>
                                                <dl><dt>{item.label.clone()}</dt><dd>{when(item)}</dd></dl>
                                            </div>
                                        </div>
                                    }
                                })
                                .collect_view();
                            view! {
                                <div data-part="lane">
                                    <div data-part="lane-name">{lane.name.clone()}</div>
                                    <div data-part="track" style=format!("--rows: {rows};")>{items}</div>
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>
            <div data-part="axis" aria-hidden="true">
                {move || {
                    layout
                        .get()
                        .ticks
                        .into_iter()
                        .map(|t| view! { <span data-part="tick" style=format!("left: {};", pct(t.fraction))>{t.label}</span> })
                        .collect_view()
                }}
            </div>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Lane"</th>
                        <th scope="col">"Item"</th>
                        <th scope="col">"When"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        lanes
                            .get()
                            .into_iter()
                            .flat_map(|lane| {
                                let name = lane.name;
                                lane.items.into_iter().map(move |i| {
                                    view! { <tr><th scope="row">{name.clone()}</th><td>{i.label.clone()}</td><td>{when(&i)}</td></tr> }
                                })
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
