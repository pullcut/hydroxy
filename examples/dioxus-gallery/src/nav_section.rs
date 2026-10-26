use dioxus::prelude::*;
use oxide_dioxus::icon::IconName;
use oxide_dioxus::nav_item::NavItem;
use oxide_dioxus::nav_section::NavSection;

#[component]
pub fn NavSectionDemo() -> Element {
    rsx! {
        nav { "aria-label": "Workspace", style: "display: grid; gap: var(--space-3); width: 260px; padding: var(--space-3); border-radius: var(--radius-md); background: var(--surface-sunken);",
            NavSection { label: "Projects", count: Some(3), default_open: true,
                NavItem { label: "Q3 customer research", icon: IconName::FileText, active: true }
                NavItem { label: "Onboarding interviews", icon: IconName::FileText }
                NavItem { label: "Support tickets", icon: IconName::Inbox }
            }
            NavSection { label: "Shared with you", count: Some(2),
                NavItem { label: "Pricing survey", icon: IconName::FileText }
                NavItem { label: "NPS August", icon: IconName::FileText }
            }
            NavSection { label: "Archived", count: Some(0), default_open: true, empty: "Nothing archived yet" }
        }
    }
}
