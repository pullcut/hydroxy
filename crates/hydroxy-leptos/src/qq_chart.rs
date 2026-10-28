//! `<QQChart>`: a normal quantile–quantile plot. Each sorted sample value is plotted against
//! the normal quantile it would have if the sample were normal; points along the dashed
//! reference line mean the data is close to normal. Geometry comes from `hydroxy_charts::qq`.

use leptos::prelude::*;
use hydroxy_charts::line::nearest_index;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Tick, Tip, TipRow, colour};
use hydroxy_charts::scale::Linear;
use hydroxy_charts::{format_value, qq};

pub use hydroxy_charts::annotate::Annotation;

use crate::plot_kit::{Plot, formatter, placed};

/// Sample values (up the side) against normal quantiles in standard deviations (along the
/// bottom). Hover or touch a point to read it; focus the plot and use Left/Right to step
/// from smallest to largest, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn QQChart(
    /// The sample, in any order. NaN values are dropped.
    #[prop(into)]
    sample: Signal<Vec<f64>>,
    /// What the chart shows, such as "Checkout latency residuals". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// What the sample measures, for the tooltip and table.
    #[prop(default = "Value".into(), into)]
    value_label: String,
    /// Plot height in px, excluding the axis labels.
    #[prop(default = 260)]
    height: u16,
    /// Formats sample values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points. `x` is a normal quantile (standard
    /// deviations), `y` a sample value.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let layout = Memo::new(move |_| sample.with(|s| annotations.with(|a| qq::layout(s, a))));
    let count = move || layout.with(|l| l.points.len());
    let xs = Memo::new(move |_| layout.with(|l| l.points.iter().map(|p| p.x).collect::<Vec<f64>>()));
    let active = RwSignal::new(None::<usize>);
    let name = StoredValue::new(value_label);

    let tip = Signal::derive(move || {
        let i = active.get()?;
        let p = layout.with(|l| l.points.get(i).copied())?;
        let rows = vec![
            TipRow { active: true, ..TipRow::new(Some(colour(0)), name.get_value(), fmt(p.sample)) },
            TipRow::new(None, "Normal quantile", format!("{:.2}σ", p.theoretical)),
        ];
        Some(Tip { x: p.x, y: p.y, title: format!("{} of {}", i + 1, count()), rows })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, _)| xs.with(|xs| nearest_index(xs, x)));
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| list_key(&key, active.get_untracked(), count()).map(|next| active.set(next)).is_some());
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(fmt)));
    let labels = Signal::derive(move || {
        layout.with(|l| l.x_ticks.iter().map(|g| Tick { at: g.fraction, text: format!("{}σ", format_value(g.value)), baseline: false }).collect::<Vec<_>>())
    });
    let notes = placed(
        move |a| {
            layout.with(|l| {
                let sx = Linear::new(l.x_domain, (0.0, 1.0));
                plot::place(a, false, |x| sx.map(x), |v| l.axis.scale.map(v))
            })
        },
        annotations,
    );
    let head = Signal::derive(move || vec!["Rank".to_string(), "Normal quantile".to_string(), name.get_value()]);
    let rows = Signal::derive(move || {
        layout.with(|l| l.points.iter().enumerate().map(|(i, p)| vec![(i + 1).to_string(), format!("{:.2}", p.theoretical), fmt(p.sample)]).collect())
    });
    let marks = move || {
        let l = layout.get();
        let dots = l
            .points
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let style = format!("--c: {:.4}; --v: {:.4}; --i: {i}; background: {};", p.x, p.y, colour(0));
                view! { <span data-part="dot" data-active=move || (active.get() == Some(i)).then_some("") style=style></span> }
            })
            .collect_view();
        view! {
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                <path data-part="reference" d=l.reference></path>
            </svg>
            <div data-part="points">{dots}</div>
        }
    };

    view! {
        <Plot
            scope="qq-chart"
            label
            hint="Use the left and right arrow keys to step from smallest to largest."
            height
            ticks
            labels
            annotations=notes
            tip
            head
            rows
            on_key
            on_point
            marks
        />
    }
}
