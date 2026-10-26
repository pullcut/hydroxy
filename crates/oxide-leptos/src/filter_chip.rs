//! `<FilterChip>`: a dashed toggle that turns solid amethyst when active.

use leptos::prelude::*;

use crate::icon::{Icon, IconName};

/// A filter toggle. Pair with Menu for multi-select filters (`dropdown`).
#[component]
pub fn FilterChip(
    /// The filter's name.
    #[prop(into)]
    label: String,
    /// Leading icon.
    #[prop(optional, into)]
    icon: Option<IconName>,
    /// How many values are selected, shown in mono.
    #[prop(optional, into)]
    count: MaybeProp<u32>,
    /// On: solid amethyst. Reported as `aria-pressed`.
    #[prop(optional, into)]
    active: MaybeProp<bool>,
    /// Adds a chevron, for chips that open a menu.
    #[prop(optional)]
    dropdown: bool,
    /// Toggles or opens the filter.
    #[prop(optional, into)]
    on_click: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            data-scope="filter-chip"
            aria-pressed=move || active.get().unwrap_or(false).to_string()
            aria-haspopup=dropdown.then_some("menu")
            on:click=move |_| {
                if let Some(cb) = on_click {
                    cb.run(());
                }
            }
        >
            {icon.map(|name| view! { <Icon name size=13 /> })}
            {label}
            {move || count.get().filter(|n| *n > 0).map(|n| view! { <b data-part="count">{n}</b> })}
            {dropdown.then(|| view! { <Icon name=IconName::ChevronDown size=13 /> })}
        </button>
    }
}
