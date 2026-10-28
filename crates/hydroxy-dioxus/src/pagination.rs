//! `Pagination`: previous, page and next buttons inside a labelled `<nav>`.
//! The current page carries `aria-current="page"`; gaps show as an ellipsis
//! hidden from assistive tech.

use dioxus::prelude::*;
use hydroxy_headless::pagination::{self as h, PageItem};

use crate::icon::{Icon, IconName};

/// Page navigation for `count` pages.
#[component]
pub fn Pagination(
    /// Total number of pages. See `hydroxy_headless::pagination::page_count` to get it from an item count.
    count: ReadSignal<usize>,
    /// Controlled current page, 1-based. Pair with `on_page_change`.
    #[props(default)]
    page: ReadSignal<Option<usize>>,
    /// Initial page when uncontrolled. Default 1.
    #[props(default = 1)]
    default_page: usize,
    /// Pages shown either side of the current one. Default 1.
    #[props(default = 1)]
    siblings: usize,
    /// Pages always shown at each end. Default 1, minimum 1.
    #[props(default = 1)]
    boundaries: usize,
    /// `aria-label` for the `<nav>`. Default "Pagination".
    #[props(default = h::DEFAULT_LABEL.to_string())]
    label: String,
    /// Called with the page the user picked.
    on_page_change: Option<EventHandler<usize>>,
) -> Element {
    let mut own = use_signal(|| default_page);
    let current = h::clamp_page(page().unwrap_or(own()), count());
    let mut go = move |to: usize| {
        if to == current {
            return;
        }
        if page.peek().is_none() {
            own.set(to);
        }
        if let Some(handler) = on_page_change {
            handler.call(to);
        }
    };
    rsx! {
        nav { "aria-label": label, "data-scope": h::SCOPE, "data-part": "root",
            ul { "data-scope": h::SCOPE, "data-part": "list",
                li { "data-scope": h::SCOPE, "data-part": "item",
                    button {
                        r#type: "button",
                        "aria-label": "Previous page",
                        disabled: !h::has_prev(current),
                        "data-scope": h::SCOPE,
                        "data-part": "prev",
                        onclick: move |_| go(current - 1),
                        Icon { name: IconName::ChevronRight, size: 14 }
                    }
                }
                // Unkeyed on purpose: slots keep their elements as the page moves,
                // so a focused button stays focused.
                for item in h::items(current, count(), siblings, boundaries) {
                    li { "data-scope": h::SCOPE, "data-part": "item",
                        match item {
                            PageItem::Page(n) => rsx! {
                                button {
                                    r#type: "button",
                                    "aria-label": h::page_label(n),
                                    "aria-current": (n == current).then_some("page"),
                                    "data-scope": h::SCOPE,
                                    "data-part": "page",
                                    onclick: move |_| go(n),
                                    "{n}"
                                }
                            },
                            PageItem::Ellipsis => rsx! {
                                span { "aria-hidden": "true", "data-scope": h::SCOPE, "data-part": "ellipsis", "…" }
                            },
                        }
                    }
                }
                li { "data-scope": h::SCOPE, "data-part": "item",
                    button {
                        r#type: "button",
                        "aria-label": "Next page",
                        disabled: !h::has_next(current, count()),
                        "data-scope": h::SCOPE,
                        "data-part": "next",
                        onclick: move |_| go(current + 1),
                        Icon { name: IconName::ChevronRight, size: 14 }
                    }
                }
            }
        }
    }
}
