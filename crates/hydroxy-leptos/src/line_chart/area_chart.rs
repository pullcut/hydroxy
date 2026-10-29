//! `<AreaChart>`: a [`LineChart`] with the area filled.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::format_value;
use hydroxy_charts::zoom::View;
use leptos::prelude::*;

use super::{Annotation, Curve, LineChart, ScaleKind, Series};

/// A [`LineChart`] with the area under each line filled.
#[allow(unreachable_pub, reason = "re-exported from `line_chart`; #[component] adds props types that are not")]
#[component]
pub fn AreaChart(
    /// The series, drawn in categorical colour order. Up to six read well.
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// One label per x position, used on the axis, tooltip and table. Leave empty with `x`
    /// set to label points from `x` (dates with `time_axis`), which saves building a string
    /// per point for large data.
    #[prop(optional, into)]
    labels: Signal<Vec<String>>,
    /// Numeric x positions (ascending), one per label, for uneven spacing. Default: evenly spaced.
    #[prop(optional, into)]
    x: MaybeProp<Vec<f64>>,
    /// What the chart shows. Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[prop(default = 220)]
    height: u16,
    /// Straight segments (default) or a monotone curve.
    #[prop(optional)]
    curve: Curve,
    /// Formats values for the axis, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, as on [`LineChart`].
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Linear (default), Log or Symlog value axis.
    #[prop(optional)]
    scale: ScaleKind,
    /// Treat `x` as Unix seconds and label the axis with calendar ticks.
    #[prop(optional)]
    time_axis: bool,
    /// Wheel, pinch, drag and keyboard zoom, as on [`LineChart`].
    #[prop(optional)]
    zoom: bool,
    /// The visible window, to control it from outside, as on [`LineChart`].
    #[prop(optional, into)]
    view: MaybeProp<View>,
    /// Called once a zoom or pan has settled, as on [`LineChart`].
    #[prop(optional, into)]
    on_view_change: Option<Callback<View>>,
    /// Called when the active position changes, as on [`LineChart`].
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter, as on [`LineChart`].
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let on_view_change = on_view_change.unwrap_or_else(|| Callback::new(|_| {}));
    match (on_hover, on_select) {
        (Some(h), Some(s)) => view! { <LineChart series labels x label height curve area=true format=format.unwrap_or_else(|| Callback::new(format_value)) annotations scale time_axis zoom view on_view_change on_hover=h on_select=s /> }.into_any(),
        (Some(h), None) => view! { <LineChart series labels x label height curve area=true format=format.unwrap_or_else(|| Callback::new(format_value)) annotations scale time_axis zoom view on_view_change on_hover=h /> }.into_any(),
        (None, Some(s)) => view! { <LineChart series labels x label height curve area=true format=format.unwrap_or_else(|| Callback::new(format_value)) annotations scale time_axis zoom view on_view_change on_select=s /> }.into_any(),
        (None, None) => view! { <LineChart series labels x label height curve area=true format=format.unwrap_or_else(|| Callback::new(format_value)) annotations scale time_axis zoom view on_view_change  /> }.into_any(),
    }
}
