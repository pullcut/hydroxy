//! `<BlandAltmanChart>`: agreement between two measurement methods. Each pair of readings is
//! plotted as its mean against its difference, with the mean difference (bias) and the 95%
//! limits of agreement. Geometry comes from `oxide_charts::bland_altman`.

use leptos::prelude::*;
use oxide_charts::bland_altman::{self, Limit};
use oxide_charts::nav::list_key;
use oxide_charts::plot::{self, Key, Tip, TipRow, colour};

pub use oxide_charts::annotate::Annotation;

use crate::plot_kit::{Plot, formatter, placed};

/// One reference line's markup: an optional confidence band, the line, and its label.
fn limit_view(l: Limit, kind: &'static str, name: String, show_ci: bool) -> impl IntoView {
    let band = show_ci.then(|| {
        let style = format!("--a: {:.4}; --b: {:.4};", l.ci_at.0, l.ci_at.1);
        view! { <span data-part="limit-band" data-kind=kind style=style></span> }
    });
    let at = format!("--v: {:.4};", l.at);
    view! {
        {band}
        <span data-part="limit" data-kind=kind style=at.clone()></span>
        <span data-part="limit-label" data-kind=kind style=at>{name}</span>
    }
}

/// Paired readings from two methods: mean along the bottom, difference (first minus second)
/// up the side, with a solid line at the mean difference and dashed lines at the limits of
/// agreement. Hover or touch a point to read the pair; focus the plot and use Left/Right to
/// step through pairs, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn BlandAltmanChart(
    /// Readings from the first method.
    #[prop(into)]
    a: Signal<Vec<f64>>,
    /// Readings from the second method, paired with `a` by position.
    #[prop(into)]
    b: Signal<Vec<f64>>,
    /// What the chart shows, such as "Agreement of finger-prick and lab glucose". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Names the first method.
    #[prop(default = "Method A".into(), into)]
    a_label: String,
    /// Names the second method.
    #[prop(default = "Method B".into(), into)]
    b_label: String,
    /// Shade the 95% confidence interval of the bias and of each limit.
    #[prop(optional)]
    show_ci: bool,
    /// Plot height in px, excluding the axis labels.
    #[prop(default = 280)]
    height: u16,
    /// Formats readings and differences for the axes, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is a mean, y a difference).
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let layout = Memo::new(move |_| a.with(|a| b.with(|b| annotations.with(|n| bland_altman::layout(a, b, n)))));
    let count = move || layout.with(|l| l.points.len());
    let active = RwSignal::new(None::<usize>);
    let names = StoredValue::new((a_label, b_label));

    let tip = Signal::derive(move || {
        let i = active.get()?;
        let p = layout.with(|l| l.points.get(i).copied())?;
        let (an, bn) = names.get_value();
        let rows = vec![
            TipRow { active: true, ..TipRow::new(Some(colour(0)), "Difference", fmt(p.diff)) },
            TipRow::new(None, "Mean", fmt(p.mean)),
            TipRow::new(None, an, fmt(p.a)),
            TipRow::new(None, bn, fmt(p.b)),
        ];
        Some(Tip { x: p.x, y: p.y, title: format!("Pair {}", p.index + 1), rows })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        // The nearest point in the plane, so dense columns of pairs can each be reached.
        let next = p.and_then(|(x, y)| {
            layout.with(|l| {
                l.points.iter().enumerate().min_by(|(_, p), (_, q)| ((p.x - x).hypot(p.y - y)).total_cmp(&(q.x - x).hypot(q.y - y))).map(|(i, _)| i)
            })
        });
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| list_key(&key, active.get_untracked(), count()).map(|next| active.set(next)).is_some());
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(fmt)));
    let labels = Signal::derive(move || layout.with(|l| l.x_axis.ticks(fmt)));
    let notes = placed(move |n| layout.with(|l| plot::place(n, false, |x| l.x_axis.scale.map(x), |v| l.axis.scale.map(v))), annotations);
    let legend = Signal::derive(move || {
        let mut keys = vec![Key::new("Pairs", colour(0)), Key::new("Mean difference", "var(--text-2)")];
        keys.push(Key { dashed: true, ..Key::new("95% limits of agreement", "var(--text-3)") });
        keys
    });
    let head = Signal::derive(move || {
        let (an, bn) = names.get_value();
        vec!["Pair".to_string(), an, bn, "Mean".to_string(), "Difference".to_string()]
    });
    let rows = Signal::derive(move || {
        layout.with(|l| l.points.iter().map(|p| vec![(p.index + 1).to_string(), fmt(p.a), fmt(p.b), fmt(p.mean), fmt(p.diff)]).collect())
    });
    let marks = move || {
        let l = layout.get();
        let dots = l
            .points
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let style = format!("--c: {:.4}; --v: {:.4}; --i: {i};", p.x, p.y);
                view! { <span data-part="dot" data-active=move || (active.get() == Some(i)).then_some("") style=style></span> }
            })
            .collect_view();
        view! {
            <div data-part="points">{dots}</div>
            <div data-part="limits" aria-hidden="true">
                {limit_view(l.lower, "lower", format!("−1.96 SD {}", fmt(l.lower.value)), show_ci)}
                {limit_view(l.upper, "upper", format!("+1.96 SD {}", fmt(l.upper.value)), show_ci)}
                {limit_view(l.bias, "bias", format!("Mean {}", fmt(l.bias.value)), show_ci)}
            </div>
        }
    };

    view! {
        <Plot
            scope="bland-altman-chart"
            label
            hint="Use the left and right arrow keys to step through pairs."
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
        />
    }
}
