use hydroxy_leptos::icon::IconName;
use hydroxy_leptos::nav_item::NavItem;
use hydroxy_leptos::sidebar_layout::SidebarLayout;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let (page, set_page) = signal("Inbox");
    let item = move |label: &'static str, icon: IconName| {
        view! {
            <NavItem
                label=label
                icon=icon
                active=Signal::derive(move || page.get() == label)
                on_click=move || set_page.set(label)
            />
        }
    };

    view! {
        // The layout fills the viewport by default; the demo gives it a fixed box.
        <div style="--ox-sidebar-layout-height: 420px; border: 1px solid var(--border-1); border-radius: var(--radius-md); overflow: hidden;">
            <SidebarLayout
                sidebar=move || view! {
                    <div style="display: flex; align-items: center; gap: var(--space-3); padding: 0 var(--space-2); font: var(--text-h4);">
                        <span aria-hidden="true">"◆"</span>
                        <span data-rail="hide">"Acme"</span>
                    </div>
                    <div style="display: grid; gap: 2px;">
                        {item("Inbox", IconName::Inbox)}
                        {item("Projects", IconName::FileText)}
                        {item("Starred", IconName::Star)}
                        {item("People", IconName::User)}
                    </div>
                }
                header=move || view! { <span style="font: var(--text-ui);">{page}</span> }
            >
                <p style="margin: 0; color: var(--text-2);">
                    "Press Ctrl/⌘ B or the panel button to switch between the full sidebar and the icon rail. "
                    "Below 768px wide the sidebar becomes a drawer."
                </p>
            </SidebarLayout>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "sidebar-layout",
    name: "Sidebar layout",
    group: "Layout",
    summary: "An app shell: a sidebar that collapses to an icon rail, a top bar and the main region. On narrow screens the sidebar is a native popover drawer.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("sidebar_layout.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/sidebar_layout.rs"),
    parts: &[
        &[
            "SidebarLayout",
            "root",
            "Flex shell. data-state=expanded|rail|collapsed. Height from --ox-sidebar-layout-height (default 100dvh)",
        ],
        &["", "sidebar", "<nav popover>. In flow on wide screens, a drawer below 768px. Width from --ox-sidebar-width"],
        &["", "header", "<header> top bar holding the toggles and your header content"],
        &["", "toggle", "Button with aria-expanded and aria-controls. Wide screens only"],
        &["", "mobile-toggle", "popovertarget button that opens the drawer. Narrow screens only"],
        &["", "main", "<main>, scrolls on its own"],
        &["", "[data-rail=hide]", "Put on your own sidebar text to hide it visually in rail mode"],
    ],
    keys: &[&["Ctrl+B / ⌘B", "Toggle the sidebar"], &["Escape", "Close the drawer on narrow screens"]],
    props: &[
        &["sidebar", "ViewFn", "Required. Sidebar content"],
        &["header", "ViewFn", "Top bar content after the toggle"],
        &["label", "String", "Accessible name of the sidebar nav. Default \"Main\""],
        &["open", "Option<bool>", "Controlled expanded state. Pair with on_open_change"],
        &["default_open", "bool", "Initial state when uncontrolled. Default true"],
        &["on_open_change", "Callback<bool>", "Called with the new state"],
        &["collapse_to", "CollapseMode", "Rail (default) or Hidden"],
        &["children", "Children", "The main content"],
    ],
    events: &[],
};
