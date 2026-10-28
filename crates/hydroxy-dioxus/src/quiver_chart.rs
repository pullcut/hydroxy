//! `QuiverChart`: a vector field as arrows on a grid. Each arrow points along the field;
//! colour (and, optionally, length) shows its magnitude, and streamlines can trace the flow.
//! Large grids draw on a canvas. Geometry comes from `hydroxy_charts::quiver`.

use dioxus::prelude::*;
use hydroxy_charts::line::nearest_index;
use hydroxy_charts::nav::grid_key;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{quiver, raster};

pub use hydroxy_charts::quiver::VectorField;

use crate::chart_frame::use_export;
use crate::plot_kit::{Plot, colour, formatter};
use crate::raster_canvas::{Frame, RasterCanvas};

/// Steps in the canvas colour ramp; DOM arrows mix the same colours continuously.
const RAMP_STEPS: usize = 32;

/// The sequential magnitude colour at `level` (0..=1): one chart hue, faint to full.
fn ramp_colour(level: f64) -> String {
    format!("color-mix(in oklab, {} {:.1}%, var(--surface-sunken))", colour(0), 25.0 + level.clamp(0.0, 1.0) * 75.0)
}

/// An arrow per grid point of a vector field, coloured from faint (weak) to strong (the
/// largest magnitude), optionally scaled in length and overlaid with streamlines. Hover or
/// touch to read the nearest vector; focus the plot and use the arrow keys to move across
/// the grid, Home/End to jump along a row, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[allow(clippy::cast_precision_loss, reason = "grid sizes are small")]
#[component]
pub fn QuiverChart(
    /// The field: `VectorField::from_fn(xs, ys, |x, y| (u, v))`, or its grids directly.
    field: ReadSignal<VectorField>,
    /// What the chart shows, such as "Surface wind over the bay". Used as the accessible name.
    label: String,
    /// Arrow length follows magnitude. When `false`, arrows share one length and only colour shows strength.
    #[props(default = true)]
    scaled: bool,
    /// Roughly how many streamlines to trace through the field; 0 for none.
    #[props(default)]
    streamlines: usize,
    /// Plot height in px, excluding the axis labels.
    #[props(default = 300)]
    height: u16,
    /// Formats x and y positions for the axes, tooltip and table.
    format: Option<Callback<f64, String>>,
    /// Formats magnitudes for the tooltip, legend and table, such as `|v| format!("{v:.1} m/s")`.
    magnitude_format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = formatter(format);
    let fmag = move |v: f64| magnitude_format.map_or_else(|| format!("{v:.2}"), |f| f.call(v));
    let layout = use_memo(move || quiver::layout(&field.read(), scaled, streamlines));
    let row_of = move |a: &quiver::Arrow| {
        vec![fmt(a.x), fmt(a.y), fmag(a.u), fmag(a.v), fmag(a.magnitude), format!("{:.0}°", a.degrees())]
    };
    let head = || ["x", "y", "u", "v", "Magnitude", "Direction"].map(String::from).to_vec();
    use_export(move |()| std::iter::once(head()).chain(layout.read().arrows.iter().map(row_of)).collect());
    let draw = use_callback(move |f: Frame| quiver::paint(&layout.read(), &f.colours, f.width, f.height, f.dpr));
    let l = layout.read();
    let (rows_n, cols_n) = l.shape;
    let big = l.arrows.len() > raster::DOM_LIMIT;
    // (row from the top, column), so Up and Down move the way they look.
    let mut active = use_signal(|| None::<(usize, usize)>);
    let current = active().filter(|&(r, c)| r < rows_n && c < cols_n);
    let arrow = current.and_then(|(top, col)| l.arrow(rows_n - 1 - top, col).copied());
    let (xs, ys): (Vec<f64>, Vec<f64>) = {
        let f = field.read();
        (f.xs.iter().map(|&x| l.x_axis.scale.map(x)).collect(), f.ys.iter().map(|&y| l.axis.scale.map(y)).collect())
    };
    let tip = arrow.map(|a| {
        let rows = vec![
            TipRow { active: true, ..TipRow::new(Some(ramp_colour(a.level)), "Magnitude", fmag(a.magnitude)) },
            TipRow::new(None, "Direction", format!("{:.0}°", a.degrees())),
            TipRow::new(None, "Components", format!("{}, {}", fmag(a.u), fmag(a.v))),
        ];
        Tip { x: a.centre().0, y: a.centre().1, title: format!("({}, {})", fmt(a.x), fmt(a.y)), rows }
    });
    let rows: Vec<Vec<String>> = l.arrows.iter().take(raster::table_rows(l.arrows.len(), 6)).map(row_of).collect();
    let ramp: Vec<String> = (0..RAMP_STEPS).map(|i| ramp_colour(i as f64 / (RAMP_STEPS - 1) as f64)).collect();

    rsx! {
        Plot {
            scope: "quiver-chart",
            label,
            hint: "Use the arrow keys to move across the grid.",
            height,
            ticks: l.axis.ticks(fmt),
            labels: l.x_axis.ticks(fmt),
            annotations: Vec::new(),
            tip,
            head: head(),
            rows,
            on_key: move |key: String| grid_key(&key, active(), rows_n, cols_n).map(|next| active.set(next)).is_some(),
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(x, y)| Some((ys.len() - 1 - nearest_index(&ys, y)?, nearest_index(&xs, x)?)));
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                if !l.streamlines.is_empty() {
                    svg { "data-part": "streamlines", view_box: "0 0 100 100", "preserveAspectRatio": "none", "aria-hidden": "true",
                        for d in l.streamlines.iter() {
                            path { "data-part": "streamline", d: "{d}" }
                        }
                    }
                }
                if big {
                    RasterCanvas { colours: ramp, draw }
                } else {
                    div { "data-part": "arrows", "aria-hidden": "true",
                        for a in l.arrows.iter() {
                            span {
                                key: "{a.row}-{a.col}",
                                "data-part": "arrow",
                                "data-still": (a.magnitude == 0.0).then_some(""),
                                "data-active": (current == Some((rows_n - 1 - a.row, a.col))).then_some(""),
                                style: "--c: {a.px:.4}; --v: {a.py:.4}; --u: {a.dx:.5}; --w: {a.dy:.5}; --i: {a.row + a.col}; --col: {ramp_colour(a.level)};",
                            }
                        }
                    }
                }
            },
            overlay: arrow.filter(|_| big).map(|a| rsx! {
                span { "data-part": "dot", style: "--c: {a.centre().0:.4}; --v: {a.centre().1:.4}; background: {ramp_colour(a.level)};" }
            }),
        }
        div { "data-scope": "quiver-chart", "data-part": "scale", "aria-hidden": "true",
            span { "data-part": "scale-name", "Magnitude" }
            span { "data-part": "scale-value", {fmag(l.range.0)} }
            span { "data-part": "ramp" }
            span { "data-part": "scale-value", {fmag(l.range.1)} }
        }
    }
}
