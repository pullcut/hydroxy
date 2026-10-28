//! `AreaChart`: a [`LineChart`] with the area filled.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::zoom::View;

use super::{Annotation, Curve, LineChart, ScaleKind, Series};

/// A [`LineChart`] with the area under each line filled.
#[component]
pub fn AreaChart(
    /// The series, drawn in categorical colour order. Up to six read well.
    series: ReadSignal<Vec<Series>>,
    /// One label per x position, used on the axis, tooltip and table. Leave empty with `x`
    /// set to label points from `x` (dates with `time_axis`), which saves building a string
    /// per point for large data.
    #[props(default)]
    labels: ReadSignal<Vec<String>>,
    /// Numeric x positions (ascending), one per label, for uneven spacing. Default: evenly spaced.
    x: ReadSignal<Option<Vec<f64>>>,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[props(default = 220)]
    height: u16,
    /// Straight segments (default) or a monotone curve.
    #[props(default)]
    curve: Curve,
    /// Formats values for the axis, tooltip and table.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, as on [`LineChart`].
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Linear (default), Log or Symlog value axis.
    #[props(default)]
    scale: ScaleKind,
    /// Treat `x` as Unix seconds and label the axis with calendar ticks.
    #[props(default)]
    time_axis: bool,
    /// Wheel, pinch, drag and keyboard zoom, as on [`LineChart`].
    #[props(default)]
    zoom: bool,
    /// The visible window, to control it from outside, as on [`LineChart`].
    view: ReadSignal<Option<View>>,
    /// Called once a zoom or pan has settled, as on [`LineChart`].
    on_view_change: Option<EventHandler<View>>,
    /// Called when the active position changes, as on [`LineChart`].
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter, as on [`LineChart`].
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    rsx! {
        LineChart {
            series,
            labels,
            x,
            label,
            height,
            curve,
            area: true,
            format,
            annotations,
            scale,
            time_axis,
            zoom,
            view,
            on_view_change,
            on_hover,
            on_select,
        }
    }
}
