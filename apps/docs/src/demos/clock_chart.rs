use leptos::prelude::*;
use oxide_leptos::chart_frame::ChartFrame;
use oxide_leptos::clock_chart::{ClockChart, Series};

pub fn demo() -> AnyView {
    // Support chats started in each hour, averaged over October.
    let weekdays = Series::new(
        "Weekdays",
        [3.0, 2.0, 1.0, 1.0, 2.0, 4.0, 9.0, 21.0, 38.0, 46.0, 44.0, 40.0, 35.0, 39.0, 42.0, 37.0, 31.0, 24.0, 17.0, 13.0, 11.0, 9.0, 6.0, 4.0],
    );
    let weekends = Series::new(
        "Weekends",
        [6.0, 4.0, 2.0, 1.0, 1.0, 1.0, 2.0, 5.0, 9.0, 15.0, 20.0, 24.0, 26.0, 25.0, 23.0, 22.0, 21.0, 20.0, 19.0, 18.0, 17.0, 14.0, 11.0, 8.0],
    );
    // Steps by hour on one Saturday.
    let steps = Series::new(
        "Steps",
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 120.0, 840.0, 2_310.0, 1_460.0, 980.0, 1_720.0, 640.0, 3_850.0, 4_120.0, 1_210.0, 690.0, 1_030.0, 410.0, 260.0, 180.0, 40.0, 0.0],
    );

    view! {
        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: var(--space-10);">
            <ChartFrame title="Support chats by hour" description="Chats started per hour, averaged over October. Weekdays peak mid-morning; weekends spread out.">
                <ClockChart series=vec![weekdays, weekends] label="Support chats started by hour, weekdays and weekends" />
            </ChartFrame>
            <ChartFrame title="Steps on Saturday" description="One series, hour by hour. Hover an hour or focus the clock and use the arrow keys.">
                <ClockChart series=vec![steps] label="Steps by hour on Saturday" size=260 format=|v: f64| format!("{v:.0}") />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "clock-chart",
    name: "Clock chart",
    group: "Charts",
    summary: "Activity by hour of the day as bars growing out from the centre of a 24-hour clock face, midnight at the top. The shape of a day reads at a glance, and the night wraps round to the morning instead of splitting across two ends of an axis. A second series sits beside the first in each hour.",
    demo,
    leptos: include_str!("clock_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/clock_chart.rs"),
    parts: &[
        &["ClockChart", "root", "figure with a screen-reader table of every hour"],
        &["", "face", "The clock face"],
        &["", "hour-tick", "A tick per hour on the rim, longer every six hours"],
        &["", "hour", "Hour labels every three hours"],
        &["", "ring", "Value rings at nice steps, labelled up the middle"],
        &["", "bar", "One per hour and series, growing out from the hub"],
        &["", "tooltip", "The hour and each series' value"],
        &["", "legend", "Series names and colours, shown for two series"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next hour (Up and Down too)"],
        &["Home / End", "Midnight or the last hour"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["series", "Vec<Series>", "Required. One or two series of 24 values from midnight"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["size", "u16", "Face diameter in px. Default 300"],
        &["format", "Fn(f64) -> String", "Formats the rings, tooltip and table"],
    ],
};
