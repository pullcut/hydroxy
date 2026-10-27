//! `<MetricCard>`: one KPI with its change against the previous value and an optional
//! trend sparkline. The change is shown by sign, arrow and colour together, never
//! colour alone. Maths from `oxide_charts::metric` and `oxide_charts::sparkline`.

use crate::icon::{Icon, IconName};
use leptos::prelude::*;
use oxide_charts::line::Curve;
use oxide_charts::{format_value, metric, nav, sparkline};

const SPARK_W: f64 = 160.0;
const SPARK_H: f64 = 40.0;
const SPARK_PAD: f64 = 3.0;

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// A KPI tile. With a trend, hover or touch the sparkline to read a point, or focus it
/// and use the arrow keys to step through points, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn MetricCard(
    /// What the figure measures, in sentence case.
    #[prop(into)]
    label: String,
    /// The current value.
    #[prop(into)]
    value: Signal<f64>,
    /// The value to compare against, such as last month's.
    #[prop(optional, into)]
    previous: MaybeProp<f64>,
    /// What the comparison is against, such as "vs last month".
    #[prop(optional, into)]
    comparison: Option<String>,
    /// Unit after the figure, such as "ms".
    #[prop(optional, into)]
    unit: Option<String>,
    /// Whether a rise is good news (default) or bad, such as for error rates.
    #[prop(default = true)]
    higher_is_better: bool,
    /// Recent values, oldest first, drawn as a sparkline.
    #[prop(optional, into)]
    trend: Signal<Vec<f64>>,
    /// A name for each trend point, such as its date. Default "Point n".
    #[prop(optional, into)]
    trend_labels: Signal<Vec<String>>,
    /// Formats the figure, tooltip and table values.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let delta = Memo::new(move |_| previous.get().map(|p| metric::delta(value.get(), p, higher_is_better)));
    let spark = Memo::new(move |_| trend.with(|t| sparkline::layout(t, SPARK_W, SPARK_H, SPARK_PAD, Curve::Linear)));
    let active = RwSignal::new(None::<usize>);
    let point_name = move |i: usize| trend_labels.with(|l| l.get(i).cloned()).unwrap_or_else(|| format!("Point {}", i + 1));
    let point_value = move |i: usize| trend.with(|t| t.get(i).copied()).filter(|v| v.is_finite());
    let describe = move |i: usize| Some(format!("{}: {}", point_name(i), point_value(i).map_or_else(|| "no value".into(), fmt)));
    let move_to = move |ev: leptos::ev::PointerEvent| {
        let Some(el) = ev.current_target().and_then(|t| leptos::wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(t).ok()) else {
            return;
        };
        let f = f64::from(ev.offset_x()) / f64::from(el.offset_width().max(1));
        let near = metric::spark_index(f, trend.with(Vec::len), SPARK_W, SPARK_PAD);
        if active.get_untracked() != near {
            active.set(near);
        }
    };
    let comparison_text = comparison.clone();
    let spark_label = format!("{label} trend. Use the arrow keys to step through points.");
    let unit_table = unit.clone();

    view! {
        <div data-scope="metric-card" data-part="root">
            <span data-part="label">{label.clone()}</span>
            <p data-part="figure">
                <span data-part="value">{move || fmt(value.get())}</span>
                {unit.map(|u| view! { <span data-part="unit">{u}</span> })}
            </p>
            {move || {
                delta
                    .get()
                    .map(|d| {
                        let icon = if d.trend == metric::Trend::Flat { IconName::ArrowRight } else { IconName::ArrowUp };
                        let change = d.ratio.map_or_else(
                            || format!("{}{}", if d.change > 0.0 { "+" } else { "" }, fmt(d.change)),
                            metric::signed_percent,
                        );
                        view! {
                            <div data-part="delta" data-trend=d.trend.attr() data-tone=d.tone>
                                <span data-part="delta-icon">
                                    <Icon name=icon size=14 />
                                </span>
                                <span data-part="delta-value">{change}</span>
                                {comparison_text.clone().map(|c| view! { <span data-part="comparison">{c}</span> })}
                            </div>
                        }
                    })
            }}
            <Show when=move || trend.with(|t| t.len() > 1)>
                <div
                    data-part="spark"
                    tabindex="0"
                    role="group"
                    aria-label=spark_label.clone()
                    style=format!("aspect-ratio: {SPARK_W} / {SPARK_H};")
                    on:pointermove=move_to
                    on:pointerdown=move_to
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), trend.with(Vec::len)) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    <svg viewBox=format!("0 0 {SPARK_W} {SPARK_H}") preserveAspectRatio="none" aria-hidden="true">
                        <path data-part="area" d=move || spark.get().area></path>
                        <path data-part="line" d=move || spark.get().line pathLength="1"></path>
                    </svg>
                    {move || {
                        let i = active.get()?;
                        let (x, y) = trend.with(|t| metric::spark_point(t, i, SPARK_W, SPARK_H, SPARK_PAD))?;
                        let (fx, fy) = (x / SPARK_W, y / SPARK_H);
                        Some(
                            view! {
                                <span data-part="marker" style=format!("left: {}; top: {};", pct(fx), pct(fy))></span>
                                <div
                                    data-part="tooltip"
                                    data-side={if fx > 0.5 { "left" } else { "right" }}
                                    style=format!("left: {}; top: {};", pct(fx), pct(fy))
                                    aria-hidden="true"
                                >
                                    <div data-part="tooltip-label">{point_name(i)}</div>
                                    <ul>
                                        <li data-part="tooltip-row">
                                            <span data-part="tooltip-name">"Value"</span>
                                            <span data-part="tooltip-value">{point_value(i).map(fmt).unwrap_or_default()}</span>
                                        </li>
                                    </ul>
                                </div>
                            },
                        )
                    }}
                    {move || spark.get().last.map(|(x, y)| view! {
                        <span data-part="last" style=format!("left: {}; top: {};", pct(x / SPARK_W), pct(y / SPARK_H))></span>
                    })}
                </div>
            </Show>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label.clone()}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Point"</th>
                        <th scope="col">{format!("Value{}", unit_table.map(|u| format!(" ({u})")).unwrap_or_default())}</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <th scope="row">"Current"</th>
                        <td>{move || fmt(value.get())}</td>
                    </tr>
                    {move || previous.get().map(|p| view! {
                        <tr>
                            <th scope="row">{comparison.clone().unwrap_or_else(|| "Previous".into())}</th>
                            <td>{fmt(p)}</td>
                        </tr>
                    })}
                    {move || {
                        (0..trend.with(Vec::len))
                            .map(|i| {
                                view! {
                                    <tr>
                                        <th scope="row">{point_name(i)}</th>
                                        <td>{point_value(i).map(fmt).unwrap_or_default()}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </div>
    }
}
