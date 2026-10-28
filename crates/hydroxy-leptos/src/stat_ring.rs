//! `<StatRing>`: a compact ring showing progress towards a goal, with the figure in the
//! middle. Progress comes from `hydroxy_charts::metric::progress`.

use hydroxy_charts::{format_value, metric};
use leptos::prelude::*;

/// A progress ring. Hover, touch or focus it to read the value against the goal;
/// Escape hides the reading.
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn StatRing(
    /// The current value.
    #[prop(into)]
    value: Signal<f64>,
    /// The goal the ring fills towards.
    #[prop(into)]
    goal: Signal<f64>,
    /// What the ring measures, such as "Weekly active". Shown under the figure.
    #[prop(into)]
    label: String,
    /// Ring diameter in px.
    #[prop(default = 112)]
    size: u16,
    /// Ring width in px.
    #[prop(default = 10)]
    thickness: u16,
    /// Formats the value and goal. The centre shows the share of the goal.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let progress = Memo::new(move |_| metric::progress(value.get(), goal.get()));
    // The centre shows the true share, so 112% reads as over the goal while the ring stays full.
    let share = move || {
        let g = goal.get();
        if g > 0.0 { format!("{}%", format_value((value.get() / g * 100.0).round())) } else { "—".into() }
    };
    let active = RwSignal::new(false);
    let r = (f64::from(size) - f64::from(thickness)) / 2.0;
    let c = f64::from(size) / 2.0;
    let live_label = label.clone();
    let tip_label = label.clone();
    let describe = move || format!("{live_label}: {} of {}, {}", fmt(value.get()), fmt(goal.get()), share());

    view! {
        <figure data-scope="stat-ring" data-part="root" data-over=move || (value.get() > goal.get() && goal.get() > 0.0).then_some("")>
            <div
                data-part="ring"
                tabindex="0"
                role="group"
                aria-label=format!("{label}. Focus to read the value against the goal.")
                style=format!("width: {size}px; height: {size}px;")
                on:pointerenter=move |_| active.set(true)
                on:pointerdown=move |_| active.set(true)
                on:pointerleave=move |_| active.set(false)
                on:focus=move |_| active.set(true)
                on:blur=move |_| active.set(false)
                on:keydown=move |ev| {
                    if ev.key() == "Escape" && active.get_untracked() {
                        ev.prevent_default();
                        active.set(false);
                    }
                }
            >
                <svg viewBox=format!("0 0 {size} {size}") aria-hidden="true">
                    <circle data-part="track" cx=c cy=c r=r stroke-width=thickness></circle>
                    <circle
                        data-part="value"
                        cx=c
                        cy=c
                        r=r
                        stroke-width=thickness
                        pathLength="1"
                        transform=format!("rotate(-90 {c} {c})")
                        style=move || format!("stroke-dasharray: {:.4} 1;", progress.get())
                    ></circle>
                </svg>
                <div data-part="center" aria-hidden="true">
                    <span data-part="center-value">{share}</span>
                    <span data-part="center-label">{label.clone()}</span>
                </div>
                <Show when=move || active.get()>
                    <div data-part="tooltip" style="left: 100%; top: 50%;" aria-hidden="true">
                        <div data-part="tooltip-label">
                            <span data-part="swatch" style="background: var(--chart-1);"></span>
                            {tip_label.clone()}
                        </div>
                        <ul>
                            <li data-part="tooltip-row">
                                <span data-part="tooltip-name">"Value"</span>
                                <span data-part="tooltip-value">{move || fmt(value.get())}</span>
                            </li>
                            <li data-part="tooltip-row">
                                <span data-part="tooltip-name">"Goal"</span>
                                <span data-part="tooltip-value">{move || fmt(goal.get())}</span>
                            </li>
                        </ul>
                    </div>
                </Show>
            </div>
            <span data-part="sr-live" aria-live="polite">{move || if active.get() { describe() } else { String::new() }}</span>
            <table data-part="sr-table">
                <caption>{label.clone()}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Measure"</th>
                        <th scope="col">"Value"</th>
                        <th scope="col">"Goal"</th>
                        <th scope="col">"Progress"</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <th scope="row">{label.clone()}</th>
                        <td>{move || fmt(value.get())}</td>
                        <td>{move || fmt(goal.get())}</td>
                        <td>{share}</td>
                    </tr>
                </tbody>
            </table>
        </figure>
    }
}
