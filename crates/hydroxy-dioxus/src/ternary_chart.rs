//! `TernaryChart`: compositions of three parts as points in a triangle, with
//! gridlines every 20%. A sits bottom left, B bottom right, C at the top. Geometry
//! comes from `hydroxy_charts::ternary`.

use dioxus::prelude::*;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, nav, ternary};

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
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn TernaryChart(
    /// The groups, one colour each in categorical order.
    groups: ReadSignal<Vec<TernaryGroup>>,
    /// What the chart shows, such as "Soil samples by texture". Used as the accessible name.
    label: String,
    /// Name of the first component, at the bottom-left corner.
    a_label: String,
    /// Name of the second component, at the bottom-right corner.
    b_label: String,
    /// Name of the third component, at the top corner.
    c_label: String,
    /// Triangle width in px.
    #[props(default = 360)]
    size: u16,
    /// Formats component values for the tooltip and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let groups_v = groups();
    let dots = ternary::dots(&groups_v);
    let n = dots.len();
    let grid = ternary::grid(5);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let tip = current.and_then(|i| {
        let d = dots[i];
        let p = &groups_v[d.group].points[d.index];
        p.shares().map(|s| (d, p, s))
    });
    let announcement = tip
        .map(|(_, p, (a, b, c))| {
            format!("{}: {a_label} {}, {b_label} {}, {c_label} {}", p.label, percent(a), percent(b), percent(c))
        })
        .unwrap_or_default();
    let h = ternary::HEIGHT * 100.0;

    rsx! {
        figure { "data-scope": "ternary-chart", "data-part": "root",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": nav::hinted(&label, nav::HINT_POINTS),
                style: "width: {size}px; aspect-ratio: 1 / {ternary::HEIGHT:.4};",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if let Some(next) = nav::list_key(&e.key().to_string(), active(), n) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg { view_box: "0 0 100 {h:.3}", "preserveAspectRatio": "none", "aria-hidden": "true",
                    for l in grid.iter() {
                        polyline { "data-part": "gridline", "data-axis": "{l.axis}", points: "{svg_at(l.from)} {svg_at(l.to)}" }
                    }
                    path { "data-part": "edge", d: "M0,{h:.3} L100,{h:.3} L50,0 Z" }
                }
                for l in grid.iter() {
                    span { "data-part": "tick", "data-axis": "{l.axis}", "aria-hidden": "true", style: "left: {pct(l.from.0)}; bottom: {pct(l.from.1)};",
                        {percent(l.value)}
                    }
                }
                span { "data-part": "corner", "data-corner": "a", "aria-hidden": "true", "{a_label}" }
                span { "data-part": "corner", "data-corner": "b", "aria-hidden": "true", "{b_label}" }
                span { "data-part": "corner", "data-corner": "c", "aria-hidden": "true", "{c_label}" }
                for (i , d) in dots.iter().enumerate() {
                    span {
                        key: "{i}",
                        "data-part": "dot",
                        "data-group": "{d.group}",
                        "data-active": (current == Some(i)).then_some(""),
                        style: "left: {pct(d.at.0)}; bottom: {pct(d.at.1)}; --i: {i.min(40)}; background: {colour(d.group)};",
                        onpointerenter: move |_| active.set(Some(i)),
                        onpointerdown: move |_| active.set(Some(i)),
                    }
                }
                if let Some((d , p , (sa , sb , sc))) = tip {
                    ChartTip {
                        tip: Tip {
                            x: d.at.0,
                            y: d.at.1,
                            title: format!("{} · {}", p.label, groups_v[d.group].name),
                            rows: [(&a_label, sa, p.a), (&b_label, sb, p.b), (&c_label, sc, p.c)]
                                .into_iter()
                                .map(|(name, share, raw)| TipRow::new(None, name.clone(), format!("{} ({})", percent(share), fmt(raw))))
                                .collect(),
                        },
                        swatch: colour(d.group),
                    }
                }
            }
            if groups_v.len() > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i , g) in groups_v.iter().enumerate() {
                        li { "data-part": "legend-item",
                            span { "data-part": "swatch", style: "background: {colour(i)};" }
                            "{g.name}"
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Group" }
                        th { scope: "col", "Point" }
                        th { scope: "col", "{a_label}" }
                        th { scope: "col", "{b_label}" }
                        th { scope: "col", "{c_label}" }
                    }
                }
                tbody {
                    for g in groups_v.iter() {
                        for p in g.points.iter() {
                            tr {
                                td { "{g.name}" }
                                th { scope: "row", "{p.label}" }
                                td { {fmt(p.a)} }
                                td { {fmt(p.b)} }
                                td { {fmt(p.c)} }
                            }
                        }
                    }
                }
            }
        }
    }
}
