use leptos::prelude::*;
use oxide_leptos::chart_frame::ChartFrame;
use oxide_leptos::composed_chart::*;

pub fn demo() -> AnyView {
    let months: Vec<String> = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"].map(String::from).to_vec();
    // Orders and returns as bars, the rolling average as a line, and promotions as points.
    let layers = vec![
        Layer::bar("Orders", [312.0, 288.0, 341.0, 365.0, 402.0, 389.0, 420.0, 455.0, 431.0, 470.0, 512.0, 598.0]),
        Layer::bar("Returns", [-24.0, -19.0, -28.0, -31.0, -26.0, -35.0, -30.0, -38.0, -33.0, -41.0, -45.0, -62.0]),
        Layer::line("3-month average", [f64::NAN, f64::NAN, 314.0, 331.0, 369.0, 385.0, 404.0, 421.0, 435.0, 452.0, 471.0, 527.0]),
        Layer::points("Promotion weeks", [f64::NAN, f64::NAN, 341.0, f64::NAN, f64::NAN, f64::NAN, 420.0, f64::NAN, f64::NAN, f64::NAN, 512.0, 598.0]),
    ];
    let quarters: Vec<String> = ["Q1", "Q2", "Q3", "Q4"].map(String::from).to_vec();
    let traffic = vec![
        Layer::area("Visits (k)", [182.0, 214.0, 236.0, 271.0]),
        Layer::bar("Signups", [96.0, 121.0, 118.0, 154.0]),
    ];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            // Hover, or focus the plot and press Left/Right, to read every layer at once.
            <ChartFrame title="Orders" description="Monthly orders and returns, with the rolling average and promotion months." export=true>
                <ComposedChart
                    layers=layers
                    labels=months
                    label="Orders, returns and the rolling average by month"
                    curve=Curve::Monotone
                    annotations=vec![Annotation::y_line(450.0, "Target"), Annotation::x_band(10.0, 11.0, "Peak season")]
                />
            </ChartFrame>
            <ChartFrame title="Traffic and signups" description="An area and bars on one axis.">
                <ComposedChart layers=traffic labels=quarters label="Visits and signups by quarter" height=200 />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "composed-chart",
    name: "Composed chart",
    group: "Charts",
    summary: "Layer bars, lines, areas and points over the same categories and one value axis, such as orders with a rolling average. A crosshair reads every layer at once.",
    demo,
    leptos: include_str!("composed_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/composed_chart.rs"),
    parts: &[
        &["ComposedChart", "root", "figure with a screen-reader table of every layer"],
        &["", "gridline", "Recessive gridlines at nice round ticks, always including zero"],
        &["", "bar", "Bar layers sit side by side in each category; negative bars hang below zero"],
        &["", "line", "Line and area layers run through the category centres"],
        &["", "point", "A dot per value of a points layer"],
        &["", "overlay", "Focusable plot that drives the crosshair"],
        &["", "tooltip", "The category and every layer's value"],
        &["", "legend", "Layer names, with swatches shaped like each layer"],
        &["", "annotation", "Reference line, band or labelled point; x is a category index"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Move the crosshair one category"],
        &["Home / End", "Jump to the first or last category"],
        &["Escape", "Hide the crosshair"],
    ],
    props: &[
        &["layers", "Vec<Layer>", "Required. Layer::bar, Layer::line, Layer::area or Layer::points(name, values); NaN leaves a gap"],
        &["labels", "Vec<String>", "Required. One label per category"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["height", "u16", "Plot height in px. Default 240"],
        &["curve", "Curve", "Linear (default) or Monotone, for line and area layers"],
        &["format", "Fn(f64) -> String", "Formats ticks, tooltip and table"],
        &["annotations", "Vec<Annotation>", "Reference lines, bands and labelled points; x is a category index"],
    ],
};
