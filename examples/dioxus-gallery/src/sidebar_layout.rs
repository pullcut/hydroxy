use dioxus::prelude::*;
use oxide_dioxus::icon::IconName;
use oxide_dioxus::nav_item::NavItem;
use oxide_dioxus::sidebar_layout::SidebarLayout;

#[component]
pub fn SidebarLayoutDemo() -> Element {
    let mut page = use_signal(|| "Inbox");
    let items = [
        ("Inbox", IconName::Inbox),
        ("Projects", IconName::FileText),
        ("Starred", IconName::Star),
        ("People", IconName::User),
    ];

    rsx! {
        // The layout fills the viewport by default; the demo gives it a fixed box.
        div { style: "--ox-sidebar-layout-height: 420px; border: 1px solid var(--border-1); border-radius: var(--radius-md); overflow: hidden;",
            SidebarLayout {
                sidebar: rsx! {
                    div { style: "display: flex; align-items: center; gap: var(--space-3); padding: 0 var(--space-2); font: var(--text-h4);",
                        span { "aria-hidden": "true", "◆" }
                        span { "data-rail": "hide", "Acme" }
                    }
                    div { style: "display: grid; gap: 2px;",
                        for (label, icon) in items {
                            NavItem {
                                key: "{label}",
                                label,
                                icon,
                                active: page() == label,
                                on_click: move |()| page.set(label),
                            }
                        }
                    }
                },
                header: rsx! { span { style: "font: var(--text-ui);", "{page}" } },
                p { style: "margin: 0; color: var(--text-2);",
                    "Press Ctrl/⌘ B or the panel button to switch between the full sidebar and the icon rail. "
                    "Below 768px wide the sidebar becomes a drawer."
                }
            }
        }
    }
}
