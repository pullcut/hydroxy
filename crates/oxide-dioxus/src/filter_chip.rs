//! `FilterChip`: a dashed toggle that turns solid amethyst when active.

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};

/// A filter toggle. Pair with Menu for multi-select filters (`dropdown`).
#[component]
pub fn FilterChip(
    /// The filter's name.
    label: String,
    /// Leading icon.
    icon: Option<IconName>,
    /// How many values are selected, shown in mono.
    #[props(default)]
    count: ReadSignal<Option<u32>>,
    /// On: solid amethyst. Reported as `aria-pressed`.
    #[props(default)]
    active: ReadSignal<bool>,
    /// Adds a chevron, for chips that open a menu.
    #[props(default)]
    dropdown: bool,
    /// Toggles or opens the filter.
    on_click: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        button {
            r#type: "button",
            "data-scope": "filter-chip",
            "aria-pressed": active().to_string(),
            "aria-haspopup": dropdown.then_some("menu"),
            onclick: move |_| {
                if let Some(handler) = on_click {
                    handler.call(());
                }
            },
            if let Some(name) = icon {
                Icon { name, size: 13 }
            }
            "{label}"
            if let Some(n) = count().filter(|n| *n > 0) {
                b { "data-part": "count", "{n}" }
            }
            if dropdown {
                Icon { name: IconName::ChevronDown, size: 13 }
            }
        }
    }
}
