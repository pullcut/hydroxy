//! `EmptyState`: centred icon, title, line and optional action for empty lists and panes.

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};

/// Placeholder for an empty list or pane. Children are the action, such as a button.
#[component]
pub fn EmptyState(
    /// Icon in the tile.
    #[props(default = IconName::Inbox)]
    icon: IconName,
    /// What is empty, such as "Nothing archived yet".
    title: String,
    /// One line on why, or what fills it.
    #[props(default)]
    description: Option<String>,
    /// Optional action.
    children: Element,
) -> Element {
    rsx! {
        div { "data-scope": "empty-state", "data-part": "root",
            span { "data-scope": "empty-state", "data-part": "icon",
                Icon { name: icon, size: 40 }
            }
            p { "data-scope": "empty-state", "data-part": "title", "{title}" }
            if let Some(d) = description {
                p { "data-scope": "empty-state", "data-part": "description", "{d}" }
            }
            if children != VNode::empty() {
                div { "data-scope": "empty-state", "data-part": "action", {children} }
            }
        }
    }
}
