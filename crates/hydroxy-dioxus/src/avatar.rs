//! `Avatar`: a person or agent. People are jewel-tinted circles, agents are
//! hexagons, and no initials shows an unassigned placeholder.

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};
pub use crate::tag::Jewel;

/// A person or agent avatar.
#[component]
pub fn Avatar(
    /// One or two letters. None shows the unassigned placeholder.
    #[props(default)]
    initials: Option<String>,
    /// Tint for people. Default amethyst.
    #[props(default)]
    jewel: Jewel,
    /// Draw as an agent: an ink hexagon.
    #[props(default)]
    agent: bool,
    /// Width and height in px. Default 22.
    #[props(default = 22)]
    size: u16,
    /// Accessible name, such as the person's full name. Falls back to the initials.
    #[props(default)]
    title: Option<String>,
) -> Element {
    let kind = match (&initials, agent) {
        (None, _) => "none",
        (Some(_), true) => "agent",
        (Some(_), false) => "person",
    };
    // Font is 40% of the size for people, 38% for agents.
    let font = if agent { size * 19 / 50 } else { size * 2 / 5 };
    let label = title.or_else(|| initials.clone()).unwrap_or_else(|| "Unassigned".to_string());

    rsx! {
        span {
            "data-scope": "avatar",
            "data-kind": kind,
            "data-jewel": jewel.as_str(),
            role: "img",
            "aria-label": "{label}",
            title: "{label}",
            style: "width: {size}px; height: {size}px; font-size: {font}px;",
            if let Some(text) = initials {
                "{text}"
            } else {
                Icon { name: IconName::User, size: size * 11 / 20 }
            }
        }
    }
}
