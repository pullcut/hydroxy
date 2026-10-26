//! `Chip`: a compact pill label, smaller than a tag, for dense rows and cards.

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};
pub use crate::tag::Jewel;

/// A compact label. Monospaced by default.
#[component]
pub fn Chip(
    /// Show a small diamond in this colour.
    #[props(default)]
    jewel: Option<Jewel>,
    /// Monospaced text. Default true; pass false for body text.
    #[props(default = true)]
    mono: bool,
    /// Adds a "Remove" button after the label.
    on_remove: Option<EventHandler<()>>,
    /// The label.
    children: Element,
) -> Element {
    rsx! {
        span {
            "data-scope": "chip",
            "data-jewel": jewel.map(Jewel::as_str),
            "data-mono": mono.then_some(""),
            if jewel.is_some() {
                span { "data-part": "diamond", "aria-hidden": "true" }
            }
            {children}
            if let Some(handler) = on_remove {
                button {
                    r#type: "button",
                    "data-part": "remove",
                    "aria-label": "Remove",
                    onclick: move |_| handler.call(()),
                    Icon { name: IconName::X, size: 11 }
                }
            }
        }
    }
}
