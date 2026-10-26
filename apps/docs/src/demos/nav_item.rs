use leptos::prelude::*;
use oxide_leptos::icon::IconName;
use oxide_leptos::nav_item::NavItem;

pub fn demo() -> AnyView {
    let (current, set_current) = signal("inbox");
    let is = move |page: &'static str| Signal::derive(move || current.get() == page);

    view! {
        <div style="display: flex; gap: var(--space-8); align-items: flex-start;">
            <nav aria-label="Workspace" style="display: grid; gap: 2px; width: 240px; padding: var(--space-3); border-radius: var(--radius-md); background: var(--surface-sunken);">
                <NavItem label="Inbox" icon=IconName::Inbox count=3 active=is("inbox") on_click=move || set_current.set("inbox") />
                <NavItem label="Projects" icon=IconName::FileText quiet_count=12 active=is("projects") on_click=move || set_current.set("projects") />
                <NavItem label="Q3 customer research" sub=true active=is("q3") on_click=move || set_current.set("q3") />
                <NavItem label="Onboarding interviews" sub=true active=is("onboarding") on_click=move || set_current.set("onboarding") />
                <NavItem label="Starred" icon=IconName::Star active=is("starred") on_click=move || set_current.set("starred") />
            </nav>
            // Compact: icon only, the label becomes the accessible name and tooltip.
            <nav aria-label="Workspace, compact" style="display: grid; gap: 2px; padding: var(--space-3); border-radius: var(--radius-md); background: var(--surface-sunken);">
                <NavItem label="Inbox" icon=IconName::Inbox count=3 compact=true active=is("inbox") on_click=move || set_current.set("inbox") />
                <NavItem label="Projects" icon=IconName::FileText compact=true active=is("projects") on_click=move || set_current.set("projects") />
                <NavItem label="Starred" icon=IconName::Star compact=true active=is("starred") on_click=move || set_current.set("starred") />
            </nav>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "nav-item",
    name: "Nav item",
    group: "Navigation",
    summary: "A sidebar row. The count is a loud amethyst badge for unread items; the quiet count appears only on hover.",
    demo,
    leptos: include_str!("nav_item.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/nav_item.rs"),
    parts: &[
        &["NavItem", "—", "a with href, otherwise button; aria-current=page when active"],
        &["", "label", "The row's text, truncated with an ellipsis"],
        &["", "count", "Unread badge, read as \"N unread\""],
        &["", "quiet", "Total count, shown on hover or focus"],
    ],
    keys: &[&["Enter", "Follows the link, or activates the button"]],
    props: &[
        &["label", "String", "Required. In compact mode, the accessible name and tooltip"],
        &["icon", "IconName", "Leading icon"],
        &["href", "String", "Renders a link instead of a button"],
        &["active", "bool", "The current page"],
        &["count", "u32", "Unread count"],
        &["quiet_count", "u32", "Total count, shown on hover"],
        &["sub", "bool", "Indented child row"],
        &["compact", "bool", "Icon only"],
        &["on_click", "Callback<()>", "Called on click"],
    ],
};
