//! `<TernaryChart>`: compositions of three parts as points in a triangle, with
//! gridlines every 20%. A sits bottom left, B bottom right, C at the top. Geometry
//! comes from `hydroxy_charts::ternary`.

use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, nav, ternary};
use leptos::prelude::*;

use crate::plot_kit::{ChartTip, colour_or_muted, pct};

pub use hydroxy_charts::ternary::{TernaryGroup, TernaryPoint};

/// Group `i`'s colour; groups past the sixth share the muted colour.
fn colour(i: usize) -> String {
    colour_or_muted(i, 6)
}

fn percent(f: f64) -> String {
    format!("{}%", format_value(f * 100.0))
}

/// The SVG point for a box position (y from the bottom) in the 100-wide viewBox.
fn svg_at((x, y): (f64, f64)) -> String {
    format!("{:.3},{:.3}", x * 100.0, (1.0 - y) * ternary::HEIGHT * 100.0)
}

/// A ternary plot. Hover, touch or focus a point to read its shares; focus the plot and
/// use the arrow keys to step through points, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn TernaryChart(
    /// The groups, one colour each in categorical order.
    #[prop(into)]
    groups: Signal<Vec<TernaryGroup>>,
    /// What the chart shows, such as "Soil samples by texture". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Name of the first component, at the bottom-left corner.
    #[prop(into)]
    a_label: String,
    /// Name of the second component, at the bottom-right corner.
    #[prop(into)]
    b_label: String,
    /// Name of the third component, at the top corner.
    #[prop(into)]
    c_label: String,
    /// Triangle width in px.
    #[prop(default = 360)]
    size: u16,
    /// Formats component values for the tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let dots = Memo::new(move |_| groups.with(|g| ternary::dots(g)));
    let active = RwSignal::new(None::<usize>);
    let names = StoredValue::new([a_label.clone(), b_label.clone(), c_label.clone()]);
    let point = move |d: ternary::Dot| groups.with(|g| g.get(d.group).and_then(|g| g.points.get(d.index).cloned()));
    let describe = move |i: usize| {
        let p = point(dots.with(|ds| ds.get(i).copied())?)?;
        let (a, b, c) = p.shares()?;
        let [na, nb, nc] = names.get_value();
        Some(format!("{}: {na} {}, {nb} {}, {nc} {}", p.label, percent(a), percent(b), percent(c)))
    };
    let h = ternary::HEIGHT * 100.0;
    let edge = format!("M0,{h:.3} L100,{h:.3} L50,0 Z");

    view! {
        <figure data-scope="ternary-chart" data-part="root">
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=nav::hinted(&label, nav::HINT_POINTS)
                style=format!("width: {size}px; aspect-ratio: 1 / {:.4};", ternary::HEIGHT)
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), dots.with(Vec::len)) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <svg viewBox=format!("0 0 100 {h:.3}") preserveAspectRatio="none" aria-hidden="true">
                    {ternary::grid(5)
                        .into_iter()
                        .map(|l| view! { <polyline data-part="gridline" data-axis=l.axis points=format!("{} {}", svg_at(l.from), svg_at(l.to))></polyline> })
                        .collect_view()}
                    <path data-part="edge" d=edge></path>
                </svg>
                {ternary::grid(5)
                    .into_iter()
                    .map(|l| {
                        view! {
                            <span data-part="tick" data-axis=l.axis aria-hidden="true" style=format!("left: {}; bottom: {};", pct(l.from.0), pct(l.from.1))>
                                {percent(l.value)}
                            </span>
                        }
                    })
                    .collect_view()}
                <span data-part="corner" data-corner="a" aria-hidden="true">{a_label.clone()}</span>
                <span data-part="corner" data-corner="b" aria-hidden="true">{b_label.clone()}</span>
                <span data-part="corner" data-corner="c" aria-hidden="true">{c_label.clone()}</span>
                {move || {
                    dots.get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, d)| {
                            view! {
                                <span
                                    data-part="dot"
                                    data-group=d.group
                                    data-active=move || (active.get() == Some(i)).then_some("")
                                    style=format!("left: {}; bottom: {}; --i: {}; background: {};", pct(d.at.0), pct(d.at.1), i.min(40), colour(d.group))
                                    on:pointerenter=move |_| active.set(Some(i))
                                    on:pointerdown=move |_| active.set(Some(i))
                                ></span>
                            }
                        })
                        .collect_view()
                }}
                {move || {
                    let i = active.get()?;
                    let d = dots.with(|ds| ds.get(i).copied())?;
                    let p = point(d)?;
                    let shares = p.shares()?;
                    let group = groups.with(|g| g[d.group].name.clone());
                    let [na, nb, nc] = names.get_value();
                    let row = |name: String, share: f64, raw: f64| {
                        TipRow::new(None, name, format!("{} ({})", percent(share), fmt(raw)))
                    };
                    let tip = Tip {
                        x: d.at.0,
                        y: d.at.1,
                        title: format!("{} · {group}", p.label),
                        rows: vec![row(na, shares.0, p.a), row(nb, shares.1, p.b), row(nc, shares.2, p.c)],
                    };
                    Some(view! { <ChartTip tip swatch=colour(d.group) /> })
                }}
            </div>
            <Show when=move || groups.with(|g| g.len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        groups
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, g)| {
                                view! {
                                    <li data-part="legend-item">
                                        <span data-part="swatch" style=format!("background: {};", colour(i))></span>
                                        {g.name}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label.clone()}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Group"</th>
                        <th scope="col">"Point"</th>
                        <th scope="col">{a_label}</th>
                        <th scope="col">{b_label}</th>
                        <th scope="col">{c_label}</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        groups
                            .get()
                            .into_iter()
                            .flat_map(|g| g.points.into_iter().map(move |p| (g.name.clone(), p)))
                            .map(|(name, p)| {
                                view! {
                                    <tr>
                                        <td>{name}</td>
                                        <th scope="row">{p.label}</th>
                                        <td>{fmt(p.a)}</td>
                                        <td>{fmt(p.b)}</td>
                                        <td>{fmt(p.c)}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
