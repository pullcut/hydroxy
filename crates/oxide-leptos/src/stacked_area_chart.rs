//! `<StackedAreaChart>`: series stacked over a shared x axis, from zero, as shares of 100%,
//! or as a streamgraph. Geometry comes from `oxide_charts::stacked_area`.

use leptos::prelude::*;
use oxide_charts::line::nearest_index;
use oxide_charts::nav::list_key;
use oxide_charts::plot::{self, Key, Tip, TipRow, colour};
use oxide_charts::stacked_area;

pub use oxide_charts::annotate::Annotation;
pub use oxide_charts::line::{Curve, Series};
pub use oxide_charts::stacked_area::StackOffset;

use crate::plot_kit::{Plot, formatter, pct, placed};

fn share(f: f64) -> String {
    format!("{:.0}%", f * 100.0)
}

/// Series stacked on top of each other so the total and each part read together. Hover or
/// touch to read every series at that x; focus the plot and use Left/Right to step,
/// Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn StackedAreaChart(
    /// The series, bottom of the stack first, one value per label. Past six, the smallest
    /// fold into "Other". Missing, negative or NaN values count as zero.
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// One label per x position, used on the axis, tooltip and table.
    #[prop(into)]
    labels: Signal<Vec<String>>,
    /// What the chart shows, such as "Traffic by source". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[prop(default = 240)]
    height: u16,
    /// Zero (default), Expand (100%), Wiggle (streamgraph) or Silhouette (centred).
    #[prop(optional)]
    offset: StackOffset,
    /// Straight segments (default) or a monotone curve.
    #[prop(optional)]
    curve: Curve,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points (x is a point index). Values are read on
    /// the stacked axis; with Expand they are shares, 0..=1.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// A value range the axis must cover, such as the shared domain of small multiples.
    #[prop(optional, into)]
    y_domain: MaybeProp<(f64, f64)>,
) -> impl IntoView {
    let fmt = formatter(format);
    let series = Memo::new(move |_| series.with(|s| plot::fold_series(s, 6)));
    let count = move || labels.with(Vec::len);
    // The shared domain only widens the axis: it reaches the layout as an unlabelled, undrawn band.
    let layout = Memo::new(move |_| {
        let mut a = annotations.get();
        a.extend(y_domain.get().map(|(lo, hi)| Annotation::y_band(lo, hi, "")));
        series.with(|s| stacked_area::layout(s, count(), offset, curve, &a))
    });
    let active = RwSignal::new(None::<usize>);
    // The series under the pointer, if any.
    let layer = RwSignal::new(None::<usize>);

    let tip = Signal::derive(move || {
        let i = active.get()?;
        let title = labels.with(|l| l.get(i).cloned())?;
        let (x, y, total) = layout.with(|l| (l.xs[i], l.tops.last().map_or(0.0, |t| t[i]), l.totals[i]));
        let hovered = layer.get();
        let mut rows: Vec<TipRow> = series.with(|all| {
            all.iter()
                .enumerate()
                .rev()
                .map(|(s, x)| {
                    let v = x.values.get(i).copied();
                    let text = match (offset, v) {
                        (StackOffset::Expand, Some(v)) if v.is_finite() && total > 0.0 => format!("{} ({})", fmt(v), share(v.max(0.0) / total)),
                        _ => plot::cell(v, fmt),
                    };
                    TipRow { active: hovered == Some(s), ..TipRow::new(Some(colour(s)), x.name.clone(), text) }
                })
                .collect()
        });
        rows.push(TipRow::new(None, "Total", fmt(total)));
        Some(Tip { x, y, title, rows })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, _)| layout.with(|l| nearest_index(&l.xs, x)));
        let over = p.zip(next).and_then(|((_, y), i)| layout.with(|l| l.tops.iter().position(|t| t[i] >= y)));
        if active.get_untracked() != next {
            active.set(next);
        }
        if layer.get_untracked() != over {
            layer.set(over);
        }
    });
    let on_key = Callback::new(move |key: String| {
        layer.set(None);
        list_key(&key, active.get_untracked(), count()).map(|next| active.set(next)).is_some()
    });
    let ticks = Signal::derive(move || layout.with(|l| if offset == StackOffset::Expand { l.axis.ticks(share) } else { l.axis.ticks(fmt) }));
    let axis_labels = Signal::derive(move || labels.with(|n| layout.with(|l| plot::point_ticks(n, &l.xs, 8))));
    let notes = placed(move |a| layout.with(|l| plot::place(a, false, |x| plot::index_at(x, count()), |v| l.axis.scale.map(v))), annotations);
    let legend = Signal::derive(move || series.with(|s| if s.len() > 1 { s.iter().enumerate().map(|(i, s)| Key::new(s.name.clone(), colour(i))).collect() } else { Vec::new() }));
    let head = Signal::derive(move || {
        let mut h = vec![String::new()];
        series.with(|s| h.extend(s.iter().map(|s| s.name.clone())));
        h.push("Total".into());
        h
    });
    let rows = Signal::derive(move || {
        labels.with(|n| {
            series.with(|s| {
                layout.with(|l| {
                    n.iter()
                        .enumerate()
                        .map(|(i, name)| {
                            let mut r = vec![name.clone()];
                            r.extend(s.iter().map(|s| plot::cell(s.values.get(i).copied(), fmt)));
                            r.push(fmt(l.totals[i]));
                            r
                        })
                        .collect()
                })
            })
        })
    });
    let marks = move || {
        let l = layout.get();
        let areas = l
            .areas
            .into_iter()
            .enumerate()
            .map(|(i, d)| view! { <path data-part="area" data-active=move || (layer.get() == Some(i)).then_some("") d=d fill=colour(i)></path> })
            .collect_view();
        view! {
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                {areas}
            </svg>
        }
    };
    let overlay = move || {
        active.get().map(|i| {
            let l = layout.get();
            let x = l.xs[i];
            let dots = l
                .tops
                .iter()
                .enumerate()
                .map(|(s, t)| view! { <span data-part="dot" style=format!("--c: {x:.4}; --v: {:.4}; background: {};", t[i], colour(s))></span> })
                .collect_view();
            view! {
                <div data-part="crosshair" style=format!("--p: {};", pct(x))></div>
                {dots}
            }
        })
    };

    view! {
        <Plot
            scope="stacked-area-chart"
            label
            hint="Use the left and right arrow keys to step through."
            height
            variant=offset.attr()
            ticks
            labels=axis_labels
            annotations=notes
            legend
            tip
            head
            rows
            on_key
            on_point
            marks
            overlay
        />
    }
}
