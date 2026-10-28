//! `PresenceBar`: an overlapping stack of avatars for who's here. Active people
//! come first; idle ones dim and away ones fade further. Each avatar shows the
//! name on hover or focus. People past `max` sit behind a "+N" button whose
//! popover lists them.

use dioxus::prelude::*;
use hydroxy_headless::presence_bar as h;

use crate::avatar::Avatar;

pub use hydroxy_headless::presence_bar::{Collaborator, PresenceStatus};

/// Who's here, as an avatar stack.
#[component]
pub fn PresenceBar(
    /// Everyone in the room, in the host's order (say, join time).
    people: ReadSignal<Vec<Collaborator>>,
    /// Most avatars before the "+N" button takes the last slot. Default 5.
    #[props(default = 5)]
    max: usize,
    /// Avatar size in px. Default 28.
    #[props(default = 28)]
    size: u16,
) -> Element {
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let ids = h::ids(&root);
    let layout = h::layout(&people.read(), max);
    let hidden = layout.overflow.len();

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            role: "group",
            "aria-label": layout.summary,
            style: "--presence-size: {size}px;",
            ul { "data-scope": h::SCOPE, "data-part": "stack",
                for p in layout.visible {
                    li {
                        key: "{p.id}",
                        "data-scope": h::SCOPE,
                        "data-part": "item",
                        "data-status": p.status.as_str(),
                        "data-self": p.is_self.then_some(""),
                        tabindex: "0",
                        "aria-label": p.label(),
                        style: h::item_style(&root, &p.id),
                        span { "data-scope": h::SCOPE, "data-part": "avatar", "aria-hidden": "true",
                            Avatar { initials: h::initials(&p.name), jewel: p.tint(), size, title: p.name.clone() }
                        }
                        span { "data-scope": h::SCOPE, "data-part": "dot", "aria-hidden": "true" }
                        span {
                            "data-scope": h::SCOPE,
                            "data-part": "tip",
                            "aria-hidden": "true",
                            style: h::tip_style(&root, &p.id),
                            span { "data-scope": h::SCOPE, "data-part": "tip-name", {h::display_name(&p)} }
                            span { "data-scope": h::SCOPE, "data-part": "tip-status", {p.status.as_str()} }
                        }
                    }
                }
                if hidden > 0 {
                    li { "data-scope": h::SCOPE, "data-part": "overflow-item",
                        button {
                            r#type: "button",
                            "data-scope": h::SCOPE,
                            "data-part": "overflow",
                            id: ids.overflow.clone(),
                            "popovertarget": ids.list.clone(),
                            "aria-label": h::overflow_label(hidden),
                            style: h::overflow_style(&ids),
                            "+{hidden}"
                        }
                    }
                }
            }
            div {
                id: ids.list.clone(),
                "popover": "auto",
                "data-overlay": "",
                "data-side": "bottom",
                "data-scope": h::SCOPE,
                "data-part": "list",
                style: h::list_style(&ids),
                ul { "data-scope": h::SCOPE, "data-part": "list-items", "aria-label": "Also here",
                    for p in layout.overflow {
                        li { key: "{p.id}", "data-scope": h::SCOPE, "data-part": "list-item", "data-status": p.status.as_str(),
                            span { "data-scope": h::SCOPE, "data-part": "avatar", "aria-hidden": "true",
                                Avatar { initials: h::initials(&p.name), jewel: p.tint(), size: 24 }
                                span { "data-scope": h::SCOPE, "data-part": "dot" }
                            }
                            span { "data-scope": h::SCOPE, "data-part": "list-name", {h::display_name(&p)} }
                            span { "data-scope": h::SCOPE, "data-part": "list-status", {p.status.as_str()} }
                        }
                    }
                }
            }
        }
    }
}
