use dioxus::prelude::*;
use hydroxy_dioxus::timeline::{Timeline, TimelineItem, Tone};

#[component]
pub fn TimelineDemo() -> Element {
    rsx! {
        div { style: "max-width: 480px;",
            Timeline { label: "Run history",
                TimelineItem { title: "Run started", time: "09:12", datetime: "2026-10-26T09:12", actor: "Scheduled" }
                TimelineItem { title: "Read 14 sources", time: "09:13", tone: Tone::Info, status: "Info",
                    "interviews-q3.pdf, tickets-sept.csv and 12 more."
                }
                TimelineItem { title: "Two figures disagree", time: "09:15", tone: Tone::Warning, status: "Warning",
                    "Churn is 4.1% in the board deck and 3.8% in the export."
                }
                TimelineItem { title: "Upload to Drive failed", time: "09:16", tone: Tone::Danger, status: "Failed", actor: "Agent",
                    "Permission denied on /Board/2026."
                }
                TimelineItem { title: "Summary approved", time: "09:40", tone: Tone::Success, status: "Done", actor: "Jo Marsh" }
            }
        }
    }
}
