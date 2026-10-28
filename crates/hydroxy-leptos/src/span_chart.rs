//! `<SpanChart>`: one bar per category from a start value to an end value, such as a
//! salary band, with an optional marker inside it. Geometry comes from `hydroxy_charts::span`.

use leptos::prelude::*;
use hydroxy_charts::plot::{self, Key, Tip, TipRow, colour};
use hydroxy_charts::scale::ScaleKind;
use hydroxy_charts::span;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::span::Span;

use crate::plot_kit::{Plot, formatter, placed};

/// Categories down the left, values along the bottom, a bar from each row's start to its
/// end with both ends labelled. A marker can sit inside the range, such as the median.
/// Bars extend from their start. Hover or touch a row to read it; focus the plot and use
/// Up/Down to move between rows, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SpanChart(
    /// One label per row.
    #[prop(into)]
    rows: Signal<Vec<String>>,
    /// One range per row: `Span::new(start, end)`, optionally `.with_marker(value)`.
    #[prop(into)]
    data: Signal<Vec<Span>>,
    /// What the chart shows, such as "Salary bands by role". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// What the marker means, such as "Median". Default: "Marker".
    #[prop(optional, into)]
    marker_label: Option<String>,
    /// Plot height in px. Default: 36px per row.
    #[prop(optional)]
    height: Option<u16>,
    /// Formats values for the axis, end labels, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is the value, y a row index).
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let marker_name = marker_label.unwrap_or_else(|| "Marker".into());
    let count = move || rows.with(Vec::len);
    let height = height.unwrap_or_else(|| u16::try_from(rows.with_untracked(Vec::len) * 36).unwrap_or(u16::MAX).max(108));
    let layout = Memo::new(move |_| data.with(|d| annotations.with(|a| span::layout(d, ScaleKind::Linear, a))));
    let marked = Memo::new(move |_| data.with(|d| d.iter().any(|s| s.marker.is_some_and(f64::is_finite))));
    let active = RwSignal::new(None::<usize>);
    let tip_marker = marker_name.clone();
    let tip = Signal::derive(move || {
        let r = active.get()?;
        let title = rows.with(|n| n.get(r).cloned())?;
        let s = data.with(|d| d.get(r).copied())?;
        let bar = layout.with(|l| l.bars.get(r).copied().flatten());
        let (lo, hi) = s.bounds().map_or((None, None), |(a, b)| (Some(a), Some(b)));
        let mut rows = vec![
            TipRow::new(Some(colour(0)), "From", plot::cell(lo, fmt)),
            TipRow::new(None, "To", plot::cell(hi, fmt)),
            TipRow::new(None, "Spread", plot::cell(lo.zip(hi).map(|(a, b)| b - a), fmt)),
        ];
        if let Some(m) = s.marker {
            rows.push(TipRow::new(Some("var(--text-1)".into()), tip_marker.clone(), plot::cell(Some(m), fmt)));
        }
        Some(Tip { x: bar.map_or(0.5, |b| b.b), y: 1.0 - plot::band_centre(r, count()), title, rows })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(_, y)| plot::band_at(1.0 - y, count()));
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| {
        hydroxy_charts::nav::list_key(plot::turn_key(&key), active.get_untracked(), count()).map(|next| active.set(next)).is_some()
    });
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(fmt)));
    let labels = Signal::derive(move || rows.with(|n| plot::category_ticks(n, usize::MAX)));
    let notes = placed(move |a| layout.with(|l| plot::place(a, true, |x| plot::category_at(x, count()), |v| l.axis.scale.map(v))), annotations);
    let legend_marker = marker_name.clone();
    let legend = Signal::derive(move || {
        if marked.get() { vec![Key::new("Range", colour(0)), Key::new(legend_marker.clone(), "var(--text-1)")] } else { Vec::new() }
    });
    let head_marker = marker_name;
    let head = Signal::derive(move || {
        let mut h = vec![String::new(), "From".into(), "To".into()];
        if marked.get() {
            h.push(head_marker.clone());
        }
        h
    });
    let table = Signal::derive(move || {
        rows.with(|names| {
            data.with(|d| {
                names
                    .iter()
                    .enumerate()
                    .map(|(r, name)| {
                        let s = d.get(r).copied();
                        let b = s.and_then(|s| s.bounds());
                        let mut row = vec![name.clone(), plot::cell(b.map(|b| b.0), fmt), plot::cell(b.map(|b| b.1), fmt)];
                        if marked.get() {
                            row.push(plot::cell(s.and_then(|s| s.marker), fmt));
                        }
                        row
                    })
                    .collect()
            })
        })
    });
    let marks = move || {
        let l = layout.get();
        let n = l.bars.len();
        let bars = l
            .bars
            .iter()
            .enumerate()
            .filter_map(|(r, bar)| {
                let bar = (*bar)?;
                let (lo, hi) = data.with(|d| d.get(r).and_then(Span::bounds))?;
                let c = plot::band_centre(r, n);
                let is_active = move || (active.get() == Some(r)).then_some("");
                let dot = bar.marker.map(|m| {
                    view! { <span data-part="marker" data-active=is_active style=format!("--c: {c:.4}; --v: {m:.4}; --i: {r};")></span> }
                });
                Some(view! {
                    <span
                        data-part="bar"
                        data-active=is_active
                        style=format!("--c: {c:.4}; --w: 0; --a: {:.4}; --b: {:.4}; --i: {r}; background: {};", bar.a, bar.b, colour(0))
                    ></span>
                    {dot}
                    <span data-part="end-label" data-side="start" style=format!("--c: {c:.4}; --v: {:.4}; --i: {r};", bar.a)>{fmt(lo)}</span>
                    <span data-part="end-label" data-side="end" style=format!("--c: {c:.4}; --v: {:.4}; --i: {r};", bar.b)>{fmt(hi)}</span>
                })
            })
            .collect_view();
        view! { <div data-part="bars">{bars}</div> }
    };

    view! {
        <Plot
            scope="span-chart"
            label
            hint="Use the up and down arrow keys to move between rows."
            height
            horizontal=true
            ticks
            labels
            annotations=notes
            legend
            tip
            head
            rows=table
            on_key
            on_point
            marks
        />
    }
}
