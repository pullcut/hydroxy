//! `<Chip>`: a compact pill label, smaller than a tag, for dense rows and cards.

use leptos::prelude::*;

use crate::icon::{Icon, IconName};
pub use crate::tag::Jewel;

/// A compact label. Monospaced by default.
#[component]
pub fn Chip(
    /// Show a small diamond in this colour.
    #[prop(optional, into)]
    jewel: Option<Jewel>,
    /// Monospaced text. Default true; pass false for body text.
    #[prop(default = true)]
    mono: bool,
    /// Adds a "Remove" button after the label.
    #[prop(optional, into)]
    on_remove: Option<Callback<()>>,
    /// The label.
    children: Children,
) -> impl IntoView {
    view! {
        <span data-scope="chip" data-jewel=jewel.map(Jewel::as_str) data-mono=mono.then_some("")>
            {jewel.map(|_| view! { <span data-part="diamond" aria-hidden="true"></span> })}
            {children()}
            {on_remove
                .map(|cb| {
                    view! {
                        <button type="button" data-part="remove" aria-label="Remove" on:click=move |_| cb.run(())>
                            <Icon name=IconName::X size=11 />
                        </button>
                    }
                })}
        </span>
    }
}
