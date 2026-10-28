use leptos::prelude::*;
use oxide_leptos::chart_frame::ChartFrame;
use oxide_leptos::range_area_chart::*;

pub fn demo() -> AnyView {
    let lows = [
        13.2, 12.8, 14.1, 15.0, 14.6, 13.9, 12.4, 11.8, 12.9, 14.3, 15.8, 16.4, 17.1, 16.2, 14.8, 13.5,
        13.1, 14.0, 15.2, 16.9, 18.3, 19.0, 17.6, 15.4, 14.2, 13.8, 14.5, 15.1, 14.0, 13.3, 12.9,
    ];
    let highs = [
        21.4, 22.9, 24.6, 25.1, 23.8, 21.2, 19.6, 20.4, 22.8, 25.3, 27.9, 29.4, 30.2, 28.1, 24.7, 22.0,
        21.8, 23.5, 26.1, 28.8, 31.6, 32.4, 29.7, 25.2, 23.0, 22.4, 24.1, 25.6, 23.3, 21.9, 21.2,
    ];
    let july: Vec<Interval> = (0..31).map(|d| Interval::new(format!("{} Jul", d + 1), f64::midpoint(lows[d], highs[d]), lows[d], highs[d])).collect();
    // No middle value: a NaN mid draws just the band.
    let price = [
        (42.0, 61.0), (39.0, 58.0), (37.0, 55.0), (36.0, 54.0), (38.0, 57.0), (45.0, 72.0), (58.0, 96.0), (71.0, 118.0),
        (66.0, 104.0), (55.0, 88.0), (48.0, 79.0), (44.0, 74.0), (41.0, 70.0), (40.0, 69.0), (43.0, 75.0), (52.0, 89.0),
        (68.0, 121.0), (84.0, 142.0), (79.0, 131.0), (66.0, 109.0), (57.0, 92.0), (51.0, 80.0), (47.0, 71.0), (44.0, 65.0),
    ];
    let hours: Vec<Interval> = price.iter().enumerate().map(|(h, &(lo, hi))| Interval::new(format!("{h:02}:00"), f64::NAN, lo, hi)).collect();

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Daily temperature" description="Overnight low to daytime high for each day of July, with the daily mean.">
                <RangeAreaChart data=july label="Daily low and high temperature in July" curve=Curve::Monotone format=|v: f64| format!("{v:.0}°C") />
            </ChartFrame>
            <ChartFrame title="Wholesale power price" description="Lowest and highest half-hourly price in each hour, last 30 days.">
                <RangeAreaChart data=hours label="Wholesale electricity price range by hour" range_label="Price range" height=200 format=|v: f64| format!("£{v:.0}") />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "range-area-chart",
    name: "Range area chart",
    group: "Charts",
    summary: "A band from each point's low to its high, such as daily low and high temperatures, with an optional line through the middle. The spread is the story, so the band leads.",
    demo,
    leptos: include_str!("range_area_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/range_area_chart.rs"),
    parts: &[
        &["RangeAreaChart", "root", "figure with a screen-reader table of lows, highs and spreads"],
        &["", "band", "The low-to-high band, revealed left to right"],
        &["", "line", "The middle value, drawn along its length"],
        &["", "span", "The hovered point's range"],
        &["", "dot", "The hovered point's high, low and middle (data-kind)"],
        &["", "tooltip", "High, low, middle and spread"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next point"],
        &["Home / End", "First or last point"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["data", "Vec<Interval>", "Required. Interval::new(label, mid, low, high); a NaN mid draws no line"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["range_label", "String", "Names the band. Default \"Range\""],
        &["mid_label", "String", "Names the middle line. Default \"Mean\""],
        &["height", "u16", "Plot height in px. Default 240"],
        &["curve", "Curve", "Linear (default) or Monotone"],
        &["scale", "ScaleKind", "Linear (default), Log or Symlog"],
        &["format", "Fn(f64) -> String", "Formats ticks, tooltip and table"],
        &["annotations", "Vec<Annotation>", "Reference lines, bands and points; x is a point index"],
    ],
};
