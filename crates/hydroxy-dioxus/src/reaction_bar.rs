//! `ReactionBar`: emoji reactions under a message. Each chip toggles your
//! reaction and shows who reacted on hover or focus; the add button opens an
//! emoji picker. The host owns the reactions and applies `on_toggle`.

use dioxus::prelude::*;
use hydroxy_headless::reaction_bar as h;

use crate::emoji_picker::{EmojiPicker, SkinTone};
use crate::focus::focus_id;
use crate::icon::{Icon, IconName};
use crate::overlay::hide_popover;

pub use hydroxy_headless::reaction_bar::{Reaction, toggle};

/// A row of reaction chips with an add button.
#[component]
pub fn ReactionBar(
    /// The reactions, in the order they were first added.
    reactions: ReadSignal<Vec<Reaction>>,
    /// Called with the emoji and its name when you toggle a chip or pick a new reaction.
    on_toggle: Option<EventHandler<(String, String)>>,
    /// Recently used emoji for the picker. Store what `on_recent_change` gives you.
    #[props(default)]
    recent: ReadSignal<Vec<String>>,
    /// Called with the picker's new recent list.
    on_recent_change: Option<EventHandler<Vec<String>>>,
    /// Controlled skin tone for the picker.
    #[props(default)]
    skin_tone: ReadSignal<Option<SkinTone>>,
    /// Called when a skin tone is chosen in the picker.
    on_skin_tone_change: Option<EventHandler<SkinTone>>,
    /// Accessible name for the group. Default "Reactions".
    #[props(default = "Reactions".to_string())]
    label: String,
) -> Element {
    let ids = use_hook(|| h::ids(&hydroxy_headless::next_id(h::SCOPE)));
    let mut announce = use_signal(String::new);
    let mut toggle = move |r: &Reaction| {
        announce.set(h::toggle_announcement(r));
        if let Some(handler) = on_toggle {
            handler.call((r.emoji.clone(), r.name.clone()));
        }
    };
    let (picker_id, add_id) = (ids.picker.clone(), ids.add.clone());
    let pick = move |(emoji, name): (String, String)| {
        hide_popover(&picker_id);
        focus_id(&add_id);
        // Picking one you already hold leaves it alone rather than taking it back.
        let existing = reactions.peek().iter().find(|r| r.emoji == emoji).cloned();
        match existing {
            Some(r) if r.reacted => {}
            Some(r) => toggle(&r),
            None => toggle(&Reaction::new(emoji, name, &[])),
        }
    };
    let picker_id = ids.picker.clone();
    let list = reactions.read().clone();

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", role: "group", "aria-label": label,
            // Keyed by emoji, so a toggled chip keeps its node and focus.
            for r in list {
              span { key: "{r.emoji}", "data-scope": h::SCOPE, "data-part": "item",
                button {
                    r#type: "button",
                    id: h::chip_id(&ids, &r.emoji),
                    "aria-pressed": r.reacted.to_string(),
                    "aria-label": h::chip_label(&r),
                    "aria-describedby": h::tip_id(&ids, &r.emoji),
                    style: h::chip_style(&ids, &r.emoji),
                    "data-scope": h::SCOPE,
                    "data-part": "chip",
                    "data-state": if r.reacted { "on" } else { "off" },
                    onclick: {
                        let r = r.clone();
                        move |_| toggle(&r)
                    },
                    span { "data-scope": h::SCOPE, "data-part": "emoji", "aria-hidden": "true", "{r.emoji}" }
                    span { "data-scope": h::SCOPE, "data-part": "count", "aria-hidden": "true", "{r.count()}" }
                }
                span {
                    id: h::tip_id(&ids, &r.emoji),
                    role: "tooltip",
                    style: h::tip_style(&ids, &r.emoji),
                    "data-scope": h::SCOPE,
                    "data-part": "tip",
                    {h::who(&r, h::TIP_NAMES)}
                }
              }
            }
            button {
                r#type: "button",
                id: ids.add.clone(),
                "popovertarget": ids.picker.clone(),
                "aria-label": "Add reaction",
                title: "Add reaction",
                "aria-haspopup": "dialog",
                style: h::add_style(&ids),
                "data-scope": h::SCOPE,
                "data-part": "add",
                Icon { name: IconName::SmilePlus, size: 18 }
            }
            div {
                id: ids.picker.clone(),
                "popover": "auto",
                role: "dialog",
                "aria-label": "Add reaction",
                "data-overlay": "",
                "data-side": "bottom",
                style: h::picker_style(&ids),
                "data-scope": h::SCOPE,
                "data-part": "picker",
                ontoggle: move |_| {
                    // Opening puts focus in the search field.
                    let _ = document::eval(&format!(
                        "const p = document.getElementById({picker_id}); if (p && p.matches(':popover-open')) p.querySelector('input')?.focus();", picker_id = crate::js::js_string(&picker_id)));
                },
                EmojiPicker {
                    on_select: pick,
                    recent,
                    on_recent_change: move |list| {
                        if let Some(handler) = on_recent_change {
                            handler.call(list);
                        }
                    },
                    skin_tone,
                    on_skin_tone_change: move |t| {
                        if let Some(handler) = on_skin_tone_change {
                            handler.call(t);
                        }
                    },
                }
            }
            span { "data-scope": h::SCOPE, "data-part": "status", role: "status", "{announce}" }
        }
    }
}
