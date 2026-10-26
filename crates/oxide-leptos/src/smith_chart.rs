//! `<SmithChart>`: impedance samples on the Smith chart, for RF work. Geometry comes
//! from `oxide_charts::smith`. Every trace is focusable; hovering one picks it out.

use leptos::prelude::*;
use oxide_charts::smith;

pub use oxide_charts::smith::Sample;

fn impedance(s: &Sample) -> String {
    format!("{} {} j{}", s.real, if s.imaginary < 0.0 { "−" } else { "+" }, s.imaginary.abs())
}

#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
/// Impedance sweeps inside the unit disc, with the resistance and reactance grid.
/// Samples that share a `trace` name form one swept trace; their first and last labels name its ends.
#[component]
pub fn SmithChart(
    /// The samples, in sweep order within each trace.
    #[prop(into)]
    samples: Signal<Vec<Sample>>,
    /// What the chart shows. Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Reference impedance the samples are measured against. 1 means already normalised;
    /// pass 50 when the samples are raw ohms.
    #[prop(default = 1.0)]
    z0: f64,
) -> impl IntoView {
    let layout = Memo::new(move |_| samples.with(|s| smith::layout(s, z0)));
    let aria = label.clone();
    view! {
        <figure data-scope="smith-chart" data-part="root">
            {move || {
                let l = layout.get();
                let grid = l
                    .resistance
                    .iter()
                    .map(|&(x, y, r)| view! { <circle data-part="grid" cx=x cy=y r=r></circle> })
                    .collect_view();
                let arcs = l.reactance.iter().map(|d| view! { <path data-part="grid" d=d.clone()></path> }).collect_view();
                let traces = l
                    .traces
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        let summary = t.samples.iter().map(|s| format!("{}: {}", s.label.clone().unwrap_or_default(), impedance(s))).collect::<Vec<_>>().join("; ");
                        let dots = t
                            .points
                            .iter()
                            .zip(t.samples.iter())
                            .enumerate()
                            .map(|(k, (&(x, y), s))| {
                                let end = k == 0 || k + 1 == t.points.len();
                                view! {
                                    <circle data-part="point" data-end=end.then_some("") cx=x cy=y r=if end { "4.5" } else { "3" }>
                                        <title>{format!("{}: {}", s.label.clone().unwrap_or_else(|| t.name.clone()), impedance(s))}</title>
                                    </circle>
                                }
                            })
                            .collect_view();
                        let ends = [(0, t.samples.first()), (t.points.len().saturating_sub(1), t.samples.last())]
                            .into_iter()
                            .filter(|_| t.points.len() > 1)
                            .filter_map(|(k, s)| {
                                let text = s?.label.clone()?;
                                let (x, y) = t.points[k];
                                let right = x >= l.centre.0;
                                Some(view! {
                                    <text
                                        data-part="end-label"
                                        x=(x + if right { 10.0 } else { -10.0 }).clamp(6.0, l.width - 6.0)
                                        y=(y - 9.0).clamp(12.0, l.height - 6.0)
                                        text-anchor=if right { "start" } else { "end" }
                                    >
                                        {text}
                                    </text>
                                })
                            })
                            .collect_view();
                        view! {
                            <g
                                data-part="series"
                                data-index=i
                                style=format!("--c: var(--chart-{});", i % 6 + 1)
                                tabindex="0"
                                role="img"
                                aria-label=format!("{}: {summary}", t.name)
                            >
                                <path data-part="line" d=t.path.clone()></path>
                                {dots}
                                {ends}
                            </g>
                        }
                    })
                    .collect_view();
                view! {
                    <svg data-part="marks" viewBox=format!("0 0 {} {}", l.width, l.height) role="group" aria-label=aria.clone()>
                        <line data-part="axis" x1=l.centre.0 - l.radius y1=l.centre.1 x2=l.centre.0 + l.radius y2=l.centre.1></line>
                        {grid}
                        {arcs}
                        <circle data-part="boundary" cx=l.centre.0 cy=l.centre.1 r=l.radius></circle>
                        {traces}
                    </svg>
                }
            }}
            <Show when=move || layout.with(|l| l.traces.len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        layout
                            .get()
                            .traces
                            .into_iter()
                            .enumerate()
                            .map(|(i, t)| {
                                view! {
                                    <li data-part="legend-item" data-index=i>
                                        <span data-part="swatch" style=format!("background: var(--chart-{});", i % 6 + 1)></span>
                                        {t.name}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr><th scope="col">"Trace"</th><th scope="col">"Sample"</th><th scope="col">"Impedance"</th></tr>
                </thead>
                <tbody>
                    {move || {
                        samples
                            .get()
                            .into_iter()
                            .map(|s| view! { <tr><th scope="row">{s.trace.clone()}</th><td>{s.label.clone().unwrap_or_default()}</td><td>{impedance(&s)}</td></tr> })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
