//! `<CalendarHeatmapChart>`: one cell per day in week columns, shaded by value, like a
//! contribution graph. Spans over a year stack one row per calendar year. Geometry comes
//! from `hydroxy_charts::calendar`.

use leptos::prelude::*;
use hydroxy_charts::calendar::{self, WEEKDAYS};
use hydroxy_charts::scale::days_from_civil;
use hydroxy_charts::{format_value, raster};

use crate::chart_frame::provide_export;

/// A day-by-day calendar shaded from the lowest value (or zero) to the highest in one
/// chart colour. Hover or touch a day to read it; focus the calendar and use Up/Down to
/// step a day, Left/Right a week, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn CalendarHeatmapChart(
    /// One value per day, from `start`. Non-finite values draw as empty days.
    #[prop(into)]
    values: Signal<Vec<f64>>,
    /// The first day, as (year, month 1..=12, day).
    #[prop(into)]
    start: Signal<(i64, u32, u32)>,
    /// What the chart shows, such as "Commits per day". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Formats values for the tooltip, legend and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| {
        let (y, m, d) = start.get();
        values.with(|v| calendar::layout(days_from_civil(y, m, d), v))
    });
    let active = RwSignal::new(None::<usize>);
    let value_text = move |v: Option<f64>| v.map_or_else(|| "No data".to_string(), fmt);
    provide_export(move || {
        let l = layout.get_untracked();
        std::iter::once(vec!["Date".to_string(), "Value".to_string()])
            .chain(l.days.iter().map(|d| vec![calendar::day_name(d.date), d.value.map_or_else(String::new, fmt)]))
            .collect()
    });
    let describe = move |i: usize| layout.with(|l| l.days.get(i).map(|d| format!("{}: {}", calendar::day_name(d.date), value_text(d.value))));

    view! {
        <figure data-scope="calendar-heatmap-chart" data-part="root" style=move || format!("--weeks: {};", layout.with(|l| l.weeks.max(1)))>
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=format!("{label}. Use the arrow keys to move between days.")
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if let Some(next) = calendar::key(&ev.key(), active.get_untracked(), layout.with(|l| l.days.len())) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                {move || {
                    let l = layout.get();
                    l.blocks
                        .iter()
                        .map(|b| {
                            let months = b
                                .months
                                .iter()
                                .map(|m| view! { <span data-part="month" style=format!("--col: {};", m.week + 1)>{m.name}</span> })
                                .collect_view();
                            let cells = l.days[b.first..b.end]
                                .iter()
                                .enumerate()
                                .map(|(k, d)| {
                                    let i = b.first + k;
                                    let tip = move || {
                                        (active.get() == Some(i))
                                            .then(|| {
                                                let (name, value) = layout.with(|l| (calendar::day_name(l.days[i].date), value_text(l.days[i].value)));
                                                view! {
                                                    <span data-part="tooltip" aria-hidden="true">
                                                        <span data-part="tooltip-label">{name}</span>
                                                        <span data-part="tooltip-value">{value}</span>
                                                    </span>
                                                }
                                            })
                                    };
                                    view! {
                                        <span
                                            data-part="cell"
                                            data-empty=d.value.is_none().then_some("")
                                            data-active=move || (active.get() == Some(i)).then_some("")
                                            data-edge=calendar::edge(d.week, l.weeks)
                                            style=format!("--col: {}; --row: {}; --level: {:.3}; --i: {};", d.week + 1, d.weekday + 1, d.level, d.week)
                                            on:pointerenter=move |_| active.set(Some(i))
                                            on:pointerdown=move |_| active.set(Some(i))
                                        >
                                            {tip}
                                        </span>
                                    }
                                })
                                .collect_view();
                            view! {
                                <div data-part="block">
                                    {(!b.label.is_empty()).then(|| view! { <span data-part="year">{b.label.clone()}</span> })}
                                    <div data-part="months" aria-hidden="true">{months}</div>
                                    <div data-part="weekdays" aria-hidden="true">
                                        {WEEKDAYS.iter().enumerate().map(|(r, w)| view! { <span data-part="weekday">{(r % 2 == 0).then_some(*w)}</span> }).collect_view()}
                                    </div>
                                    <div data-part="cells">{cells}</div>
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>
            <div data-part="scale" aria-hidden="true">
                <span data-part="scale-value">{move || fmt(layout.with(|l| l.min))}</span>
                <span data-part="ramp"></span>
                <span data-part="scale-value">{move || fmt(layout.with(|l| l.max))}</span>
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>
                    {label.clone()}
                    {move || {
                        let total = layout.with(|l| l.days.len());
                        let shown = raster::table_rows(total, 2);
                        (shown < total).then(|| format!(" (first {shown} of {total} days)"))
                    }}
                </caption>
                <thead>
                    <tr>
                        <th scope="col">"Date"</th>
                        <th scope="col">"Value"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let l = layout.get();
                        let shown = raster::table_rows(l.days.len(), 2);
                        l.days
                            .iter()
                            .take(shown)
                            .map(|d| view! { <tr><th scope="row">{calendar::day_name(d.date)}</th><td>{d.value.map_or_else(|| "–".to_string(), fmt)}</td></tr> })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
