//! `<EmptyState>`: centred icon, title, line and optional action for empty lists and panes.

use leptos::prelude::*;

use crate::icon::{Icon, IconName};

/// Placeholder for an empty list or pane. Children are the action, such as a button.
#[component]
pub fn EmptyState(
    /// Icon in the tile.
    #[prop(default = IconName::Inbox)]
    icon: IconName,
    /// What is empty, such as "Nothing archived yet".
    #[prop(into)]
    title: String,
    /// One line on why, or what fills it.
    #[prop(optional, into)]
    description: Option<String>,
    /// Optional action.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    view! {
        <div data-scope="empty-state" data-part="root">
            <span data-scope="empty-state" data-part="icon">
                <Icon name=icon size=40 />
            </span>
            <p data-scope="empty-state" data-part="title">{title}</p>
            {description.map(|d| view! { <p data-scope="empty-state" data-part="description">{d}</p> })}
            {children.map(|c| view! { <div data-scope="empty-state" data-part="action">{c()}</div> })}
        </div>
    }
}
