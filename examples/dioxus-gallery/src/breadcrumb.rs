use dioxus::prelude::*;
use oxide_dioxus::breadcrumb::{Breadcrumb, BreadcrumbItem, BreadcrumbLink};
use oxide_dioxus::icon::IconName;

#[component]
pub fn BreadcrumbDemo() -> Element {
    let trail = vec![
        BreadcrumbItem::new("Acme workspace", "#workspace").icon(IconName::Monitor),
        BreadcrumbItem::new("Research", "#research"),
        BreadcrumbItem::new("2026 studies", "#studies"),
        BreadcrumbItem::new("Customer interviews", "#interviews"),
        BreadcrumbItem::new("Q3 onboarding", "#q3").siblings(vec![
            BreadcrumbLink::new("Q1 pricing", "#q1"),
            BreadcrumbLink::new("Q2 retention", "#q2"),
            BreadcrumbLink::new("Q4 enterprise", "#q4"),
        ]),
        BreadcrumbItem::current("Synthesis notes").icon(IconName::FileText),
    ];

    rsx! {
        // Drag the corner to narrow it: middle pages fold into "…".
        div { style: "resize: horizontal; overflow: hidden; min-width: 220px; max-width: 100%; width: 640px; padding: var(--space-3); border: 1px dashed var(--border-2); border-radius: var(--radius-md);",
            Breadcrumb { items: trail }
        }
    }
}
