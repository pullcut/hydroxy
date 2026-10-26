use dioxus::prelude::*;
use oxide_dioxus::icon::IconName;
use oxide_dioxus::nav_item::NavItem;

#[component]
pub fn NavItemDemo() -> Element {
    let mut current = use_signal(|| "inbox");
    let is = move |page: &'static str| current() == page;
    rsx! {
        div { style: "display: flex; gap: var(--space-8); align-items: flex-start;",
            nav { "aria-label": "Workspace", style: "display: grid; gap: 2px; width: 240px; padding: var(--space-3); border-radius: var(--radius-md); background: var(--surface-sunken);",
                NavItem { label: "Inbox", icon: IconName::Inbox, count: Some(3), active: is("inbox"), on_click: move |()| current.set("inbox") }
                NavItem { label: "Projects", icon: IconName::FileText, quiet_count: Some(12), active: is("projects"), on_click: move |()| current.set("projects") }
                NavItem { label: "Q3 customer research", sub: true, active: is("q3"), on_click: move |()| current.set("q3") }
                NavItem { label: "Onboarding interviews", sub: true, active: is("onboarding"), on_click: move |()| current.set("onboarding") }
                NavItem { label: "Starred", icon: IconName::Star, active: is("starred"), on_click: move |()| current.set("starred") }
            }
            nav { "aria-label": "Workspace, compact", style: "display: grid; gap: 2px; padding: var(--space-3); border-radius: var(--radius-md); background: var(--surface-sunken);",
                NavItem { label: "Inbox", icon: IconName::Inbox, count: Some(3), compact: true, active: is("inbox"), on_click: move |()| current.set("inbox") }
                NavItem { label: "Projects", icon: IconName::FileText, compact: true, active: is("projects"), on_click: move |()| current.set("projects") }
                NavItem { label: "Starred", icon: IconName::Star, compact: true, active: is("starred"), on_click: move |()| current.set("starred") }
            }
        }
    }
}
