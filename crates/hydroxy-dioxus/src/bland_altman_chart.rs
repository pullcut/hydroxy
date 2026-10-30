//! `BlandAltmanChart`: agreement between two measurement methods. Each pair of readings is
//! plotted as its mean against its difference, with the mean difference (bias) and the 95%
//! limits of agreement. Geometry comes from `hydroxy_charts::bland_altman`.

use dioxus::prelude::*;
use hydroxy_charts::bland_altman::{self, Limit};
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow, colour};

pub use hydroxy_charts::annotate::Annotation;

use crate::plot_kit::{Plot, formatter};

/// One reference line's markup: an optional confidence band, the line, and its label.
fn limit_view(l: Limit, kind: &'static str, name: &str, show_ci: bool) -> Element {
    rsx! {
        if show_ci {
            span { "data-part": "limit-band", "data-kind": kind, style: "--a: {l.ci_at.0:.4}; --b: {l.ci_at.1:.4};" }
        }
        span { "data-part": "limit", "data-kind": kind, style: "--v: {l.at:.4};" }
        span { "data-part": "limit-label", "data-kind": kind, style: "--v: {l.at:.4};", "{name}" }
    }
}

/// Paired readings from two methods: mean along the bottom, difference (first minus second)
/// up the side, with a solid line at the mean difference and dashed lines at the limits of
/// agreement. Hover or touch a point to read the pair; focus the plot and use Left/Right to
/// step through pairs, Home/End to jump, Escape to clear.
#[component]
pub fn BlandAltmanChart(
    /// Readings from the first method.
    a: ReadSignal<Vec<f64>>,
    /// Readings from the second method, paired with `a` by position.
    b: ReadSignal<Vec<f64>>,
    /// What the chart shows, such as "Agreement of finger-prick and lab glucose". Used as the accessible name.
    label: String,
    /// Names the first method.
    #[props(default = "Method A".to_string())]
    a_label: String,
    /// Names the second method.
    #[props(default = "Method B".to_string())]
    b_label: String,
    /// Shade the 95% confidence interval of the bias and of each limit.
    #[props(default)]
    show_ci: bool,
    /// Plot height in px, excluding the axis labels.
    #[props(default = 280)]
    height: u16,
    /// Formats readings and differences for the axes, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is a mean, y a difference).
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = formatter(format);
    let notes = annotations();
    let layout = bland_altman::layout(&a(), &b(), &notes);
    let n = layout.points.len();
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let cur = if on_hover.is_some() || on_select.is_some() {
        current.map(|i| EventDatum::category(i, format!("Pair {}", layout.points[i].index + 1), layout.points[i].diff))
    } else {
        None
    };
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&cur,), move |(cur,)| cur)).into();

    let tip = current.map(|i| {
        let p = layout.points[i];
        let rows = vec![
            TipRow { active: true, ..TipRow::new(Some(colour(0)), "Difference", fmt(p.diff)) },
            TipRow::new(None, "Mean", fmt(p.mean)),
            TipRow::new(None, a_label.clone(), fmt(p.a)),
            TipRow::new(None, b_label.clone(), fmt(p.b)),
        ];
        Tip { x: p.x, y: p.y, title: format!("Pair {}", p.index + 1), rows }
    });
    let legend = vec![
        Key::new("Pairs", colour(0)),
        Key::new("Mean difference", "var(--text-2)"),
        Key { dashed: true, ..Key::new("95% limits of agreement", "var(--text-3)") },
    ];
    let rows: Vec<Vec<String>> = layout
        .points
        .iter()
        .map(|p| vec![(p.index + 1).to_string(), fmt(p.a), fmt(p.b), fmt(p.mean), fmt(p.diff)])
        .collect();
    let spots: Vec<(f64, f64)> = layout.points.iter().map(|p| (p.x, p.y)).collect();

    rsx! {
        Plot {
            scope: "bland-altman-chart",
            label,
            hint: "Use the left and right arrow keys to step through pairs.",
            height,
            ticks: layout.axis.ticks(&fmt),
            labels: layout.x_axis.ticks(&fmt),
            annotations: plot::place(&notes, false, |x| layout.x_axis.scale.map(x), |v| layout.axis.scale.map(v)),
            legend,
            tip,
            datum: Some(datum),
            on_hover,
            on_select,
            head: vec!["Pair".to_string(), a_label, b_label, "Mean".to_string(), "Difference".to_string()],
            rows,
            on_key: move |key: String| list_key(&key, active(), n).map(|next| active.set(next)).is_some(),
            on_point: move |p: Option<(f64, f64)>| {
                // The nearest point in the plane, so dense columns of pairs can each be reached.
                let next = p.and_then(|(x, y)| spots.iter().enumerate().min_by(|(_, p), (_, q)| (p.0 - x).hypot(p.1 - y).total_cmp(&(q.0 - x).hypot(q.1 - y))).map(|(i, _)| i));
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                div { "data-part": "points", style: "--ox-n: {layout.points.len()};",
                    for (i , p) in layout.points.iter().enumerate() {
                        span {
                            key: "{i}",
                            "data-part": "dot",
                            "data-active": (current == Some(i)).then_some(""),
                            style: "--c: {p.x:.4}; --v: {p.y:.4}; --i: {i};",
                        }
                    }
                }
                div { "data-part": "limits", "aria-hidden": "true",
                    {limit_view(layout.lower, "lower", &format!("−1.96 SD {}", fmt(layout.lower.value)), show_ci)}
                    {limit_view(layout.upper, "upper", &format!("+1.96 SD {}", fmt(layout.upper.value)), show_ci)}
                    {limit_view(layout.bias, "bias", &format!("Mean {}", fmt(layout.bias.value)), show_ci)}
                }
            },
        }
    }
}
