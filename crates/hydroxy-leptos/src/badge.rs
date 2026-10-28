//! `<Badge>`: a small status label, such as "Ready" or "2 flags".

use leptos::prelude::*;

pub use hydroxy_headless::badge::Tone;

/// A status label.
#[component]
pub fn Badge(
    /// Colour. Default neutral.
    #[prop(optional)]
    tone: Tone,
    /// Show a dot before the text.
    #[prop(optional)]
    dot: bool,
    /// The label.
    children: Children,
) -> impl IntoView {
    view! {
        <span data-scope="badge" data-tone=tone.as_str()>
            {dot.then(|| view! { <span data-part="dot" aria-hidden="true"></span> })}
            {children()}
        </span>
    }
}
