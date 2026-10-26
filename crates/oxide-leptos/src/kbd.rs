//! `<Kbd>`: a keyboard shortcut hint.

use leptos::prelude::*;

/// One key. Put several side by side for a combination.
#[component]
pub fn Kbd(
    /// The key, such as "⌘" or "K".
    children: Children,
) -> impl IntoView {
    view! { <kbd data-scope="kbd">{children()}</kbd> }
}
