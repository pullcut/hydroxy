//! `<UpsetChart>`: how sets overlap, for more sets than a Venn diagram can hold. Bars on
//! top count the members of each exclusive intersection, the dot matrix below shows which
//! sets it combines, and bars on the left give each set's size. Layout comes from
//! `hydroxy_charts::upset`.

use hydroxy_charts::{format_value, nav, upset};
use leptos::prelude::*;

use crate::sets_kit::SetsTip;

pub use hydroxy_charts::upset::Intersection;

/// An `UpSet` plot. Intersections sort largest first, sets by size. Hover or touch a column
/// to light its sets, or a set to light every intersection that includes it; focus the
/// chart and use the arrow keys to step through the columns and then the sets, Home and
/// End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn UpsetChart(
    /// Members per exclusive intersection: each entry counts the members in exactly those
    /// sets. Sets are created from the names.
    #[prop(into)]
    data: Signal<Vec<Intersection>>,
    /// What the chart shows, such as "Notification channels customers use". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Drawing width in px; the chart scales down to fit its container. The height follows the set count.
    #[prop(default = 720)]
    width: u16,
    /// Most intersections to draw; smaller ones are left out of the plot but stay in the table.
    #[prop(default = 16)]
    max_columns: usize,
    /// Formats counts for labels, the tooltip and the table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let w = f64::from(width);
    let layout = Memo::new(move |_| data.with(|d| upset::layout(d, w, max_columns)));
    let combos = Memo::new(move |_| data.with(|d| upset::index(d)));
    let total = Memo::new(move |_| combos.with(|(_, c)| c.iter().map(|x| x.1).sum::<f64>()));
    // Marks are the columns first, then the sets.
    let active = RwSignal::new(None::<usize>);
    let lit = Memo::new(move |_| active.get().map(|m| layout.with(|l| l.lit(m))));
    let col_lit = move |k: usize| lit.with(|x| x.as_ref().map(|(c, _)| c.get(k).copied().unwrap_or(false)));
    let row_lit = move |k: usize| lit.with(|x| x.as_ref().map(|(_, r)| r.get(k).copied().unwrap_or(false)));
    let flag = |b: Option<bool>, want: bool| (b == Some(want)).then_some("");
    let pick = move |m: usize| {
        if active.get_untracked() != Some(m) {
            active.set(Some(m));
        }
    };
    let tip = move || active.get().and_then(|m| layout.with(|l| l.tip(m, total.get(), &fmt)));
    let group_label = format!("{label}. Use the arrow keys to step through the intersections and then the sets.");
    let caption = label;

    view! {
        <figure data-scope="upset-chart" data-part="root" data-active=move || active.get().map(|_| "") style=format!("max-width: {width}px;")>
            <div data-part="plot">
                <svg
                    data-part="marks"
                    viewBox=move || format!("0 0 {width} {:.0}", layout.with(|l| l.height))
                    tabindex="0"
                    role="group"
                    aria-label=group_label
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        let len = layout.with(|l| l.columns.len() + l.rows.len());
                        if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), len) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    {move || {
                        let l = layout.get();
                        let n = l.columns.len();
                        let half = l.column_width / 2.0;
                        let bh = (l.row_height * 0.5).min(12.0);
                        let stripes = l
                            .rows
                            .iter()
                            .enumerate()
                            .filter(|(r, _)| r % 2 == 0)
                            .map(|(_, row)| {
                                view! { <rect data-part="stripe" x=l.label_x y=row.y - l.row_height / 2.0 width=l.width - l.label_x height=l.row_height rx="3"></rect> }
                            })
                            .collect_view();
                        let grid = l
                            .ticks
                            .iter()
                            .map(|&(v, y)| {
                                view! {
                                    <line data-part="gridline" data-baseline=(v <= 0.0).then_some("") x1=l.matrix_x x2=l.width y1=y y2=y></line>
                                    <text data-part="tick" x=l.matrix_x - 6.0 y=y text-anchor="end" dominant-baseline="middle">{fmt(v)}</text>
                                }
                            })
                            .collect_view();
                        let columns = l
                            .columns
                            .iter()
                            .enumerate()
                            .map(|(c, col)| {
                                let first = col.sets.first().map_or(0.0, |&r| l.rows[r].y);
                                let last = col.sets.last().map_or(0.0, |&r| l.rows[r].y);
                                let dots = l
                                    .rows
                                    .iter()
                                    .enumerate()
                                    .map(|(r, row)| {
                                        view! { <circle data-part="dot" data-member=col.sets.contains(&r).then_some("") cx=col.x cy=row.y r=l.dot></circle> }
                                    })
                                    .collect_view();
                                view! {
                                    <g
                                        data-part="column"
                                        data-highlighted=move || flag(col_lit(c), true)
                                        data-dim=move || flag(col_lit(c), false)
                                        data-selected=move || (active.get() == Some(c)).then_some("")
                                        style=format!("--i: {c};")
                                        on:pointerenter=move |_| pick(c)
                                        on:pointerdown=move |_| pick(c)
                                    >
                                        <rect data-part="band" x=col.x - half y="0" width=l.column_width height=l.height rx="4"></rect>
                                        <path data-part="bar" d=upset::bar_path(col.x - l.bar_width / 2.0, col.bar_y, l.bar_width, l.baseline - col.bar_y, 3.0, false)></path>
                                        {(l.column_width >= 20.0)
                                            .then(|| view! { <text data-part="bar-value" x=col.x y=col.bar_y - 5.0 text-anchor="middle">{fmt(col.count)}</text> })}
                                        {(col.sets.len() > 1).then(|| view! { <line data-part="connector" x1=col.x x2=col.x y1=first y2=last></line> })}
                                        <g data-part="dots">{dots}</g>
                                    </g>
                                }
                            })
                            .collect_view();
                        let sets = l
                            .rows
                            .iter()
                            .enumerate()
                            .map(|(r, row)| {
                                view! {
                                    <g
                                        data-part="set"
                                        data-highlighted=move || flag(row_lit(r), true)
                                        data-dim=move || flag(row_lit(r), false)
                                        data-selected=move || (active.get() == Some(n + r)).then_some("")
                                        style=format!("--i: {r};")
                                        on:pointerenter=move |_| pick(n + r)
                                        on:pointerdown=move |_| pick(n + r)
                                    >
                                        <rect data-part="band" x="0" y=row.y - l.row_height / 2.0 width=l.matrix_x height=l.row_height></rect>
                                        <path data-part="set-bar" d=upset::bar_path(row.bar_x, row.y - bh / 2.0, l.set_right - row.bar_x, bh, 3.0, true)></path>
                                        <text data-part="set-value" x=row.bar_x - 6.0 y=row.y text-anchor="end" dominant-baseline="middle">{fmt(row.size)}</text>
                                        <text data-part="set-name" x=l.label_x + 8.0 y=row.y dominant-baseline="middle">{row.name.clone()}</text>
                                    </g>
                                }
                            })
                            .collect_view();
                        view! {
                            <g data-part="stripes">{stripes}</g>
                            <g data-part="grid">{grid}</g>
                            <text data-part="axis-title" x=l.set_right y=l.baseline - 4.0 text-anchor="end">"Set size"</text>
                            <g data-part="sets">{sets}</g>
                            <g data-part="columns">{columns}</g>
                        }
                    }}
                </svg>
                {move || tip().map(|tip| view! { <SetsTip tip=tip /> })}
            </div>
            {move || {
                let hidden = layout.with(|l| l.hidden);
                (hidden > 0)
                    .then(|| {
                        let s = if hidden == 1 { "" } else { "s" };
                        view! { <p data-part="note">{format!("{hidden} smaller intersection{s} not shown")}</p> }
                    })
            }}
            <span data-part="sr-live" aria-live="polite">{move || tip().map(|t| t.announce()).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Intersection"</th>
                        <th scope="col">"Members"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let (names, mut rows) = combos.get();
                        rows.sort_by(|a, b| b.1.total_cmp(&a.1));
                        rows.into_iter()
                            .map(|(sets, count)| {
                                let names: Vec<&str> = sets.iter().map(|&s| names[s].as_str()).collect();
                                view! {
                                    <tr>
                                        <th scope="row">{upset::combo_name(&names)}</th>
                                        <td>{fmt(count)}</td>
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
