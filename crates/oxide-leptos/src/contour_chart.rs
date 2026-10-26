//! `<ContourChart>` and `<HeatField>`: a scalar field on a grid, as contour lines
//! (marching squares) and/or a filled heat field. Geometry comes from `oxide_charts::field`.

use leptos::prelude::*;
use oxide_charts::{field, format_value};

#[allow(clippy::cast_precision_loss, reason = "grid indices are small")]
/// Contour lines over a grid of values, optionally on a filled heat field. Values are
/// row-major, `columns` per row; NaN leaves a gap.
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
    let layout = Memo::new(move |_| values.with(|v| field::layout(v, columns, levels)));
    let aria = label.clone();
    view! {
        <figure data-scope="contour-chart" data-part="root" data-filled=filled.then_some("")>
            {move || {
                let l = layout.get();
                let vals = values.get();
                let cells = filled.then(|| {
                    l.heat
                        .iter()
                        .enumerate()
                        .filter_map(|(i, t)| {
                            t.map(|t| {
                                let (c, r) = (i % columns.max(1), i / columns.max(1));
                                view! {
                                    <rect
                                        data-part="cell"
                                        x=c as f64 * l.cell
                                        y=r as f64 * l.cell
                                        width=l.cell
                                        height=l.cell
                                        style=format!("--t: {t:.3};")
                                    >
                                        <title>{fmt(vals[i])}</title>
                                    </rect>
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
                            <path data-part="contour" d=c.path.clone() style=format!("--t: {:.3};", (c.level - l.range.0) / span)>
                                <title>{fmt(c.level)}</title>
                            </path>
                        }
                    })
                    .collect_view();
                view! {
                    <svg data-part="marks" viewBox=format!("0 0 {} {}", l.width, l.height) role="img" aria-label=aria.clone()>
                        {cells}
                        {lines}
                    </svg>
                }
            }}
            <div data-part="scale" aria-hidden="true">
                <span>{move || fmt(layout.with(|l| l.range.0))}</span>
                <span data-part="ramp"></span>
                <span>{move || fmt(layout.with(|l| l.range.1))}</span>
            </div>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <tbody>
                    {move || {
                        let v = values.get();
                        v.chunks(columns.max(1))
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
