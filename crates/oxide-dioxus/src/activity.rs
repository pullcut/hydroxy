//! `ActivityFeed` and `ActivityItem`: a timeline of changes and comments.

use dioxus::prelude::*;

use crate::avatar::{Avatar, Jewel};

/// An ordered list of activity, newest last, with a rule through the avatars.
#[component]
pub fn ActivityFeed(
    /// Accessible name. Default: "Activity".
    #[props(default = "Activity".to_string())]
    label: String,
    /// `ActivityItem`s.
    children: Element,
) -> Element {
    rsx! { ol { "data-scope": "activity-feed", "aria-label": "{label}", {children} } }
}

/// One entry: a line ("Jo Marsh moved the status …") or, with `comment`, a bordered bubble.
#[component]
pub fn ActivityItem(
    /// Who did it.
    name: String,
    /// Avatar initials. None shows the unassigned avatar.
    initials: Option<String>,
    /// Avatar colour.
    #[props(default)]
    jewel: Jewel,
    /// Draw the avatar as an agent.
    #[props(default)]
    agent: bool,
    /// When, such as "2h" or "26 Oct".
    time: String,
    /// Show the children as a comment bubble rather than a line.
    #[props(default)]
    comment: bool,
    /// What happened, or the comment text.
    children: Element,
) -> Element {
    let avatar = match initials {
        Some(i) => rsx! { Avatar { initials: i, jewel, agent, size: 22, title: name.clone() } },
        None => rsx! { Avatar { jewel, agent, size: 22, title: name.clone() } },
    };
    if comment {
        rsx! {
            li { "data-part": "item",
                {avatar}
                div { "data-part": "comment",
                    div { "data-part": "comment-head",
                        b { "{name}" }
                        span { "data-part": "time", "{time}" }
                    }
                    div { {children} }
                }
            }
        }
    } else {
        rsx! {
            li { "data-part": "item",
                {avatar}
                div { "data-part": "line",
                    b { "{name}" }
                    {children}
                    span { "data-part": "time", "{time}" }
                }
            }
        }
    }
}
