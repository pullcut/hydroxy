use leptos::prelude::*;
use oxide_leptos::bar_chart::*;
use oxide_leptos::chart_frame::ChartFrame;
use oxide_leptos::segmented::{Segmented, SegmentedItem, SegmentedOption};

pub fn demo() -> AnyView {
    let weeks = vec![
        Datum::new("2 Sep", 6.0),
        Datum::new("9 Sep", 9.0),
        Datum::new("16 Sep", 7.0),
        Datum::new("23 Sep", 12.0),
        Datum::new("30 Sep", 10.0),
        Datum::new("7 Oct", 14.0),
        Datum::new("14 Oct", 11.0),
        Datum::new("21 Oct", 17.0),
    ];
    let october = vec![
        Datum::new("Research", 4200.0),
        Datum::new("Support", 2600.0),
        Datum::new("Sales", 3100.0),
        Datum::new("Product", 1800.0),
    ];
    let september = vec![
        Datum::new("Research", 3400.0),
        Datum::new("Support", 2900.0),
        Datum::new("Sales", 2200.0),
        Datum::new("Product", 2500.0),
    ];
    let sources = vec![
        Datum::new("Customer interviews", 18.0),
        Datum::new("Support tickets", 14.0),
        Datum::new("Sales call notes", 11.0),
        Datum::new("In-app survey comments", 8.0),
        Datum::new("App store reviews", 5.0),
    ];
    let (month, set_month) = signal(Some(String::from("oct")));
    // Same categories, new values: each bar glides to its new height.
    let spend = Signal::derive(move || {
        if month.get().as_deref() == Some("sep") { september.clone() } else { october.clone() }
    });

    view! {
        <div style="display: grid; gap: var(--space-10);">
            // The latest week is picked out; hover any bar for its value.
            <ChartFrame title="Interviews read" description="Per week, since the start of September.">
                <BarChart data=weeks label="Interviews read per week" />
            </ChartFrame>

            // A chosen bar, every value shown, money formatting, and live data.
            <Segmented
                label="Month"
                items=vec![SegmentedOption::new("sep"), SegmentedOption::new("oct")]
                value=month
                on_value_change=move |v| set_month.set(Some(v))
            >
                <SegmentedItem value="sep" label="September" />
                <SegmentedItem value="oct" label="October" />
            </Segmented>
            <ChartFrame title="Monthly spend" description="By team, switch the month to watch the bars glide.">
                <BarChart
                    data=spend
                    label="Monthly spend by team"
                    height=160
                    highlight=Highlight::Index(0)
                    show_values=true
                    format=|v: f64| {
                        let k = v / 1000.0;
                        if k.fract() == 0.0 { format!("£{k:.0}k") } else { format!("£{k:.1}k") }
                    }
                />
            </ChartFrame>

            // Horizontal suits long category names.
            <ChartFrame title="Interviews by source" description="Horizontal bars keep long names readable.">
                <BarChart
                    data=sources
                    label="Interviews by source"
                    orientation=Orientation::Horizontal
                    height=200
                    highlight=Highlight::Index(0)
                    show_values=true
                />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "bar-chart",
    name: "Bar chart",
    group: "Charts",
    summary: "Compare one measure across categories or weeks. Bars are grey with one picked out, so the eye goes to the value that matters.",
    demo,
    leptos: include_str!("bar_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/bar_chart.rs"),
    parts: &[
        &["BarChart", "root", "figure with a screen-reader table of every value"],
        &["", "gridline", "Recessive gridlines at nice round ticks, starting at zero"],
        &["", "column", "One per value; hovering reveals its value"],
        &["", "bar", "4px rounded top, --chart-muted, or --chart-1 when highlighted"],
        &["", "label", "Category name, truncated with an ellipsis"],
    ],
    keys: &[],
    props: &[
        &["data", "Vec<Datum>", "Required. Datum::new(label, value), in display order"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["orientation", "Orientation", "Vertical (default) or Horizontal, with labels down the left"],
        &["height", "u16", "Plot height in px. Default 180"],
        &["highlight", "Highlight", "Last (default), Index(i) or Off"],
        &["show_values", "bool", "Show every value, not just the highlighted and hovered ones"],
        &["format", "Fn(f64) -> String", "Formats ticks, values and the table"],
    ],
};
