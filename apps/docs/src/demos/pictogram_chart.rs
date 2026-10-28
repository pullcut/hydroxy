use leptos::prelude::*;
use oxide_leptos::chart_frame::ChartFrame;
use oxide_leptos::pictogram_chart::*;

/// A house, drawn in a 24×24 box.
const HOUSE: &str = "M12 3.2l8.5 7.3V20a1 1 0 0 1-1 1h-5v-6h-5v6h-5a1 1 0 0 1-1-1v-9.5z";

pub fn demo() -> AnyView {
    let visitors = vec![
        Datum::new("Science museum", 18_400.0),
        Datum::new("Botanic garden", 12_750.0),
        Datum::new("City gallery", 9_300.0),
        Datum::new("Maritime museum", 4_150.0),
    ];
    let homes = vec![
        Datum::new("North", 1_240.0),
        Datum::new("Central", 2_180.0),
        Datum::new("South", 860.0),
    ];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Weekend visitors" description="Tickets scanned on the first weekend of May.">
                <PictogramChart data=visitors label="Weekend visitors by venue" unit=1_000.0 unit_label="visitors" />
            </ChartFrame>
            <ChartFrame title="New homes" description="Completions this year, by region.">
                <PictogramChart data=homes label="New homes completed by region" unit=100.0 unit_label="homes" icon=PictogramIcon::Custom(HOUSE.into()) icon_size=22 />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "pictogram-chart",
    name: "Pictogram chart",
    group: "Bars",
    summary: "An isotype chart: each category's count drawn as a row of repeated icons, each standing for a fixed number of units, with the last icon cut to the remainder. Use the built-in person, square or circle, or pass any SVG path.",
    demo,
    leptos: include_str!("pictogram_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/pictogram_chart.rs"),
    parts: &[
        &["PictogramChart", "root", "figure; `data-active` while a row is read"],
        &["", "plot", "Focusable group holding the rows"],
        &["", "row", "Label, icons and value for one category"],
        &["", "icon", "One unit's icon; `data-partial` on the last, cut to the remainder"],
        &["", "tooltip", "Value and icon count, at the end of the row"],
        &["", "key", "What one icon stands for"],
        &["", "sr-table", "Every value and icon count, for screen readers and CSV export"],
    ],
    keys: &[
        &["ArrowUp / ArrowDown", "Previous or next row"],
        &["Home / End", "First or last row"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["data", "Vec<Datum>", "Required. One row per category; past six, the smallest fold into Other"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["unit", "f64", "What one icon stands for. Default: a nice unit that fits the largest row in columns icons"],
        &["unit_label", "String", "What is counted, for the key, such as people"],
        &["icon", "PictogramIcon", "Person (default), Square, Circle or Custom(path) in a 24×24 box"],
        &["columns", "usize", "Icons per line before a row wraps. Default 20"],
        &["icon_size", "u16", "Icon size in px. Default 20"],
        &["format", "Fn(f64) -> String", "Formats values, the key, tooltip and table"],
    ],
};
