//! `<Timeline>` and `<TimelineItem>`: a vertical list of events, oldest first.

use leptos::prelude::*;

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
    #[prop(default = "Timeline".into(), into)]
    label: String,
    /// `TimelineItem`s.
    children: Children,
) -> impl IntoView {
    view! { <ol data-scope="timeline" data-part="root" aria-label=label>{children()}</ol> }
}

/// One event: a coloured marker, a title and time, an optional actor and body.
#[component]
pub fn TimelineItem(
    /// What happened.
    #[prop(into)]
    title: String,
    /// When, as display text, such as "14:02" or "26 Oct".
    #[prop(into)]
    time: String,
    /// Machine-readable time for `<time datetime>`, such as "2026-10-26T14:02".
    #[prop(optional, into)]
    datetime: Option<String>,
    /// Marker colour and icon. Default neutral.
    #[prop(optional)]
    tone: Tone,
    /// Status text for screen readers, such as "Failed". The marker alone is only colour.
    #[prop(optional, into)]
    status: Option<String>,
    /// Who did it.
    #[prop(optional, into)]
    actor: Option<String>,
    /// Details under the title.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    view! {
        <li data-scope="timeline" data-part="item" data-tone=tone.as_str()>
            <span data-part="marker" data-dot=marker(tone).is_none().then_some("") aria-hidden="true">
                {marker(tone).map(|name| view! { <Icon name size=12 /> })}
            </span>
            <div data-part="content">
                <div data-part="header">
                    <span data-part="title">
                        {status.map(|s| view! { <span data-part="status">{s} ": "</span> })}
                        {title}
                    </span>
                    <time data-part="time" datetime=datetime>{time}</time>
                </div>
                {actor.map(|a| view! { <span data-part="actor">{a}</span> })}
                {children.map(|c| view! { <div data-part="body">{c()}</div> })}
            </div>
        </li>
    }
}
