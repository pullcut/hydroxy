//! `<EcdfChart>`: empirical cumulative distributions. Each group's step line rises by 1/n at
//! every value, so the height at any x is the share of values at or below it. Geometry comes
//! from `hydroxy_charts::ecdf`.

use hydroxy_charts::ecdf::{self, ecdf_at};
use hydroxy_charts::plot::{self, Key, Tick, Tip, TipRow, colour};
use leptos::prelude::*;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::ecdf::Sample;

use crate::plot_kit::{Plot, formatter, pct, placed};
use crate::rank_kit::index_handlers;

/// A cumulative share as a whole percentage, such as "72%".
fn share(p: f64) -> String {
    format!("{:.0}%", p * 100.0)
}

/// Step lines of the share of values at or below each x, one per group (up to six), with an
/// optional dashed normal fit for each. Hover or touch to read every group's percentile at
/// that value; focus the plot and use Left/Right to step through values, Home/End to jump,
/// Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn EcdfChart(
    /// The groups: `Sample::new(name, values)`. Up to six; NaN values are dropped.
    #[prop(into)]
    data: Signal<Vec<Sample>>,
    /// What the chart shows, such as "Page load time by region". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Draw each group's fitted normal CDF as a dashed line, to judge normality.
    #[prop(optional)]
    reference: bool,
    /// Plot height in px, excluding the axis labels and legend.
    #[prop(default = 260)]
    height: u16,
    /// Formats values for the x axis, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points. `x` is a value, `y` a cumulative share (0..=1).
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let layout = Memo::new(move |_| data.with(|d| annotations.with(|a| ecdf::layout(d, a))));
    let positions =
        Memo::new(move |_| layout.with(|l| l.xs.iter().map(|&x| l.x_axis.scale.map(x)).collect::<Vec<f64>>()));
    let count = move || layout.with(|l| l.xs.len());
    let active = RwSignal::new(None::<usize>);

    let tip = Signal::derive(move || {
        let i = active.get()?;
        layout.with(|l| {
            let x = *l.xs.get(i)?;
            let rows = l
                .curves
                .iter()
                .enumerate()
                .map(|(g, c)| {
                    let mut row = TipRow::new(Some(colour(g)), c.name.clone(), share(ecdf_at(&c.sorted, x)));
                    row.active = l.curves.len() == 1;
                    row
                })
                .collect();
            let y = l.curves.iter().map(|c| ecdf_at(&c.sorted, x)).fold(0.0, f64::max);
            Some(Tip { x: l.x_axis.scale.map(x), y, title: format!("At or below {}", fmt(x)), rows })
        })
    });
    let (on_point, on_key) = index_handlers(active, positions.into(), Signal::derive(count));
    let ticks = Signal::derive(move || {
        layout.with(|l| {
            l.axis
                .gridlines
                .iter()
                .map(|g| Tick { at: g.fraction, text: share(g.value), baseline: g.value == 0.0 })
                .collect::<Vec<_>>()
        })
    });
    let labels = Signal::derive(move || layout.with(|l| l.x_axis.ticks(fmt)));
    let notes = placed(
        move |a| layout.with(|l| plot::place(a, false, |x| l.x_axis.scale.map(x), |v| l.axis.scale.map(v))),
        annotations,
    );
    let legend = Signal::derive(move || {
        layout.with(|l| {
            let mut keys: Vec<Key> =
                l.curves.iter().enumerate().map(|(g, c)| Key::new(c.name.clone(), colour(g))).collect();
            if reference {
                keys.push(Key { dashed: true, ..Key::new("Normal fit", "var(--text-3)") });
            }
            if keys.len() == 1 { Vec::new() } else { keys }
        })
    });
    let head = Signal::derive(|| {
        [
            "Group",
            "n",
            "Mean",
            "SD",
            "10th percentile",
            "25th percentile",
            "Median",
            "75th percentile",
            "90th percentile",
        ]
        .map(String::from)
        .to_vec()
    });
    let rows = Signal::derive(move || {
        layout.with(|l| {
            l.curves
                .iter()
                .map(|c| {
                    let q = |p: f64| {
                        // The smallest value whose ECDF reaches p.
                        #[allow(
                            clippy::cast_precision_loss,
                            clippy::cast_possible_truncation,
                            clippy::cast_sign_loss,
                            reason = "sample sizes are small"
                        )]
                        let k = ((p * c.sorted.len() as f64).ceil() as usize).clamp(1, c.sorted.len().max(1)) - 1;
                        plot::cell(c.sorted.get(k).copied(), fmt)
                    };
                    vec![
                        c.name.clone(),
                        c.sorted.len().to_string(),
                        fmt(c.mean),
                        fmt(c.sd),
                        q(0.1),
                        q(0.25),
                        q(0.5),
                        q(0.75),
                        q(0.9),
                    ]
                })
                .collect()
        })
    });
    let marks = move || {
        let l = layout.get();
        let lines = l
            .curves
            .iter()
            .enumerate()
            .map(|(g, c)| {
                let fit = (reference && !c.reference.is_empty()).then(|| view! { <path data-part="reference" d=c.reference.clone() style=format!("stroke: {};", colour(g))></path> });
                view! {
                    {fit}
                    <path data-part="line" d=c.line.clone() stroke=colour(g) style=format!("--i: {g};")></path>
                }
            })
            .collect_view();
        view! { <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">{lines}</svg> }
    };
    let overlay = move || {
        let i = active.get()?;
        layout.with(|l| {
            let x = *l.xs.get(i)?;
            let fx = l.x_axis.scale.map(x);
            let dots = l
                .curves
                .iter()
                .enumerate()
                .filter(|(_, c)| !c.sorted.is_empty())
                .map(|(g, c)| {
                    let style = format!("--c: {fx:.4}; --v: {:.4}; background: {};", ecdf_at(&c.sorted, x), colour(g));
                    view! { <span data-part="dot" style=style></span> }
                })
                .collect_view();
            Some(view! {
                <div data-part="crosshair" style=format!("--p: {};", pct(fx))></div>
                {dots}
            })
        })
    };

    view! {
        <Plot
            scope="ecdf-chart"
            label
            hint="Use the left and right arrow keys to step through values."
            height
            ticks
            labels
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
