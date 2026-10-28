use leptos::prelude::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::raincloud_chart::*;

/// `n` page load times in ms around `median`, skewed right like real loads. A fixed
/// generator, so the sample is the same on every render.
fn loads(seed: u32, n: usize, median: f64) -> Vec<f64> {
    let mut s = seed;
    (0..n)
        .map(|_| {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let u = f64::from(s >> 8) / f64::from(1u32 << 24);
            (median * (0.6 + 1.3 * u * u)).round()
        })
        .collect()
}

fn regions() -> Vec<Group> {
    vec![
        Group::new("Europe", loads(7, 60, 820.0)),
        Group::new("North America", loads(11, 60, 760.0)),
        Group::new("Asia Pacific", loads(23, 48, 1080.0)),
        Group::new("South America", loads(41, 36, 1290.0)),
    ]
}

pub fn demo() -> AnyView {
    view! {
        <ChartFrame title="Page load time by region" description="Last week's checkout page loads: the shape, the quartiles and every visit.">
            <RaincloudChart
                groups=regions()
                orientation=Orientation::Horizontal
                label="Checkout page load time by region"
                annotations=vec![Annotation::x_line(1000.0, "1 s budget")]
                format=|v: f64| format!("{v:.0} ms")
            />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "raincloud-chart",
    name: "Raincloud plot",
    group: "Distributions",
    summary: "Each group's distribution three ways at once: a half violin for its shape, a slim box plot for its quartiles, and every observation jittered beside them. Standing up or lying down.",
    demo,
    leptos: include_str!("raincloud_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/raincloud_chart.rs"),
    parts: &[
        &["RaincloudChart", "root", "figure"],
        &["", "rain", "All the marks; `data-orientation` is vertical or horizontal, `data-active` while a group is picked out"],
        &["", "cloud", "Half violin: kernel density trimmed to the data, revealed from its baseline"],
        &["", "box-plot", "Whisker, box and median for one group; opens out from the median"],
        &["", "dot", "One observation, jittered across a strip; a canvas past 2,000 observations"],
        &["", "overlay", "Focusable layer that reads the pointer and keys"],
        &["", "tooltip", "Count and quartiles of the active group"],
        &["", "legend", "Group names, for two to six groups"],
        &["", "sr-table", "Every observation (up to 1,000 rows; CSV export gets them all)"],
    ],
    keys: &[
        &["Arrow keys", "Move between groups"],
        &["Home / End", "First or last group"],
        &["Escape", "Clear the active group"],
    ],
    props: &[
        &["groups", "Vec<Group>", "Required. Group::new(label, values)"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["orientation", "Orientation", "Vertical (default): groups along the bottom. Horizontal: stacked rows"],
        &["bandwidth", "f64", "Kernel bandwidth in data units. Default: Silverman's rule per group"],
        &["annotations", "Vec<Annotation>", "Reference lines and bands along the value axis"],
        &["height", "u16", "Plot height in px. Default 280"],
        &["format", "Fn(f64) -> String", "Formats values on the axis, tooltip and table"],
    ],
};
