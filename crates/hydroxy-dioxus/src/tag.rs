//! `Tag`: a pill with a jewel diamond, for filters, categories and suggestions.

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};

pub use hydroxy_headless::jewel::Jewel;

/// A pill label. A toggle button when `on_click` is set; removable when `on_remove` is set.
#[component]
pub fn Tag(
    /// Diamond colour, and the border and tint when selected.
    #[props(default)]
    jewel: Jewel,
    /// Selected look. Reported as `aria-pressed` when the tag is clickable.
    #[props(default)]
    selected: ReadSignal<bool>,
    /// Makes the tag a toggle button.
    on_click: Option<EventHandler<()>>,
    /// Adds a "Remove" button after the label.
    on_remove: Option<EventHandler<()>>,
    /// The label.
    children: Element,
) -> Element {
    let is_selected = selected();
    rsx! {
        span {
            "data-scope": "tag",
            "data-jewel": jewel.as_str(),
            "data-selected": is_selected.then_some(""),
            "data-clickable": on_click.is_some().then_some(""),
            if let Some(handler) = on_click {
                button {
                    r#type: "button",
                    "data-part": "label",
                    "aria-pressed": is_selected.to_string(),
                    onclick: move |_| handler.call(()),
                    span { "data-part": "diamond", "aria-hidden": "true" }
                    {children.clone()}
                }
            } else {
                span { "data-part": "label",
                    span { "data-part": "diamond", "aria-hidden": "true" }
                    {children}
                }
            }
            if let Some(handler) = on_remove {
                button {
                    r#type: "button",
                    "data-part": "remove",
                    "aria-label": "Remove",
                    onclick: move |_| handler.call(()),
                    Icon { name: IconName::X, size: 14 }
                }
            }
        }
    }
}
