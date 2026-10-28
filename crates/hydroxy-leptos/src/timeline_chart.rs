//! `<TimelineChart>`: spans and events in lanes on a date axis. Placement and calendar
//! ticks come from `hydroxy_charts::timeline`; layout is plain CSS percentages.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{nav, timeline};
use leptos::prelude::*;

use crate::plot_kit::{colour, pct};

pub use hydroxy_charts::timeline::{DAY, HOUR, Item, Lane, ms_from_civil};

/// Time ranges and moments, one row group per lane. Overlapping items in a lane stack.
/// Times are milliseconds since the Unix epoch (UTC); see [`ms_from_civil`].
/// Hover or touch an item to read its dates; focus the chart and use the arrow keys to
/// step through items lane by lane, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn TimelineChart(
    /// The lanes, top to bottom.
    #[prop(into)]
    lanes: Signal<Vec<Lane>>,
    /// What the chart shows, such as "Release plan". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Formats times for the tooltip, live region and table. Default: "5 Oct 2026", with
    /// the time when it is not midnight.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called when the hovered or focused item changes; `None` when it clears.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on an item.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| timeline::format_date(v), |f| f.run(v));
    let layout = Memo::new(move |_| lanes.with(|l| timeline::layout_lanes(l, 6)));
    let when = move |i: &Item| if i.is_event() { fmt(i.start) } else { format!("{} to {}", fmt(i.start), fmt(i.end)) };
    let active = RwSignal::new(None::<usize>);
    let count = move || lanes.with(|l| l.iter().map(|lane| lane.items.len()).sum::<usize>());
    let describe = move |at: usize| {
        lanes.with(|l| {
            let (lane, item) = l.iter().flat_map(|lane| lane.items.iter().map(move |i| (lane, i))).nth(at)?;
            Some(format!("{}: {}, {}", lane.name, item.label, when(item)))
        })
    };
    // `value` is the item's start time in ms since the epoch; `series` is its lane.
    let datum = Signal::derive(move || {
        let mut at = active.get()?;
        lanes.with(|l| {
            l.iter().enumerate().find_map(|(li, lane)| match lane.items.get(at) {
                Some(item) => Some(EventDatum {
                    series: li,
                    index: at,
                    label: Some(item.label.clone()),
                    value: item.start,
                    node: None,
                }),
                None => {
                    at -= lane.items.len();
                    None
                }
            })
        })
    });
    crate::callbacks::watch_hover(Some(datum), on_hover);
    let group_label = format!("{label}. Use the arrow keys to step through items.");
    let caption = label;

    view! {
        <figure data-scope="timeline-chart" data-part="root">
            <div
                data-part="body"
                role="group"
                tabindex="0"
                aria-label=group_label
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:click=move |_| crate::callbacks::select_active(Some(datum), on_select)
                on:keydown=move |ev| {
                    if ev.key() == "Enter" {
                            crate::callbacks::select_active(Some(datum), on_select);
                        }
                    if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), count()) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
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
                            let start = offset;
                            let placed = &l.items[offset..offset + lane.items.len()];
                            offset += lane.items.len();
                            let items = lane
                                .items
                                .iter()
                                .zip(placed)
                                .enumerate()
                                .map(|(ii, (item, p))| {
                                    let at = start + ii;
                                    let kind = if item.is_event() { "event" } else { "span" };
                                    let text = format!("{}, {}", item.label, when(item));
                                    view! {
                                        <div
                                            data-part="item"
                                            data-kind=kind
                                            data-flip=(p.left > 0.6).then_some("")
                                            role="img"
                                            aria-label=text
                                            data-active=move || (active.get() == Some(at)).then_some("")
                                            on:pointerenter=move |_| active.set(Some(at))
                                            on:pointerdown=move |_| active.set(Some(at))
                                            style=format!(
                                                "left: {}; width: {}; --row: {}; --lane: {li}; --ox-colour: {};",
                                                pct(p.left),
                                                pct(p.width),
                                                p.row,
                                                colour(li),
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
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
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
