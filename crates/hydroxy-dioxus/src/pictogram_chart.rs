//! `PictogramChart`: an isotype chart, each category's count drawn as a row of repeated
//! icons. Icon counts come from `hydroxy_charts::pictogram`. More than six categories fold
//! into "Other".

use dioxus::prelude::*;
use hydroxy_charts::{fold_other, format_value, nav, pictogram, plot};

pub use hydroxy_charts::Datum;
pub use hydroxy_charts::pictogram::PictogramIcon;

/// Most icons a row draws; a smaller unit is raised to fit.
const MOST_ICONS: usize = 100;

fn row_colour(i: usize, len: usize, folded: bool) -> String {
    if folded && i + 1 == len { "var(--chart-muted)".into() } else { format!("var(--chart-{})", i.min(5) + 1) }
}

/// One row per category, its value drawn as repeated icons that each stand for `unit`
/// units; the last icon is cut to the remainder. A key under the rows says what one icon
/// is worth. Icons pop in row by row. Hover, touch or focus a row to read it; focus the
/// chart and use Up/Down to move between rows, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn PictogramChart(
    /// The categories, top to bottom, in categorical colour order.
    data: ReadSignal<Vec<Datum>>,
    /// What the chart shows, such as "Visitors by region". Used as the accessible name.
    label: String,
    /// What one icon stands for. Default: a 1, 2 or 5 × 10ⁿ unit that fits the largest
    /// row in `columns` icons. A unit that would need more than 100 icons a row is raised.
    unit: Option<f64>,
    /// What is counted, for the key, such as "people". Default: no noun.
    unit_label: Option<String>,
    /// The icon: `Person` (default), `Square`, `Circle` or `Custom(path)` in a 24×24 box.
    #[props(default)]
    icon: PictogramIcon,
    /// Icons per line before a row wraps.
    #[props(default = 20)]
    columns: usize,
    /// Icon size in px.
    #[props(default = 20)]
    icon_size: u16,
    /// Formats values for the rows, key, tooltip and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let columns = columns.max(1);
    let path = icon.path().to_string();
    let raw = data();
    let parts = fold_other(&raw, 6);
    let folded = raw.len() > 6;
    let len = parts.len();
    let values: Vec<f64> = parts.iter().map(|d| d.value).collect();
    let fit = pictogram::auto_unit(values.iter().copied().filter(|v| v.is_finite()).fold(0.0, f64::max), columns);
    let each = pictogram::unit_for(&values, Some(unit.unwrap_or(fit)), MOST_ICONS);
    let rows: Vec<(Datum, String, Vec<f64>)> = parts
        .iter()
        .enumerate()
        .map(|(i, d)| (d.clone(), row_colour(i, len, folded), pictogram::icons(d.value, each)))
        .collect();
    let noun = unit_label.map(|n| format!(" {n}")).unwrap_or_default();
    let icons_of = |v: f64| format_value((v / each).max(0.0));
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < len);
    let announcement = current
        .map(|i| format!("{}: {}{noun}, {} icons", parts[i].label, fmt(parts[i].value), icons_of(parts[i].value)));

    rsx! {
        figure {
            "data-scope": "pictogram-chart",
            "data-part": "root",
            "data-active": current.map(|_| ""),
            style: "--ox-icon: {icon_size}px; --ox-cols: {columns};",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the up and down arrow keys to move between rows.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if let Some(next) = nav::list_key(plot::turn_key(&e.key().to_string()), active(), len) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                for (i , (d , fill , icons)) in rows.into_iter().enumerate() {
                    div {
                        key: "{i}",
                        "data-part": "row",
                        "data-index": "{i}",
                        "data-active": (current == Some(i)).then_some(""),
                        style: "--r: {i}; --ox-fill: {fill};",
                        onpointerenter: move |_| active.set(Some(i)),
                        onpointerdown: move |_| active.set(Some(i)),
                        span { "data-part": "row-label", title: "{d.label}", "{d.label}" }
                        span { "data-part": "icons",
                            for (j , f) in icons.into_iter().enumerate() {
                                span {
                                    key: "{j}",
                                    "data-part": "icon",
                                    "data-partial": (f < 1.0).then_some(""),
                                    style: "--i: {j}; --f: {f:.4};",
                                    svg { "data-part": "icon-ghost", view_box: "0 0 24 24", "aria-hidden": "true",
                                        path { d: "{path}" }
                                    }
                                    svg { "data-part": "icon-fill", view_box: "0 0 24 24", "aria-hidden": "true",
                                        path { d: "{path}" }
                                    }
                                }
                            }
                            span { "data-part": "anchor",
                                if current == Some(i) {
                                    div { "data-part": "tooltip", "data-anchored": "", style: "left: 0; top: 50%;", "aria-hidden": "true",
                                        div { "data-part": "tooltip-label",
                                            span { "data-part": "swatch", style: "background: {fill};" }
                                            "{d.label}"
                                        }
                                        ul {
                                            li { "data-part": "tooltip-row",
                                                span { "data-part": "tooltip-name", "Value" }
                                                span { "data-part": "tooltip-value", {fmt(d.value)} }
                                            }
                                            li { "data-part": "tooltip-row",
                                                span { "data-part": "tooltip-name", "Icons" }
                                                span { "data-part": "tooltip-value", {icons_of(d.value)} }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        span { "data-part": "row-value", {fmt(d.value)} }
                    }
                }
            }
            p { "data-part": "key", "aria-hidden": "true",
                svg { "data-part": "key-icon", view_box: "0 0 24 24",
                    path { d: "{path}" }
                }
                "= {fmt(each)}{noun}"
            }
            span { "data-part": "sr-live", "aria-live": "polite", {announcement.unwrap_or_default()} }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Category" }
                        th { scope: "col", "Value" }
                        th { scope: "col", "Icons" }
                    }
                }
                tbody {
                    for d in parts.iter() {
                        tr {
                            th { scope: "row", "{d.label}" }
                            td { {fmt(d.value)} }
                            td { {icons_of(d.value)} }
                        }
                    }
                }
            }
        }
    }
}
