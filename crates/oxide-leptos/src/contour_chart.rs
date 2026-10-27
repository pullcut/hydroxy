//! `<ContourChart>` and `<HeatField>`: a scalar field on a grid, as contour lines
//! (marching squares) and/or a filled heat field. Geometry comes from `oxide_charts::field`.

use leptos::prelude::*;
use oxide_charts::{field, format_value, nav};

/// Contour lines over a grid of values, optionally on a filled heat field. Values are
/// row-major, `columns` per row; NaN leaves a gap. Hover or touch to read a cell; focus
/// the field and use the arrow keys to move between cells, Home and End to jump along
/// the row, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::cast_precision_loss, reason = "grid indices are small")]
#[component]
pub fn ContourChart(
    /// Grid values, row-major.
    #[prop(into)]
    values: Signal<Vec<f64>>,
    /// Values per row.
    columns: usize,
    /// What the chart shows. Used as the accessible name.
    #[prop(into)]
    label: String,
    /// About how many contour levels to draw.
    #[prop(default = 6)]
    levels: usize,
    /// Shade each cell by its value underneath the lines.
    #[prop(optional)]
    filled: bool,
    /// Formats values on the legend, tooltips and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let show = move |v: Option<f64>| v.filter(|v| v.is_finite()).map_or_else(|| "no data".to_string(), fmt);
    let cols = columns.max(1);
    let layout = Memo::new(move |_| values.with(|v| field::layout(v, columns, levels)));
    let rows = Memo::new(move |_| values.with(Vec::len).div_ceil(cols));
    let active = RwSignal::new(None::<(usize, usize)>);
    let value_at = move |(r, c): (usize, usize)| values.with(|v| v.get(r * cols + c).copied());
    let describe = move |at: (usize, usize)| format!("Row {}, column {}: {}", at.0 + 1, at.1 + 1, show(value_at(at)));
    let pick = move |ev: leptos::ev::PointerEvent| {
        let Some(el) = ev.current_target().and_then(|t| leptos::wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(t).ok()) else {
            return;
        };
        let fx = f64::from(ev.offset_x()) / f64::from(el.offset_width().max(1));
        let fy = f64::from(ev.offset_y()) / f64::from(el.offset_height().max(1));
        let next = field::cell_at(fx, fy, rows.get_untracked(), cols);
        if active.get_untracked() != next {
            active.set(next);
        }
    };
    let group_label = format!("{label}. Use the arrow keys to move between cells.");
    let caption = label;

    view! {
        <figure data-scope="contour-chart" data-part="root" data-filled=filled.then_some("")>
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=group_label
                on:pointermove=pick
                on:pointerdown=pick
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if let Some(next) = nav::grid_key(&ev.key(), active.get_untracked(), rows.get_untracked(), cols) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                {move || {
                    let l = layout.get();
                    let cells = filled.then(|| {
                        l.heat
                            .iter()
                            .enumerate()
                            .filter_map(|(i, t)| {
                                t.map(|t| {
                                    let (c, r) = (i % cols, i / cols);
                                    view! {
                                        <rect
                                            data-part="cell"
                                            x=c as f64 * l.cell
                                            y=r as f64 * l.cell
                                            width=l.cell
                                            height=l.cell
                                            style=format!("--t: {t:.3};")
                                        ></rect>
                                    }
                                })
                            })
                            .collect_view()
                    });
                    let span = (l.range.1 - l.range.0).max(f64::EPSILON);
                    let lines = l
                        .contours
                        .iter()
                        .map(|c| {
                            view! {
                                <path
                                    data-part="contour"
                                    d=c.path.clone()
                                    pathLength="1"
                                    style=format!("--t: {:.3};", (c.level - l.range.0) / span)
                                ></path>
                            }
                        })
                        .collect_view();
                    let cell = l.cell;
                    view! {
                        <svg data-part="marks" viewBox=format!("0 0 {} {}", l.width, l.height) aria-hidden="true">
                            {cells}
                            {lines}
                            {move || {
                                active
                                    .get()
                                    .map(|(r, c)| {
                                        view! {
                                            <rect data-part="active-cell" x=c as f64 * cell y=r as f64 * cell width=cell height=cell></rect>
                                        }
                                    })
                            }}
                        </svg>
                    }
                }}
                {move || {
                    let (r, c) = active.get()?;
                    let n_rows = rows.get().max(1);
                    let x = (c as f64 + 0.5) / cols as f64;
                    let y = (r as f64 + 0.5) / n_rows as f64;
                    Some(
                        view! {
                            <div
                                data-part="tooltip"
                                data-anchored=""
                                data-side={if x > 0.5 { "left" } else { "right" }}
                                style=format!("left: {:.3}%; top: {:.3}%;", x * 100.0, y * 100.0)
                                aria-hidden="true"
                            >
                                <div data-part="tooltip-label">{format!("Row {}, column {}", r + 1, c + 1)}</div>
                                <ul>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Value"</span>
                                        <span data-part="tooltip-value">{show(value_at((r, c)))}</span>
                                    </li>
                                </ul>
                            </div>
                        },
                    )
                }}
            </div>
            <div data-part="scale" aria-hidden="true">
                <span>{move || fmt(layout.with(|l| l.range.0))}</span>
                <span data-part="ramp"></span>
                <span>{move || fmt(layout.with(|l| l.range.1))}</span>
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().map(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Row"</th>
                        {(1..=cols).map(|c| view! { <th scope="col">{format!("Column {c}")}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let v = values.get();
                        v.chunks(cols)
                            .enumerate()
                            .map(|(r, row)| {
                                let cells = row.iter().map(|&x| view! { <td>{if x.is_finite() { fmt(x) } else { "–".to_string() }}</td> }).collect_view();
                                view! { <tr><th scope="row">{format!("Row {}", r + 1)}</th>{cells}</tr> }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}

/// A filled heat field with no contour lines: a smooth-ish picture of a grid of values.
/// Reads and navigates like [`ContourChart`].
#[component]
pub fn HeatField(
    /// Grid values, row-major.
    #[prop(into)]
    values: Signal<Vec<f64>>,
    /// Values per row.
    columns: usize,
    /// What the chart shows. Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Formats values on the legend, tooltips and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    view! { <ContourChart values columns label levels=0 filled=true format=format.unwrap_or_else(|| Callback::new(format_value)) /> }
}
