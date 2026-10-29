//! `<QuiverChart>`: a vector field as arrows on a grid. Each arrow points along the field;
//! colour (and, optionally, length) shows its magnitude, and streamlines can trace the flow.
//! Large grids draw on a canvas. Geometry comes from `hydroxy_charts::quiver`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::line::nearest_index;
use hydroxy_charts::nav::grid_key;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{quiver, raster};
use leptos::prelude::*;

pub use hydroxy_charts::quiver::VectorField;
use hydroxy_charts::quiver::ramp_colour;

use crate::chart_frame::provide_export;
use crate::plot_kit::{Plot, formatter};
use crate::raster_canvas::{Frame, RasterCanvas};

/// Steps in the canvas colour ramp; DOM arrows mix the same colours continuously.
const RAMP_STEPS: usize = 32;

/// An arrow per grid point of a vector field, coloured from faint (weak) to strong (the
/// largest magnitude), optionally scaled in length and overlaid with streamlines. Hover or
/// touch to read the nearest vector; focus the plot and use the arrow keys to move across
/// the grid, Home/End to jump along a row, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::cast_precision_loss, reason = "grid sizes are small")]
#[component]
pub fn QuiverChart(
    /// The field: `VectorField::from_fn(xs, ys, |x, y| (u, v))`, or its grids directly.
    #[prop(into)]
    field: Signal<VectorField>,
    /// What the chart shows, such as "Surface wind over the bay". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Arrow length follows magnitude. When `false`, arrows share one length and only colour shows strength.
    #[prop(default = true)]
    scaled: bool,
    /// Roughly how many streamlines to trace through the field; 0 for none.
    #[prop(optional)]
    streamlines: usize,
    /// Plot height in px, excluding the axis labels.
    #[prop(default = 300)]
    height: u16,
    /// Formats x and y positions for the axes, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Formats magnitudes for the tooltip, legend and table, such as `|v| format!("{v:.1} m/s")`.
    #[prop(optional, into)]
    magnitude_format: Option<Callback<f64, String>>,
    /// Called with the mark under the pointer or keyboard focus, and `None` when it clears.
    #[prop(optional)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called with the mark when it is clicked, tapped or Enter is pressed on it.
    #[prop(optional)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let fmag = move |v: f64| magnitude_format.map_or_else(|| format!("{v:.2}"), |f| f.run(v));
    let layout = Memo::new(move |_| field.with(|f| quiver::layout(f, scaled, streamlines)));
    let big = Memo::new(move |_| layout.with(|l| l.arrows.len() > raster::DOM_LIMIT));
    let grid = Memo::new(move |_| {
        layout.with(|l| {
            let f = field.get_untracked();
            (
                f.xs.iter().map(|&x| l.x_axis.scale.map(x)).collect::<Vec<f64>>(),
                f.ys.iter().map(|&y| l.axis.scale.map(y)).collect::<Vec<f64>>(),
            )
        })
    });
    // (row from the top, column), so Up and Down move the way they look.
    let active = RwSignal::new(None::<(usize, usize)>);
    let arrow =
        move |(top, col): (usize, usize)| layout.with(|l| l.arrow(l.shape.0.checked_sub(top + 1)?, col).copied());

    let datum = Signal::derive(move || {
        let at = active.get()?;
        let a = arrow(at)?;
        Some(EventDatum {
            series: at.0,
            index: at.1,
            label: Some(format!("({}, {})", fmt(a.x), fmt(a.y))),
            value: a.magnitude,
            node: None,
        })
    });
    let tip = Signal::derive(move || {
        let a = arrow(active.get()?)?;
        let rows = vec![
            TipRow { active: true, ..TipRow::new(Some(ramp_colour(a.level)), "Magnitude", fmag(a.magnitude)) },
            TipRow::new(None, "Direction", format!("{:.0}°", a.degrees())),
            TipRow::new(None, "Components", format!("{}, {}", fmag(a.u), fmag(a.v))),
        ];
        Some(Tip { x: a.centre().0, y: a.centre().1, title: format!("({}, {})", fmt(a.x), fmt(a.y)), rows })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, y)| {
            grid.with(|(xs, ys)| {
                let col = nearest_index(xs, x)?;
                let row = nearest_index(ys, y)?;
                Some((ys.len() - 1 - row, col))
            })
        });
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| {
        let (rows, cols) = layout.with(|l| l.shape);
        grid_key(&key, active.get_untracked(), rows, cols).map(|next| active.set(next)).is_some()
    });
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(&fmt)));
    let labels = Signal::derive(move || layout.with(|l| l.x_axis.ticks(&fmt)));
    let head = Signal::derive(|| ["x", "y", "u", "v", "Magnitude", "Direction"].map(String::from).to_vec());
    let row_of = move |a: &quiver::Arrow| {
        vec![fmt(a.x), fmt(a.y), fmag(a.u), fmag(a.v), fmag(a.magnitude), format!("{:.0}°", a.degrees())]
    };
    let rows = Signal::derive(move || {
        layout.with(|l| l.arrows.iter().take(raster::table_rows(l.arrows.len(), 6)).map(row_of).collect())
    });
    provide_export(move || {
        let head = ["x", "y", "u", "v", "Magnitude", "Direction"].map(String::from).to_vec();
        std::iter::once(head)
            .chain(layout.with_untracked(|l| l.arrows.iter().map(row_of).collect::<Vec<_>>()))
            .collect()
    });
    let ramp =
        Signal::derive(|| (0..RAMP_STEPS).map(|i| ramp_colour(i as f64 / (RAMP_STEPS - 1) as f64)).collect::<Vec<_>>());
    let draw = Callback::new(move |f: Frame| layout.with(|l| quiver::paint(l, &f.colours, f.width, f.height, f.dpr)));
    let marks = move || {
        let l = layout.get();
        let lines = (!l.streamlines.is_empty()).then(|| {
            let paths = l.streamlines.iter().map(|d| view! { <path data-part="streamline" d=d.clone()></path> }).collect_view();
            view! { <svg data-part="streamlines" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">{paths}</svg> }
        });
        let arrows = if big.get() {
            view! { <RasterCanvas colours=ramp draw /> }.into_any()
        } else {
            let list = l
                .arrows
                .iter()
                .map(|a| {
                    let at = (l.shape.0 - 1 - a.row, a.col);
                    let style = format!(
                        "--c: {:.4}; --v: {:.4}; --u: {:.5}; --w: {:.5}; --i: {}; --col: {};",
                        a.px,
                        a.py,
                        a.dx,
                        a.dy,
                        a.row + a.col,
                        ramp_colour(a.level)
                    );
                    view! { <span data-part="arrow" data-still=(a.magnitude == 0.0).then_some("") data-active=move || (active.get() == Some(at)).then_some("") style=style></span> }
                })
                .collect_view();
            view! { <div data-part="arrows" aria-hidden="true">{list}</div> }.into_any()
        };
        view! { {lines}{arrows} }
    };
    let overlay = move || {
        if !big.get() {
            return None;
        }
        let a = arrow(active.get()?)?;
        Some(
            view! { <span data-part="dot" style=format!("--c: {:.4}; --v: {:.4}; background: {};", a.centre().0, a.centre().1, ramp_colour(a.level))></span> },
        )
    };

    view! {
        <Plot
            scope="quiver-chart"
            label
            hint="Use the arrow keys to move across the grid."
            height
            ticks
            labels
            annotations=Signal::derive(Vec::new)
            tip
            head
            rows
            on_key
            on_point
            marks
            overlay
            datum=Some(datum)
            on_hover
            on_select
        />
        <div data-scope="quiver-chart" data-part="scale" aria-hidden="true">
            <span data-part="scale-name">"Magnitude"</span>
            <span data-part="scale-value">{move || fmag(layout.with(|l| l.range.0))}</span>
            <span data-part="ramp"></span>
            <span data-part="scale-value">{move || fmag(layout.with(|l| l.range.1))}</span>
        </div>
    }
}
