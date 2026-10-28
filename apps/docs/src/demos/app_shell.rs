use hydroxy_leptos::app_shell::{AppShell, ScrollMode};
use hydroxy_leptos::icon::IconName;
use hydroxy_leptos::nav_item::NavItem;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (page, set_page) = signal("Overview");
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
        // The shell is a fixed 460px tall here, with only the main region scrolling.
        // Below 768px wide the navigation becomes a drawer behind the menu button.
        <div style="--ox-app-shell-nav-width: 180px; --ox-app-shell-aside-width: 150px; border: 1px solid var(--border-1); border-radius: var(--radius-md); overflow: hidden;">
            <AppShell
                scroll_mode=ScrollMode::Main
                host_block_size=460.0
                main_id="app-shell-demo-main"
                skip_label="Skip to content"
                navigation=move || view! {
                    <div style="display: grid; gap: 2px;">
                        {item("Overview", IconName::Inbox)}
                        {item("Projects", IconName::FileText)}
                        {item("Starred", IconName::Star)}
                        {item("People", IconName::User)}
                    </div>
                }
                header=move || view! { <strong style="font: var(--text-h4);">"Acme"</strong> }
                aside=move || view! { <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">"Notes and activity sit in the aside."</p> }
                footer=move || view! { <span>"Footer"</span> }
            >
                <h2 style="margin: 0 0 var(--space-4); font: var(--text-h3);">{move || page.get()}</h2>
                <p style="margin: 0 0 var(--space-4); color: var(--text-2);">
                    "The header, navigation, aside and footer stay put while this region scrolls."
                </p>
                {(1..=12)
                    .map(|i| view! { <p style="margin: 0 0 var(--space-4); color: var(--text-2);">{format!("Paragraph {i} of placeholder content to scroll.")}</p> })
                    .collect_view()}
            </AppShell>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "app-shell",
    name: "App shell",
    group: "Layout",
    summary: "A page frame with header, navigation, main, aside and footer. The navigation sits beside the content on wide screens and becomes a native drawer on narrow ones, with a skip link and an optional fixed-height scroll mode.",
    demo,
    leptos: include_str!("app_shell.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/app_shell.rs"),
    parts: &[
        &["AppShell", "root", "data-scroll-mode is document or main; data-sticky-header when set"],
        &["(inside)", "skip-link", "Hidden until focused. Needs main_id and skip_label together"],
        &["(inside)", "header", "Holds the menu button and your header content"],
        &["(inside)", "menu", "Opens the drawer on narrow screens. aria-expanded follows it"],
        &["(inside)", "body", "Row of navigation, main and aside"],
        &["(inside)", "navigation", "A nav landmark, rendered once. A native popover on narrow screens"],
        &["(inside)", "close", "Closes the drawer"],
        &["(inside)", "main", "The page content. Receives focus from the skip link"],
        &["(inside)", "aside / footer", "Optional regions. The aside hides on narrow screens"],
    ],
    keys: &[&["Esc", "Close the drawer"], &["Tab", "From the top of the page, reaches the skip link first"]],
    props: &[
        &["navigation", "ViewFn", "Navigation content, rendered once"],
        &["header / aside / footer", "Option<ViewFn>", "Optional regions"],
        &["navigation_open", "Option<bool>", "Controlled drawer state. Pair with on_navigation_open_change"],
        &["default_navigation_open", "bool", "Initial drawer state when uncontrolled"],
        &[
            "on_navigation_open_change",
            "Callback<(bool, NavReason)>",
            "The drawer wants to open or close, and why: Trigger or Dismiss",
        ],
        &["scroll_mode", "ScrollMode", "Document (default) or Main. Main needs host_block_size"],
        &["host_block_size", "Option<f64>", "The shell's height in pixels for Main"],
        &["sticky_header", "bool", "Keep the header visible while the page scrolls"],
        &["main_id / skip_label", "Option<String>", "Enable the skip link. Both or neither"],
        &["navigation_label", "Option<String>", "Name of the landmark. Default Navigation"],
    ],
};
