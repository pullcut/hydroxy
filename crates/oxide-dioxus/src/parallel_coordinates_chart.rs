//! `ParallelCoordinatesChart`: one vertical axis per dimension and one line per row
//! across them. Drag on an axis to brush a range; rows outside any brush dim. Geometry
//! comes from `oxide_charts::parallel`; thousands of rows draw on a canvas.

use dioxus::prelude::*;
use oxide_charts::parallel::{self, Range};
use oxide_charts::plot::{self, Tip, TipRow};
use oxide_charts::{format_value, raster};

pub use oxide_charts::parallel::Dimension;

use crate::chart_frame::use_export;
use crate::line_chart::plot_fractions;
use crate::raster_canvas::{Frame, RasterCanvas};

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// How close to an axis, in px, a press starts a brush rather than reading a line.
const GRAB_PX: f64 = 14.0;

/// One line per row across an axis per dimension, each axis on its own scale. Drag up or
/// down an axis to keep only rows in that range; brushes on several axes combine, and a
/// click on an axis clears its brush. Hover a line, or focus the plot and use Left/Right,
/// to read a row; Escape clears.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn ParallelCoordinatesChart(
    /// The axes, left to right, each with one value per row.
    dimensions: ReadSignal<Vec<Dimension>>,
    /// Each row's category, which picks its colour. Up to six read well; leave empty for one colour.
    #[props(default)]
    groups: ReadSignal<Vec<String>>,
    /// Each row's name, for the tooltip and table. Default "Row 1", "Row 2"…
    #[props(default)]
    names: ReadSignal<Vec<String>>,
    /// What the chart shows, such as "Cars by economy, power and weight". Used as the accessible name.
    label: String,
    /// Plot height in px, excluding the axis names and legend.
    #[props(default = 320)]
    height: u16,
    /// Formats values for the ticks, tooltip and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    // Memos, so a hover never re-lays out or re-filters the rows.
    let layout = use_memo(move || parallel::layout(&dimensions.read()));
    let grouped = use_memo(move || parallel::groups(&groups.read()));
    let mut brushes = use_signal(Vec::<Option<Range>>::new);
    let keep = use_memo(move || parallel::selected(&layout.read(), &brushes.read()));
    let mut active = use_signal(|| None::<usize>);
    let mut at_x = use_signal(|| 0.5_f64);
    let mut dragging = use_signal(|| None::<(usize, f64)>);
    let mut size = use_signal(|| (0.0_f64, 0.0_f64));
    let row_name = move |r: usize| names.read().get(r).cloned().unwrap_or_else(|| format!("Row {}", r + 1));
    let table_head = move || {
        let mut h = vec![String::new()];
        if grouped.read().0.len() > 1 {
            h.push("Group".into());
        }
        h.extend(dimensions.read().iter().map(|d| d.name.clone()));
        h
    };
    let table_row = move |r: usize| {
        let mut row = vec![row_name(r)];
        let g = grouped.read();
        if g.0.len() > 1 {
            row.push(g.0.get(g.1.get(r).copied().unwrap_or(0)).cloned().unwrap_or_default());
        }
        row.extend(dimensions.read().iter().map(|d| plot::cell(d.values.get(r).copied(), fmt)));
        row
    };
    use_export(move |()| std::iter::once(table_head()).chain((0..layout.read().ys.len()).map(table_row)).collect());
    let draw = use_callback(move |f: Frame| {
        let (groups, muted) = f.colours.split_at(f.colours.len().saturating_sub(1));
        let muted = muted.first().copied().unwrap_or([128, 128, 128, 255]);
        parallel::raster(&layout.read(), &keep.read(), &grouped.read().1, (groups, muted), (f.width, f.height), f.dpr)
    });

    let l = layout.read();
    let n = l.ys.len();
    let big = n > raster::DOM_LIMIT;
    let kept = keep.read();
    let shown = kept.iter().filter(|k| **k).count();
    let marks = brushes.read().clone();
    let filtered = marks.iter().any(Option::is_some);
    let (group_names, group_of) = grouped.read().clone();
    let colour_of = |r: usize| plot::colour(group_of.get(r).copied().unwrap_or(0));
    let colours: Vec<String> = (0..group_names.len().clamp(1, 6))
        .map(plot::colour)
        .chain(std::iter::once("var(--chart-muted)".to_string()))
        .collect();
    let dims = dimensions.read();
    let gutter = plot::axis_width(
        l.axes.first().map(|a| a.ticks(fmt)).unwrap_or_default().iter().map(|t| t.text.as_str()),
        28.0,
    );
    let current = active().filter(|&r| r < n);
    let tip = current.and_then(|r| {
        let x = at_x();
        let y = parallel::height_at(&l.xs, &l.ys[r], x)?;
        let mut rows: Vec<TipRow> = Vec::new();
        if group_names.len() > 1 {
            let g = group_of.get(r).copied().unwrap_or(0);
            rows.push(TipRow::new(Some(plot::colour(g)), group_names.get(g).cloned().unwrap_or_default(), ""));
        }
        rows.extend(dims.iter().map(|d| TipRow::new(None, d.name.clone(), plot::cell(d.values.get(r).copied(), fmt))));
        Some(Tip { x, y, title: row_name(r), rows })
    });
    let announcement = tip.as_ref().map(Tip::announce).unwrap_or_default();
    let last = l.xs.len().saturating_sub(1);
    let listed = raster::table_rows(n, dims.len() + 1);
    let xs_for_pointer = l.xs.clone();
    let mut set_brush = move |axis: usize, range: Option<Range>| {
        let axes = layout.peek().xs.len();
        brushes.with_mut(|b| {
            b.resize(axes, None);
            if let Some(slot) = b.get_mut(axis) {
                *slot = range;
            }
        });
    };
    let mut hover = move |x: f64, y: f64| {
        let next = parallel::nearest(&layout.peek(), &keep.peek(), x, y, 0.06);
        at_x.set(x);
        if *active.peek() != next {
            active.set(next);
        }
    };

    rsx! {
        figure {
            "data-scope": "parallel-coordinates-chart",
            "data-part": "root",
            "data-raster": big.then_some(""),
            "data-filtered": filtered.then_some(""),
            style: "--ox-axis: {gutter}px; --ox-height: {height}px;",
            div { "data-part": "plot",
                if big {
                    RasterCanvas { colours, draw }
                } else {
                    svg { "data-part": "lines", view_box: "0 0 100 100", preserve_aspect_ratio: "none", "aria-hidden": "true",
                        for (r , row) in l.ys.iter().enumerate() {
                            path {
                                key: "{r}",
                                "data-part": "line",
                                "data-dim": (!kept.get(r).copied().unwrap_or(true)).then_some(""),
                                d: parallel::path(&l.xs, row),
                                stroke: colour_of(r),
                                "vector-effect": "non-scaling-stroke",
                            }
                        }
                    }
                }
                svg { "data-part": "highlight", view_box: "0 0 100 100", preserve_aspect_ratio: "none", "aria-hidden": "true",
                    if let Some(r) = current {
                        path { "data-part": "active-line", d: parallel::path(&l.xs, &l.ys[r]), stroke: colour_of(r), "vector-effect": "non-scaling-stroke" }
                    }
                }
                for (a , (axis , dim)) in l.axes.iter().zip(dims.iter()).enumerate() {
                    div {
                        key: "{a}",
                        "data-part": "axis",
                        "data-edge": if a == 0 { "start" } else if a == last { "end" } else { "" },
                        style: "--x: {pct(l.xs[a])}; --i: {a};",
                        span { "data-part": "axis-name", title: "{dim.name}", "{dim.name}" }
                        for t in axis.ticks(fmt) {
                            span { "data-part": "tick", style: "--p: {pct(t.at)};", "{t.text}" }
                        }
                        if let Some((lo , hi)) = marks.get(a).copied().flatten() {
                            span { "data-part": "brush", style: "--lo: {pct(lo)}; --hi: {pct(hi)};" }
                        }
                    }
                }
                div {
                    "data-part": "overlay",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Drag along an axis to filter. Use the left and right arrow keys to read each row.",
                    onresize: move |e| {
                        if let Ok(s) = e.data().get_border_box_size() {
                            size.set((s.width, s.height));
                        }
                    },
                    onpointerdown: move |e| {
                        let (w, h) = size();
                        let Some((x, y)) = plot_fractions(e.data().element_coordinates(), (w, h)) else { return };
                        if let Some(a) = parallel::axis_near(&xs_for_pointer, x, GRAB_PX / w) {
                            dragging.set(Some((a, y)));
                            active.set(None);
                        } else {
                            hover(x, y);
                        }
                    },
                    onpointermove: move |e| {
                        let Some((x, y)) = plot_fractions(e.data().element_coordinates(), size()) else { return };
                        match dragging() {
                            Some((a, start)) => {
                                if let Some(range) = parallel::drag(start, y) {
                                    set_brush(a, Some(range));
                                }
                            }
                            None => hover(x, y),
                        }
                    },
                    onpointerup: move |e| {
                        if let (Some((a, start)), Some((_, y))) = (dragging(), plot_fractions(e.data().element_coordinates(), size())) {
                            set_brush(a, parallel::drag(start, y));
                        }
                        dragging.set(None);
                    },
                    onpointerleave: move |_| {
                        dragging.set(None);
                        active.set(None);
                    },
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if let Some(next) = parallel::step(&e.key().to_string(), &keep.read(), active()) {
                            e.prevent_default();
                            at_x.set(0.5);
                            active.set(next);
                        }
                    },
                    if let Some(t) = tip {
                        div {
                            "data-part": "tooltip",
                            "data-side": t.side(),
                            "data-vertical": t.vertical(),
                            style: "left: {pct(t.x)}; bottom: {pct(t.y)};",
                            div { "data-part": "tooltip-label", "{t.title}" }
                            ul {
                                for r in t.rows.iter() {
                                    li { "data-part": "tooltip-row",
                                        if let Some(c) = &r.colour {
                                            span { "data-part": "swatch", style: "background: {c};" }
                                        }
                                        span { "data-part": "tooltip-name", "{r.name}" }
                                        span { "data-part": "tooltip-value", "{r.value}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { "data-part": "footer",
                if group_names.len() > 1 {
                    ul { "data-part": "legend", "aria-hidden": "true",
                        for (i , g) in group_names.iter().enumerate() {
                            li { "data-part": "legend-item",
                                span { "data-part": "swatch", style: "background: {plot::colour(i)};" }
                                "{g}"
                            }
                        }
                    }
                }
                span { "data-part": "count", "aria-live": "polite",
                    if filtered {
                        "{shown} of {n} rows"
                    } else {
                        "{n} rows"
                    }
                }
                if filtered {
                    button { r#type: "button", "data-part": "clear", onclick: move |_| brushes.set(Vec::new()), "Clear filters" }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption {
                    "{label}"
                    if listed < n {
                        " (first {listed} of {n} rows)"
                    }
                }
                thead {
                    tr {
                        for h in table_head() {
                            th { scope: "col", "{h}" }
                        }
                    }
                }
                tbody {
                    for r in 0..listed {
                        tr {
                            for (j , c) in table_row(r).into_iter().enumerate() {
                                if j == 0 {
                                    th { scope: "row", "{c}" }
                                } else {
                                    td { "{c}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
