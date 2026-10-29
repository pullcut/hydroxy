use hydroxy_leptos::icon::IconName;
use hydroxy_leptos::nav_item::NavItem;
use hydroxy_leptos::nav_section::NavSection;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    view! {
        <nav aria-label="Workspace" style="display: grid; gap: var(--space-3); width: 260px; padding: var(--space-3); border-radius: var(--radius-md); background: var(--surface-sunken);">
            <NavSection label="Projects" count=3 default_open=true>
                <NavItem label="Q3 customer research" icon=IconName::FileText active=true />
                <NavItem label="Onboarding interviews" icon=IconName::FileText />
                <NavItem label="Support tickets" icon=IconName::Inbox />
            </NavSection>
            // Folded, it shows its count.
            <NavSection label="Shared with you" count=2>
                <NavItem label="Pricing survey" icon=IconName::FileText />
                <NavItem label="NPS August" icon=IconName::FileText />
            </NavSection>
            <NavSection label="Archived" count=0 default_open=true empty="Nothing archived yet">
                ""
            </NavSection>
        </nav>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "nav-section",
    name: "Nav section",
    group: "Navigation",
    summary: "A collapsible sidebar group with a mono label. It shows its count while folded and an empty line when it holds nothing.",
    demo,
    leptos: include_str!("nav_section.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/nav_section.rs"),
    parts: &[
        &["NavSection", "—", "div[data-scope=nav-section][data-state=open|closed]"],
        &["", "head", "button with aria-expanded and aria-controls"],
        &["", "count", "Item count, shown while folded"],
        &["", "body", "role=group, hidden while folded"],
    ],
    keys: &[&["Enter / Space", "Opens or folds the group"]],
    props: &[
        &["label", "String", "Required. The group's name"],
        &["count", "u32", "Items in the group; 0 shows `empty`"],
        &["open", "Option<bool>", "Controlled open state"],
        &["default_open", "bool", "Initial open state"],
        &["empty", "String", "Shown when count is 0"],
        &["on_open_change", "Callback<bool>", "Called with the new open state"],
    ],
    events: &[],
};
