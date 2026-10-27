use leptos::prelude::*;
use oxide_leptos::metric_card::MetricCard;

pub fn demo() -> AnyView {
    let weeks: Vec<String> = (1..=12).map(|w| format!("Week {w}")).collect();
    let revenue = vec![38.2, 39.0, 40.1, 39.4, 41.8, 42.5, 43.0, 42.2, 44.9, 46.1, 45.3, 48.4];
    let latency = vec![212.0, 205.0, 220.0, 198.0, 190.0, 186.0, 192.0, 181.0, 176.0, 179.0, 171.0, 168.0];
    let errors = vec![0.42, 0.38, 0.40, 0.44, 0.51, 0.48, 0.55, 0.61, 0.58, 0.66, 0.63, 0.71];

    view! {
        <div style="display: grid; gap: var(--space-4); grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));">
            <MetricCard
                label="Weekly revenue"
                value=48.4
                previous=45.3
                comparison="vs last week"
                trend=revenue
                trend_labels=weeks.clone()
                format=|v: f64| format!("${v:.1}k")
            />
            <MetricCard
                label="p95 latency"
                value=168.0
                previous=171.0
                unit="ms"
                comparison="vs last week"
                higher_is_better=false
                trend=latency
                trend_labels=weeks.clone()
            />
            <MetricCard
                label="Error rate"
                value=0.71
                previous=0.63
                unit="%"
                comparison="vs last week"
                higher_is_better=false
                trend=errors
                trend_labels=weeks
                format=|v: f64| format!("{v:.2}")
            />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "metric-card",
    name: "Metric card",
    group: "Charts",
    summary: "One KPI with its change against the previous value and a trend sparkline. The change shows as a sign, an arrow and a status colour together, and higher_is_better flips the colours for measures like latency.",
    demo,
    leptos: include_str!("metric_card.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/metric_card.rs"),
    parts: &[
        &["MetricCard", "root", "card with a screen-reader table of the figures and trend"],
        &["", "figure", "The value and unit"],
        &["", "delta", "Arrow, signed change and comparison; data-tone is good, bad or neutral"],
        &["", "spark", "Focusable sparkline; arrow keys step through points"],
        &["", "tooltip", "Point name and value under the pointer or focus"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Next or previous point on the sparkline"],
        &["Home / End", "First or last point"],
        &["Escape", "Clear the active point"],
    ],
    props: &[
        &["label", "String", "Required. What the figure measures"],
        &["value", "f64", "Required. The current value"],
        &["previous", "f64", "Value to compare against; shows the change"],
        &["comparison", "String", "What the change is against, such as \"vs last week\""],
        &["unit", "String", "Unit after the figure"],
        &["higher_is_better", "bool", "Whether a rise is good news. Default true"],
        &["trend / trend_labels", "Vec<f64> / Vec<String>", "Sparkline values, oldest first, and their names"],
        &["format", "Fn(f64) -> String", "Formats the figure, tooltip and table"],
    ],
};
