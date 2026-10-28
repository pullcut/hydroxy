//! `<Icon>`: a line icon from the hydroxy set, sized in px, coloured by `currentColor`.

use leptos::prelude::*;
pub use hydroxy_headless::icons::Icon as IconName;

/// A decorative line icon. Pair with visible text, or give the parent an `aria-label`.
#[component]
pub fn Icon(
    /// Which icon.
    name: IconName,
    /// Width and height in px.
    #[prop(default = 16)]
    size: u16,
) -> impl IntoView {
    view! {
        <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.75"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
            width=size
            height=size
            inner_html=name.markup()
        ></svg>
    }
}
