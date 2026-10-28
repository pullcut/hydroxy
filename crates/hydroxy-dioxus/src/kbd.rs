//! `Kbd`: a keyboard shortcut hint.

use dioxus::prelude::*;

/// One key. Put several side by side for a combination.
#[component]
pub fn Kbd(
    /// The key, such as "⌘" or "K".
    children: Element,
) -> Element {
    rsx! {
        kbd { "data-scope": "kbd", {children} }
    }
}
