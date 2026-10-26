use leptos::prelude::*;
use oxide_leptos::line_chart::*;

pub fn demo() -> AnyView {
    let weeks: Vec<String> = ["5 Aug", "12 Aug", "19 Aug", "26 Aug", "2 Sep", "9 Sep", "16 Sep", "23 Sep", "30 Sep", "7 Oct", "14 Oct", "21 Oct"]
        .map(String::from)
        .to_vec();
    let users = vec![
        Series::new("This year", [412.0, 438.0, 431.0, 470.0, 502.0, 495.0, 540.0, 566.0, 551.0, 598.0, 627.0, 654.0]),
        Series::new("Last year", [380.0, 371.0, 395.0, 402.0, 398.0, 421.0, 433.0, 429.0, 447.0, 452.0, 470.0, 468.0]),
    ];
    let months: Vec<String> = ["May", "Jun", "Jul", "Aug", "Sep", "Oct"].map(String::from).to_vec();
    // A missing month (NaN) leaves a gap rather than a made-up value.
    let revenue = vec![Series::new("Revenue", [18_400.0, 21_900.0, f64::NAN, 24_300.0, 27_800.0, 31_200.0])];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            // Hover, or focus the plot and press Left/Right, to read both series at once.
            <LineChart series=users labels=weeks label="Weekly active users" />

            // A smooth filled area with money formatting.
            <AreaChart
                series=revenue
                labels=months
                label="Monthly revenue"
                height=180
                curve=Curve::Monotone
                format=|v: f64| format!("£{:.0}k", v / 1000.0)
            />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "line-chart",
    name: "Line chart",
    group: "Data",
    summary: "Show how one or more measures move over time. A crosshair reads every series at the same point, by pointer or keyboard. AreaChart fills under each line.",
    demo,
    leptos: include_str!("line_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/line_chart.rs"),
    parts: &[
        &["LineChart", "root", "figure with a screen-reader table of every value"],
        &["", "gridline", "Recessive gridlines at nice round ticks, always including zero"],
        &["", "line", "2px stroke in the series colour; gaps where values are missing"],
        &["", "area", "Fill under the line at low opacity (AreaChart, or area=true)"],
        &["", "overlay", "Focusable plot that drives the crosshair"],
        &["", "crosshair", "Vertical rule at the active x position, with a dot per series"],
        &["", "tooltip", "The x label and every series' value; flips side past the middle"],
        &["", "legend", "Series names and colours, shown for two or more series"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Move the crosshair one point"],
        &["Home / End", "Jump to the first or last point"],
        &["Escape", "Hide the crosshair"],
    ],
    props: &[
        &["series", "Vec<Series>", "Required. Series::new(name, values); NaN leaves a gap"],
        &["labels", "Vec<String>", "Required. One label per x position"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["x", "Vec<f64>", "Numeric x positions for uneven spacing. Default: evenly spaced"],
        &["height", "u16", "Plot height in px. Default 220"],
        &["curve", "Curve", "Linear (default) or Monotone"],
        &["area", "bool", "Fill under each line (LineChart only; AreaChart always fills)"],
        &["format", "Fn(f64) -> String", "Formats ticks, tooltip and table"],
    ],
};
