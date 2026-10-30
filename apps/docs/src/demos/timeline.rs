use hydroxy_leptos::timeline::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    view! {
        <div style="max-width: 480px;">
            <Timeline label="Run history">
                <TimelineItem title="Run started" time="09:12" datetime="2026-10-26T09:12" actor="Scheduled" />
                <TimelineItem title="Read 14 sources" time="09:13" tone=Tone::Info status="Info">
                    "interviews-q3.pdf, tickets-sept.csv and 12 more."
                </TimelineItem>
                <TimelineItem title="Two figures disagree" time="09:15" tone=Tone::Warning status="Warning">
                    "Churn is 4.1% in the board deck and 3.8% in the export."
                </TimelineItem>
                <TimelineItem title="Upload to Drive failed" time="09:16" tone=Tone::Danger status="Failed" actor="Agent">
                    "Permission denied on /Board/2026."
                </TimelineItem>
                <TimelineItem title="Summary approved" time="09:40" tone=Tone::Success status="Done" actor="Jo Marsh" />
            </Timeline>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "timeline",
    name: "Timeline",
    group: "Activity",
    summary: "A vertical list of events, oldest first. Each has a time, a title, an optional actor and body, and a coloured status marker.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("timeline.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/timeline.rs"),
    parts: &[
        &["Timeline", "root", "<ol> with an aria-label"],
        &["TimelineItem", "item", "<li>. data-tone sets the marker colour"],
        &["TimelineItem", "marker", "Icon for the tone, or a dot for neutral and accent. Hidden from assistive tech"],
        &["TimelineItem", "title, status", "The event. status is visually hidden text that says what the colour means"],
        &["TimelineItem", "time", "<time>, with datetime when given"],
        &["TimelineItem", "actor, body", "Who did it, and details"],
    ],
    keys: &[],
    props: &[
        &["label", "String", "Timeline's accessible name. Default \"Timeline\""],
        &["title", "String", "Required. What happened"],
        &["time", "String", "Required. Display time"],
        &["datetime", "Option<String>", "Machine-readable time for <time datetime>"],
        &["tone", "Tone", "Neutral (default), Success, Warning, Danger, Info or Accent"],
        &["status", "Option<String>", "Screen reader text for the tone, such as \"Failed\""],
        &["actor", "Option<String>", "Who did it"],
        &["children", "Children", "Optional details under the title"],
    ],
    events: &[],
};
