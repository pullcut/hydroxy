//! `<ActivityFeed>` and `<ActivityItem>`: a timeline of changes and comments.

use leptos::prelude::*;

use crate::avatar::{Avatar, Jewel};

/// An ordered list of activity, newest last, with a rule through the avatars.
#[component]
pub fn ActivityFeed(
    /// Accessible name. Default: "Activity".
    #[prop(default = "Activity".into(), into)]
    label: String,
    /// `ActivityItem`s.
    children: Children,
) -> impl IntoView {
    view! { <ol data-scope="activity-feed" aria-label=label>{children()}</ol> }
}

/// One entry: a line ("Jo Marsh moved the status …") or, with `comment`, a bordered bubble.
#[component]
pub fn ActivityItem(
    /// Who did it.
    #[prop(into)]
    name: String,
    /// Avatar initials. None shows the unassigned avatar.
    #[prop(optional, into)]
    initials: Option<String>,
    /// Avatar colour.
    #[prop(optional)]
    jewel: Jewel,
    /// Draw the avatar as an agent.
    #[prop(optional)]
    agent: bool,
    /// When, such as "2h" or "26 Oct".
    #[prop(into)]
    time: String,
    /// Show the children as a comment bubble rather than a line.
    #[prop(optional)]
    comment: bool,
    /// What happened, or the comment text.
    children: Children,
) -> impl IntoView {
    let title = name.clone();
    let avatar = match initials {
        Some(i) => view! { <Avatar initials=i jewel=jewel agent=agent size=22 title=title /> }.into_any(),
        None => view! { <Avatar jewel=jewel agent=agent size=22 title=title /> }.into_any(),
    };
    if comment {
        view! {
            <li data-part="item">
                {avatar}
                <div data-part="comment">
                    <div data-part="comment-head">
                        <b>{name}</b>
                        <span data-part="time">{time}</span>
                    </div>
                    <div>{children()}</div>
                </div>
            </li>
        }
        .into_any()
    } else {
        view! {
            <li data-part="item">
                {avatar}
                <div data-part="line">
                    <b>{name}</b>
                    {children()}
                    <span data-part="time">{time}</span>
                </div>
            </li>
        }
        .into_any()
    }
}
