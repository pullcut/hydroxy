//! `Badge`: a small status label, such as "Ready" or "2 flags".

use dioxus::prelude::*;

pub use hydroxy_headless::badge::Tone;

/// A status label.
#[component]
pub fn Badge(
    /// Colour. Default neutral.
    #[props(default)]
    tone: Tone,
    /// Show a dot before the text.
    #[props(default)]
    dot: bool,
    /// The label.
    children: Element,
) -> Element {
    rsx! {
        span { "data-scope": "badge", "data-tone": tone.as_str(),
            if dot {
                span { "data-part": "dot", "aria-hidden": "true" }
            }
            {children}
        }
    }
}
