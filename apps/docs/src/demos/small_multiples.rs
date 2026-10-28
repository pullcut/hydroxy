use leptos::prelude::*;
use oxide_leptos::chart_frame::ChartFrame;
use oxide_leptos::small_multiples::*;

pub fn demo() -> AnyView {
    let weeks: Vec<String> = (1..=12).map(|w| format!("W{w}")).collect();
    // Weekly signups by region, organic against paid. Every panel shares one value axis.
    let regions = [
        ("London", [410.0, 432.0, 451.0, 470.0, 466.0, 498.0, 521.0, 540.0, 552.0, 571.0, 590.0, 612.0], [220.0, 240.0, 236.0, 261.0, 280.0, 276.0, 301.0, 318.0, 309.0, 335.0, 342.0, 360.0]),
        ("Manchester", [180.0, 186.0, 195.0, 201.0, 214.0, 209.0, 226.0, 238.0, 244.0, 252.0, 265.0, 271.0], [90.0, 96.0, 104.0, 99.0, 112.0, 118.0, 121.0, 130.0, 128.0, 137.0, 141.0, 150.0]),
        ("Edinburgh", [120.0, 118.0, 126.0, 131.0, 129.0, 140.0, 146.0, 151.0, 149.0, 158.0, 162.0, 170.0], [40.0, 44.0, 48.0, 46.0, 52.0, 55.0, 58.0, 61.0, 60.0, 66.0, 70.0, 72.0]),
        ("Bristol", [140.0, 146.0, 151.0, 160.0, 158.0, 167.0, 171.0, 180.0, 186.0, 184.0, 193.0, 201.0], [70.0, 68.0, 75.0, 81.0, 79.0, 86.0, 92.0, 90.0, 97.0, 101.0, 99.0, 108.0]),
        ("Leeds", [150.0, 149.0, 155.0, 162.0, 170.0, 168.0, 176.0, 181.0, 189.0, 196.0, 194.0, 205.0], [60.0, 66.0, 63.0, 70.0, 74.0, 79.0, 77.0, 85.0, 88.0, 86.0, 93.0, 97.0]),
        ("Cardiff", [80.0, 84.0, 83.0, 90.0, 94.0, 92.0, 99.0, 103.0, 101.0, 108.0, 112.0, 115.0], [30.0, 33.0, 31.0, 36.0, 38.0, 41.0, 40.0, 44.0, 47.0, 46.0, 50.0, 53.0]),
    ];
    let signups: Vec<Facet> = regions
        .iter()
        .map(|(name, organic, paid)| Facet::new(*name, vec![Series::new("Organic", *organic), Series::new("Paid", *paid)]))
        .collect();

    let months: Vec<String> = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"].map(String::from).to_vec();
    let rainfall = vec![
        Facet::new("Glasgow", vec![Series::new("Rainfall", [128.0, 92.0, 98.0, 61.0, 63.0, 62.0, 69.0, 86.0, 99.0, 125.0, 119.0, 124.0])]),
        Facet::new("London", vec![Series::new("Rainfall", [55.0, 41.0, 41.0, 44.0, 49.0, 45.0, 45.0, 50.0, 49.0, 69.0, 59.0, 55.0])]),
        Facet::new("Madrid", vec![Series::new("Rainfall", [33.0, 34.0, 25.0, 46.0, 50.0, 19.0, 10.0, 9.0, 21.0, 59.0, 52.0, 43.0])]),
        Facet::new("Mumbai", vec![Series::new("Rainfall", [1.0, 0.0, 0.0, 1.0, 11.0, 538.0, 841.0, 514.0, 319.0, 55.0, 15.0, 4.0])]),
    ];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Weekly signups by region" description="Organic and paid signups this quarter. Every panel shares one axis, so London's lead reads at a glance." export=true>
                <SmallMultiples facets=signups labels=weeks.clone() label="Weekly signups by region" />
            </ChartFrame>
            <ChartFrame title="Monthly rainfall" description="Millimetres per month. On a shared axis the monsoon dwarfs a Scottish winter.">
                <SmallMultiples facets=rainfall labels=months label="Average monthly rainfall by city" kind=Kind::Bar height=120 min_width=200 format=|v: f64| format!("{v:.0} mm") />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "small-multiples",
    name: "Small multiples",
    group: "Charts",
    summary: "The same chart repeated for each facet, such as a region or product, in a grid that wraps to fit. Every panel shares one value axis so heights compare across panels, the series keep their colours everywhere, and one legend names them. Panels enter one after another.",
    demo,
    leptos: include_str!("small_multiples.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/small_multiples.rs"),
    parts: &[
        &["SmallMultiples", "root", "group named by label, holding the legend and grid"],
        &["", "legend", "Series names and colours, once for the whole grid (two or more series)"],
        &["", "grid", "Wraps panels to the container; none narrower than min_width"],
        &["", "facet", "One panel; --facet staggers its entrance"],
        &["", "facet-title", "The facet's name above its chart"],
    ],
    keys: &[&["Tab", "Moves between panels; each chart keeps its own arrow keys"]],
    props: &[
        &["facets", "Vec<Facet>", "Required. Facet::new(title, series); every facet lists the same series in the same order"],
        &["labels", "Vec<String>", "Required. One label per x position, shared by every facet"],
        &["label", "String", "Required. What the grid shows; each panel is named label plus its title"],
        &["kind", "Kind", "Line (default), Area, StackedArea or Bar (each facet's first series)"],
        &["height", "u16", "Plot height of each panel in px. Default 140"],
        &["min_width", "u16", "Narrowest panel before the grid wraps, in px. Default 240"],
        &["independent", "bool", "Give each facet its own value axis instead of the shared one"],
        &["format", "Fn(f64) -> String", "Formats axes, tooltips and tables"],
        &["y_domain", "(f64, f64)", "On LineChart, BarChart and StackedAreaChart: a range the value axis must cover. Small multiples pass the shared one"],
    ],
};
