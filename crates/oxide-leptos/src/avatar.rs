//! `<Avatar>`: a person or agent. People are jewel-tinted circles, agents are
//! hexagons, and no initials shows an unassigned placeholder.

use leptos::either::Either;
use leptos::prelude::*;

use crate::icon::{Icon, IconName};
pub use crate::tag::Jewel;

/// A person or agent avatar.
#[component]
pub fn Avatar(
    /// One or two letters. None shows the unassigned placeholder.
    #[prop(optional, into)]
    initials: Option<String>,
    /// Tint for people. Default amethyst.
    #[prop(optional)]
    jewel: Jewel,
    /// Draw as an agent: an ink hexagon.
    #[prop(optional)]
    agent: bool,
    /// Width and height in px. Default 22.
    #[prop(default = 22)]
    size: u16,
    /// Accessible name, such as the person's full name. Falls back to the initials.
    #[prop(optional, into)]
    title: Option<String>,
) -> impl IntoView {
    let kind = match (&initials, agent) {
        (None, _) => "none",
        (Some(_), true) => "agent",
        (Some(_), false) => "person",
    };
    // Font is 40% of the size for people, 38% for agents.
    let font = if agent { size * 19 / 50 } else { size * 2 / 5 };
    let label = title.or_else(|| initials.clone()).unwrap_or_else(|| "Unassigned".to_string());
    let style = format!("width: {size}px; height: {size}px; font-size: {font}px;");

    view! {
        <span
            data-scope="avatar"
            data-kind=kind
            data-jewel=jewel.as_str()
            role="img"
            aria-label=label.clone()
            title=label
            style=style
        >
            {match initials {
                Some(text) => Either::Left(text),
                None => Either::Right(view! { <Icon name=IconName::User size=size * 11 / 20 /> }),
            }}
        </span>
    }
}
