use dioxus::prelude::*;
use oxide_dioxus::pagination::Pagination;

#[component]
pub fn PaginationDemo() -> Element {
    let mut page = use_signal(|| 5_usize);
    let (first, last) = ((page() - 1) * 10 + 1, page() * 10);

    rsx! {
        div { style: "display: grid; gap: var(--space-6);",
            // Controlled, with the page shown alongside.
            div { style: "display: grid; gap: var(--space-3);",
                Pagination { count: 12_usize, page: Some(page()), on_page_change: move |p| page.set(p) }
                span { style: "color: var(--text-2);", "Showing sources {first}–{last} of 120" }
            }

            // Uncontrolled, two pages either side and two at each end.
            Pagination { count: 40_usize, default_page: 20, siblings: 2, boundaries: 2, label: "Search results pages" }
        }
    }
}
