//! `SpanChart`: one bar per category from a start value to an end value, such as a salary
//! band, with an optional marker inside it. Geometry comes from `hydroxy_charts::span`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{self, Key, Tip, TipRow, colour};
use hydroxy_charts::scale::ScaleKind;
use hydroxy_charts::span;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::span::Span;

use crate::plot_kit::{Plot, formatter};

/// Categories down the left, values along the bottom, a bar from each row's start to its
/// end with both ends labelled. A marker can sit inside the range, such as the median.
/// Bars extend from their start. Hover or touch a row to read it; focus the plot and use
/// Up/Down to move between rows, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SpanChart(
    /// One label per row.
    rows: ReadSignal<Vec<String>>,
    /// One range per row: `Span::new(start, end)`, optionally `.with_marker(value)`.
    data: ReadSignal<Vec<Span>>,
    /// What the chart shows, such as "Salary bands by role". Used as the accessible name.
    label: String,
    /// What the marker means, such as "Median". Default: "Marker".
    marker_label: Option<String>,
    /// Plot height in px. Default: 36px per row.
    height: Option<u16>,
    /// Formats values for the axis, end labels, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is the value, y a row index).
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Called with the mark under the pointer or keyboard focus, and `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called with the mark when it is clicked, tapped or Enter is pressed on it.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = formatter(format);
    let marker_name = marker_label.unwrap_or_else(|| "Marker".into());
    let names = rows();
    let spans = data();
    let n = names.len();
    let notes = annotations();
    let layout = span::layout(&spans, ScaleKind::Linear, &notes);
    let height = height.unwrap_or_else(|| u16::try_from(n * 36).unwrap_or(u16::MAX).max(108));
    let marked = spans.iter().any(|s| s.marker.is_some_and(f64::is_finite));
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&r| r < n);
    let now = current.and_then(|r| {
        let (_, hi) = spans.get(r).and_then(Span::bounds)?;
        Some(EventDatum::category(r, names[r].clone(), hi))
    });
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    let tip = current.map(|r| {
        let s = spans.get(r).copied().unwrap_or(Span::new(f64::NAN, f64::NAN));
        let (lo, hi) = s.bounds().map_or((None, None), |(a, b)| (Some(a), Some(b)));
        let mut rows = vec![
            TipRow::new(Some(colour(0)), "From", plot::cell(lo, fmt)),
            TipRow::new(None, "To", plot::cell(hi, fmt)),
            TipRow::new(None, "Spread", plot::cell(lo.zip(hi).map(|(a, b)| b - a), fmt)),
        ];
        if let Some(m) = s.marker {
            rows.push(TipRow::new(Some("var(--text-1)".into()), marker_name.clone(), plot::cell(Some(m), fmt)));
        }
        let x = layout.bars.get(r).copied().flatten().map_or(0.5, |b| b.b);
        Tip { x, y: 1.0 - plot::band_centre(r, n), title: names[r].clone(), rows }
    });
    let legend: Vec<Key> = if marked {
        vec![Key::new("Range", colour(0)), Key::new(marker_name.clone(), "var(--text-1)")]
    } else {
        Vec::new()
    };
    let mut head = vec![String::new(), "From".into(), "To".into()];
    if marked {
        head.push(marker_name);
    }
    let table: Vec<Vec<String>> = names
        .iter()
        .enumerate()
        .map(|(r, name)| {
            let s = spans.get(r).copied();
            let b = s.and_then(|s| s.bounds());
            let mut row = vec![name.clone(), plot::cell(b.map(|b| b.0), fmt), plot::cell(b.map(|b| b.1), fmt)];
            if marked {
                row.push(plot::cell(s.and_then(|s| s.marker), fmt));
            }
            row
        })
        .collect();
    // (row, bar, low, high) for the rows that have a range.
    let drawn: Vec<(usize, span::SpanBar, f64, f64)> = layout
        .bars
        .iter()
        .enumerate()
        .filter_map(|(r, bar)| {
            let (lo, hi) = spans.get(r)?.bounds()?;
            Some((r, (*bar)?, lo, hi))
        })
        .collect();

    rsx! {
        Plot {
            scope: "span-chart",
            label,
            hint: "Use the up and down arrow keys to move between rows.",
            height,
            horizontal: true,
            ticks: layout.axis.ticks(fmt),
            labels: plot::category_ticks(&names, usize::MAX),
            annotations: plot::place(&notes, true, |x| plot::category_at(x, n), |v| layout.axis.scale.map(v)),
            legend,
            tip,
            datum: Some(datum),
            on_hover,
            on_select,
            head,
            rows: table,
            on_key: move |key: String| hydroxy_charts::nav::list_key(plot::turn_key(&key), active(), n).map(|next| active.set(next)).is_some(),
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(_, y)| plot::band_at(1.0 - y, n));
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                div { "data-part": "bars",
                    for (r , bar , lo , hi) in drawn {
                        span {
                            "data-part": "bar",
                            "data-active": (current == Some(r)).then_some(""),
                            style: "--c: {plot::band_centre(r, n):.4}; --w: 0; --a: {bar.a:.4}; --b: {bar.b:.4}; --i: {r}; background: {colour(0)};",
                        }
                        if let Some(m) = bar.marker {
                            span {
                                "data-part": "marker",
                                "data-active": (current == Some(r)).then_some(""),
                                style: "--c: {plot::band_centre(r, n):.4}; --v: {m:.4}; --i: {r};",
                            }
                        }
                        span {
                            "data-part": "end-label",
                            "data-side": "start",
                            style: "--c: {plot::band_centre(r, n):.4}; --v: {bar.a:.4}; --i: {r};",
                            {fmt(lo)}
                        }
                        span {
                            "data-part": "end-label",
                            "data-side": "end",
                            style: "--c: {plot::band_centre(r, n):.4}; --v: {bar.b:.4}; --i: {r};",
                            {fmt(hi)}
                        }
                    }
                }
            },
        }
    }
}
