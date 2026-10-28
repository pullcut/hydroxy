use leptos::prelude::*;
use hydroxy_leptos::breadcrumb::{Breadcrumb, BreadcrumbItem, BreadcrumbLink};
use hydroxy_leptos::icon::IconName;

pub fn demo() -> AnyView {
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

    view! {
        // Drag the corner to narrow it: middle pages fold into "…".
        <div style="resize: horizontal; overflow: hidden; min-width: 220px; max-width: 100%; width: 640px; padding: var(--space-3); border: 1px dashed var(--border-2); border-radius: var(--radius-md);">
            <Breadcrumb items=trail />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "breadcrumb",
    name: "Breadcrumb",
    group: "Navigation",
    summary: "The trail of pages leading here. When space runs out, middle pages fold into a \"…\" menu, keeping the first page and the last two. Any page can list its neighbours.",
    demo,
    leptos: include_str!("breadcrumb.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/breadcrumb.rs"),
    parts: &[
        &["Breadcrumb", "root", "<nav aria-label=\"Breadcrumb\">. Fills its container so it can measure"],
        &["Breadcrumb", "list", "<ol> of items"],
        &["Breadcrumb", "item", "<li>. data-current on the last, data-ellipsis on the fold"],
        &["Breadcrumb", "link", "<a>, or <span> without href. The last has aria-current=\"page\""],
        &["Breadcrumb", "separator", "Chevron between items, aria-hidden. Mirrors in RTL"],
        &["Breadcrumb", "menu-trigger", "\"…\" or sibling-pages button, opening a popover"],
        &["Breadcrumb", "menu / menu-link", "The links in either popover"],
        &["Breadcrumb", "measure", "Invisible copy of the full trail, measured with a ResizeObserver"],
    ],
    keys: &[
        &["Tab", "Move through the links and menu buttons"],
        &["Enter / Space", "On \"…\" or a chevron: open the list of pages"],
        &["Escape", "Close the list and return to its button"],
    ],
    props: &[
        &["items", "Vec<BreadcrumbItem>", "Required. Root first; the last is the current page"],
        &["BreadcrumbItem::new / current", "label, href", "A linked page, or the current page with no link"],
        &[".icon", "IconName", "Icon before the label"],
        &[".siblings", "Vec<BreadcrumbLink>", "Pages beside this one, in a dropdown"],
    ],
};
