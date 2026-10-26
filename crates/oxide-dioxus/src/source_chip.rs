//! `SourceChip` and `SourceRow`: mono chips naming the sources behind an answer.

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};

/// One source, such as a file name. A button when `on_click` is set.
#[component]
pub fn SourceChip(
    /// Leading icon. Default: a document.
    #[props(default = IconName::FileText)]
    icon: IconName,
    /// Opens the source.
    on_click: Option<EventHandler<()>>,
    /// The source's name.
    children: Element,
) -> Element {
    match on_click {
        Some(handler) => rsx! {
            button { r#type: "button", "data-scope": "source-chip", onclick: move |_| handler.call(()),
                Icon { name: icon, size: 12 }
                {children}
            }
        },
        None => rsx! {
            span { "data-scope": "source-chip",
                Icon { name: icon, size: 12 }
                {children}
            }
        },
    }
}

/// A wrapping row of source chips.
#[component]
pub fn SourceRow(
    /// Accessible name for the group. Default: "Sources".
    #[props(default = "Sources".to_string())]
    label: String,
    /// The chips.
    children: Element,
) -> Element {
    rsx! { div { "data-scope": "source-row", role: "group", "aria-label": "{label}", {children} } }
}
