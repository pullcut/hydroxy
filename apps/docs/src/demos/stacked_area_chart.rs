use leptos::prelude::*;
use oxide_leptos::chart_frame::ChartFrame;
use oxide_leptos::stacked_area_chart::*;

pub fn demo() -> AnyView {
    // Stored signals are Copy, so the three charts can share them.
    let weeks = Signal::stored(["Jan 6", "Jan 13", "Jan 20", "Jan 27", "Feb 3", "Feb 10", "Feb 17", "Feb 24", "Mar 3", "Mar 10", "Mar 17", "Mar 24"]
        .map(String::from)
        .to_vec());
    // Weekly visits in thousands, bottom of the stack first.
    let traffic = Signal::stored(vec![
        Series::new("Organic search", [42.0, 44.0, 47.0, 45.0, 49.0, 52.0, 55.0, 53.0, 57.0, 60.0, 62.0, 65.0]),
        Series::new("Paid search", [18.0, 20.0, 22.0, 25.0, 24.0, 21.0, 19.0, 23.0, 26.0, 28.0, 27.0, 30.0]),
        Series::new("Referral", [9.0, 9.0, 10.0, 12.0, 11.0, 10.0, 13.0, 14.0, 13.0, 15.0, 16.0, 15.0]),
        Series::new("Social", [6.0, 8.0, 7.0, 9.0, 14.0, 18.0, 12.0, 10.0, 9.0, 11.0, 10.0, 12.0]),
        Series::new("Email", [4.0, 12.0, 5.0, 4.0, 13.0, 5.0, 4.0, 14.0, 5.0, 4.0, 15.0, 6.0]),
    ]);
    let thousands = |v: f64| format!("{v:.0}k");

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Visits by source" description="Weekly visits, stacked from zero. The dashed line is the quarter's weekly goal.">
                <StackedAreaChart
                    series=traffic
                    labels=weeks
                    label="Weekly visits by traffic source"
                    curve=Curve::Monotone
                    format=thousands
                    annotations=vec![Annotation::y_line(120.0, "Goal")]
                />
            </ChartFrame>
            <ChartFrame title="Share of visits" description="The same data scaled to 100%, so each source's share reads directly.">
                <StackedAreaChart series=traffic labels=weeks label="Share of weekly visits by source" offset=StackOffset::Expand format=thousands />
            </ChartFrame>
            <ChartFrame title="Traffic streamgraph" description="A wiggle baseline keeps each ribbon as flat as it can. Values are relative, so there is no axis.">
                <StackedAreaChart series=traffic labels=weeks label="Weekly visits by source, as a streamgraph" offset=StackOffset::Wiggle curve=Curve::Monotone format=thousands />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "stacked-area-chart",
    name: "Stacked area chart",
    group: "Lines and areas",
    summary: "Parts of a total over time, stacked from zero, scaled to 100%, or as a streamgraph. Hover to read every series at one point.",
    demo,
    leptos: include_str!("stacked_area_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/stacked_area_chart.rs"),
    parts: &[
        &["StackedAreaChart", "root", "figure with data-variant set to the offset, and a screen-reader table of every value"],
        &["", "area", "One ribbon per series; the hovered one stays strong while the rest fade"],
        &["", "crosshair", "Vertical line at the active x"],
        &["", "dot", "The top of each ribbon at the active x"],
        &["", "tooltip", "Every series at that x, top of the stack first, and the total"],
        &["", "legend", "Shown for two or more series"],
        &["", "annotation", "Reference lines, bands and points"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next x"],
        &["Home / End", "First or last x"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["series", "Vec<Series>", "Required. Bottom of the stack first; past six, the smallest fold into Other"],
        &["labels", "Vec<String>", "Required. One per x position"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["offset", "StackOffset", "Zero (default), Expand, Wiggle or Silhouette"],
        &["curve", "Curve", "Linear (default) or Monotone"],
        &["height", "u16", "Plot height in px. Default 240"],
        &["format", "Fn(f64) -> String", "Formats the ticks, tooltip and table"],
        &["annotations", "Vec<Annotation>", "x is a point index; with Expand, y is a share 0..=1"],
    ],
};
