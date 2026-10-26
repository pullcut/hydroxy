//! `<Tag>`: a pill with a jewel diamond, for filters, categories and suggestions.

use leptos::either::Either;
use leptos::prelude::*;

use crate::icon::{Icon, IconName};

pub use oxide_headless::jewel::Jewel;

/// A pill label. A toggle button when `on_click` is set; removable when `on_remove` is set.
#[component]
pub fn Tag(
    /// Diamond colour, and the border and tint when selected.
    #[prop(optional)]
    jewel: Jewel,
    /// Selected look. Reported as `aria-pressed` when the tag is clickable.
    #[prop(optional, into)]
    selected: MaybeProp<bool>,
    /// Makes the tag a toggle button.
    #[prop(optional, into)]
    on_click: Option<Callback<()>>,
    /// Adds a "Remove" button after the label.
    #[prop(optional, into)]
    on_remove: Option<Callback<()>>,
    /// The label.
    children: Children,
) -> impl IntoView {
    let is_selected = move || selected.get().unwrap_or(false);
    let diamond = || view! { <span data-part="diamond" aria-hidden="true"></span> };
    let label = match on_click {
        Some(cb) => Either::Left(view! {
            <button
                type="button"
                data-part="label"
                aria-pressed=move || is_selected().to_string()
                on:click=move |_| cb.run(())
            >
                {diamond()}
                {children()}
            </button>
        }),
        None => Either::Right(view! { <span data-part="label">{diamond()} {children()}</span> }),
    };

    view! {
        <span
            data-scope="tag"
            data-jewel=jewel.as_str()
            data-selected=move || is_selected().then_some("")
            data-clickable=on_click.is_some().then_some("")
        >
            {label}
            {on_remove
                .map(|cb| {
                    view! {
                        <button type="button" data-part="remove" aria-label="Remove" on:click=move |_| cb.run(())>
                            <Icon name=IconName::X size=14 />
                        </button>
                    }
                })}
        </span>
    }
}
