use leptos::prelude::*;
use oxide_leptos::pagination::Pagination;

pub fn demo() -> AnyView {
    let (page, set_page) = signal(5_usize);

    view! {
        <div style="display: grid; gap: var(--space-6);">
            // Controlled, with the page shown alongside.
            <div style="display: grid; gap: var(--space-3);">
                <Pagination count=12 page=page on_page_change=move |p| set_page.set(p) />
                <span style="color: var(--text-2);">"Showing sources " {move || (page.get() - 1) * 10 + 1} "–" {move || page.get() * 10} " of 120"</span>
            </div>

            // Uncontrolled, two pages either side and two at each end.
            <Pagination count=40 default_page=20 siblings=2 boundaries=2 label="Search results pages" />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "pagination",
    name: "Pagination",
    group: "Navigation",
    summary: "Move between pages of results. Ellipses stand in for long runs, and the row keeps the same length as the page changes so buttons stay put.",
    demo,
    leptos: include_str!("pagination.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/pagination.rs"),
    parts: &[
        &["Pagination", "root", "nav with aria-label"],
        &["", "list, item", "ul and li around every button"],
        &["", "prev, next", "button with aria-label, disabled at the ends"],
        &["", "page", "button, aria-label=Page N, aria-current=page on the current one"],
        &["", "ellipsis", "span, aria-hidden. Stands in for two or more pages"],
    ],
    keys: &[
        &["Tab / Shift+Tab", "Moves between the buttons"],
        &["Enter / Space", "Goes to the focused page"],
    ],
    props: &[
        &["count", "usize", "Required. Number of pages. page_count(total, size) works it out from items"],
        &["page", "Option<usize>", "Controlled current page, 1-based. Pair with on_page_change"],
        &["default_page", "usize", "Initial page when uncontrolled. Default 1"],
        &["siblings", "usize", "Pages either side of the current one. Default 1"],
        &["boundaries", "usize", "Pages always shown at each end. Default 1"],
        &["label", "String", "aria-label for the nav. Default Pagination"],
        &["on_page_change", "Callback<usize>", "Called with the page the user picked"],
    ],
};
