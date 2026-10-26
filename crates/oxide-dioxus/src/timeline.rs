//! `Timeline` and `TimelineItem`: a vertical list of events, oldest first.

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};

pub use crate::badge::Tone;

/// The marker icon for a tone. Neutral and accent show a plain dot.
fn marker(tone: Tone) -> Option<IconName> {
    match tone {
        Tone::Success => Some(IconName::Check),
        Tone::Warning => Some(IconName::TriangleAlert),
        Tone::Danger => Some(IconName::X),
        Tone::Info => Some(IconName::Info),
        Tone::Neutral | Tone::Accent => None,
    }
}

/// An ordered list of events joined by a rule.
#[component]
pub fn Timeline(
    /// Accessible name. Default: "Timeline".
    #[props(default = "Timeline".to_string())]
    label: String,
    /// `TimelineItem`s.
    children: Element,
) -> Element {
    rsx! {
        ol { "data-scope": "timeline", "data-part": "root", "aria-label": label, {children} }
    }
}

/// One event: a coloured marker, a title and time, an optional actor and body.
#[component]
pub fn TimelineItem(
    /// What happened.
    title: String,
    /// When, as display text, such as "14:02" or "26 Oct".
    time: String,
    /// Machine-readable time for `<time datetime>`, such as "2026-10-26T14:02".
    datetime: Option<String>,
    /// Marker colour and icon. Default neutral.
    #[props(default)]
    tone: Tone,
    /// Status text for screen readers, such as "Failed". The marker alone is only colour.
    status: Option<String>,
    /// Who did it.
    actor: Option<String>,
    /// Details under the title.
    children: Option<Element>,
) -> Element {
    rsx! {
        li { "data-scope": "timeline", "data-part": "item", "data-tone": tone.as_str(),
            span { "data-part": "marker", "data-dot": marker(tone).is_none().then_some(""), "aria-hidden": "true",
                if let Some(name) = marker(tone) {
                    Icon { name, size: 12 }
                }
            }
            div { "data-part": "content",
                div { "data-part": "header",
                    span { "data-part": "title",
                        if let Some(s) = status {
                            span { "data-part": "status", "{s}: " }
                        }
                        "{title}"
                    }
                    time { "data-part": "time", datetime, "{time}" }
                }
                if let Some(a) = actor {
                    span { "data-part": "actor", "{a}" }
                }
                if let Some(c) = children {
                    div { "data-part": "body", {c} }
                }
            }
        }
    }
}
