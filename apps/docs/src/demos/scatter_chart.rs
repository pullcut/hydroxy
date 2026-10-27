use leptos::prelude::*;
use oxide_leptos::chart_frame::ChartFrame;
use oxide_leptos::scatter_chart::*;

pub fn demo() -> AnyView {
    // Two groups of plain dots: hover for the nearest, or focus and use the arrow keys.
    let teams = vec![
        Group::new(
            "Research",
            vec![
                Point::new(4.0, 62.0),
                Point::new(6.0, 68.0),
                Point::new(7.0, 71.0),
                Point::new(9.0, 77.0),
                Point::new(12.0, 80.0),
                Point::new(14.0, 86.0),
            ],
        ),
        Group::new(
            "Support",
            vec![
                Point::new(3.0, 48.0),
                Point::new(5.0, 55.0),
                Point::new(8.0, 52.0),
                Point::new(10.0, 63.0),
                Point::new(13.0, 66.0),
                Point::new(15.0, 70.0),
            ],
        ),
    ];
    // A third measure becomes bubble area.
    let accounts = vec![Group::new(
        "Accounts",
        vec![
            Point::sized(12.0, 3_400.0, 8.0),
            Point::sized(28.0, 5_900.0, 34.0),
            Point::sized(41.0, 4_100.0, 15.0),
            Point::sized(55.0, 9_800.0, 62.0),
            Point::sized(63.0, 7_200.0, 24.0),
            Point::sized(80.0, 12_600.0, 90.0),
        ],
    )];

    // Revenue spans five orders of magnitude, so both axes are logarithmic.
    let companies = vec![
        Group::new(
            "Software",
            vec![
                Point::new(12.0, 1_400_000.0),
                Point::new(48.0, 6_200_000.0),
                Point::new(230.0, 41_000_000.0),
                Point::new(1_150.0, 260_000_000.0),
                Point::new(5_400.0, 1_900_000_000.0),
            ],
        ),
        Group::new(
            "Services",
            vec![
                Point::new(8.0, 650_000.0),
                Point::new(35.0, 2_800_000.0),
                Point::new(160.0, 14_000_000.0),
                Point::new(900.0, 82_000_000.0),
                Point::new(3_800.0, 410_000_000.0),
            ],
        ),
    ];
    let threshold = vec![Annotation::y_line(100_000_000.0, "£100m")];

    view! {
        <div style="display: grid; gap: var(--space-8);">
            // Scroll or pinch to zoom, drag to pan, or focus the plot and press + and −.
            <ChartFrame title="Revenue by company size" description="Both axes are logarithmic. Zoom in on a cluster.">
                <ScatterChart
                    groups=companies
                    label="Revenue against headcount"
                    x_label="Employees"
                    y_label="Revenue"
                    x_scale=ScaleKind::Log
                    y_scale=ScaleKind::Log
                    zoom=true
                    annotations=threshold
                    format_y=money
                />
            </ChartFrame>
            <ChartFrame title="Interviews against satisfaction" description="One dot per person, by how many interviews they read.">
                <ScatterChart
                    groups=teams
                    label="Interviews read against satisfaction score"
                    x_label="Interviews"
                    y_label="Satisfaction"
                />
            </ChartFrame>
            <ChartFrame title="Revenue by account" description="Bubble area is seats.">
                <ScatterChart
                    groups=accounts
                    label="Revenue against tenure, sized by seats"
                    x_label="Months"
                    y_label="Revenue"
                    size_label="Seats"
                    height=240
                    format_y=|v: f64| format!("£{:.0}k", v / 1000.0)
                />
            </ChartFrame>
        </div>
    }
    .into_any()
}

/// Short money labels for a log axis: £650k, £14m, £2bn.
fn money(v: f64) -> String {
    if v >= 1e9 {
        format!("£{:.0}bn", v / 1e9)
    } else if v >= 1e6 {
        format!("£{:.0}m", v / 1e6)
    } else {
        format!("£{:.0}k", v / 1e3)
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "scatter-chart",
    name: "Scatter chart",
    group: "Charts",
    summary: "Two numeric measures, one dot per observation. Colour by group, or give points a size for a bubble chart. The nearest dot is picked out under the pointer or the arrow keys.",
    demo,
    leptos: include_str!("scatter_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/scatter_chart.rs"),
    parts: &[
        &["ScatterChart", "root", "figure with a screen-reader table of every point"],
        &["", "gridline", "Recessive gridlines on both axes at nice ticks"],
        &["", "overlay", "Focusable plot that finds the nearest dot"],
        &["", "dot", "One per point, in the group colour; bubbles scale by area"],
        &["", "tooltip", "Group, x, y and size of the active dot; flips near the edges"],
        &["", "legend", "Group names, shown for two or more groups"],
        &["", "crosshair", "Dashed rules through the active dot on both axes"],
        &["", "annotation", "Reference line, band or labelled point, in data units"],
        &["", "zoom-reset", "\"Reset zoom\" button, shown while zoomed"],
    ],
    keys: &[
        &["ArrowRight / ArrowUp", "Next dot, left to right"],
        &["ArrowLeft / ArrowDown", "Previous dot"],
        &["Home / End", "First or last dot"],
        &["Escape", "Clear the active dot"],
        &["+ / -", "With zoom: zoom in or out around the middle"],
        &["Shift + arrows", "With zoom: pan"],
        &["0", "With zoom: reset"],
    ],
    props: &[
        &["groups", "Vec<Group>", "Required. Group::new(name, points); Point::new(x, y) or Point::sized(x, y, size)"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["x_label / y_label", "String", "Axis captions, also used in the tooltip"],
        &["size_label", "String", "Name of the bubble measure. Default \"Size\""],
        &["height", "u16", "Plot height in px. Default 280"],
        &["format_x / format_y", "Fn(f64) -> String", "Format ticks, tooltip and table"],
        &["x_scale / y_scale", "ScaleKind", "Linear (default), Log or Symlog, per axis"],
        &["annotations", "Vec<Annotation>", "Reference lines, bands and labelled points, in data units"],
        &["zoom", "bool", "Wheel or pinch to zoom both axes, drag to pan, + / - and Shift+arrows on the focused plot"],
    ],
};
