//! `<Pagination>`: previous, page and next buttons inside a labelled `<nav>`.
//! The current page carries `aria-current="page"`; gaps show as an ellipsis
//! hidden from assistive tech.

use leptos::prelude::*;
use oxide_headless::pagination::{self as h, PageItem};

use crate::icon::{Icon, IconName};

/// Page navigation for `count` pages.
#[component]
pub fn Pagination(
    /// Total number of pages. See `oxide_headless::pagination::page_count` to get it from an item count.
    #[prop(into)]
    count: Signal<usize>,
    /// Controlled current page, 1-based. Pair with `on_page_change`.
    #[prop(optional, into)]
    page: MaybeProp<usize>,
    /// Initial page when uncontrolled. Default 1.
    #[prop(default = 1)]
    default_page: usize,
    /// Pages shown either side of the current one. Default 1.
    #[prop(default = 1)]
    siblings: usize,
    /// Pages always shown at each end. Default 1, minimum 1.
    #[prop(default = 1)]
    boundaries: usize,
    /// `aria-label` for the `<nav>`. Default "Pagination".
    #[prop(default = h::DEFAULT_LABEL.into(), into)]
    label: String,
    /// Called with the page the user picked.
    #[prop(optional, into)]
    on_page_change: Option<Callback<usize>>,
) -> impl IntoView {
    let own = RwSignal::new(default_page);
    let current = Memo::new(move |_| h::clamp_page(page.get().unwrap_or_else(|| own.get()), count.get()));
    let items = Memo::new(move |_| h::items(current.get(), count.get(), siblings, boundaries));
    let go = move |to: usize| {
        if to == current.get_untracked() {
            return;
        }
        if page.get_untracked().is_none() {
            own.set(to);
        }
        if let Some(cb) = on_page_change {
            cb.run(to);
        }
    };
    // One `<li>` per slot, keyed by position: the slots keep their elements as the
    // page moves, so a focused button stays focused.
    let slot = move |i: usize| {
        let item = Memo::new(move |_| items.with(|v| v.get(i).copied()));
        let number = move || match item.get() {
            Some(PageItem::Page(n)) => n,
            _ => 0,
        };
        let is_page = move || matches!(item.get(), Some(PageItem::Page(_)));
        view! {
            <li data-scope=h::SCOPE data-part="item">
                <button
                    type="button"
                    hidden=move || !is_page()
                    aria-label=move || h::page_label(number())
                    aria-current=move || (number() == current.get()).then_some("page")
                    data-scope=h::SCOPE
                    data-part="page"
                    on:click=move |_| go(number())
                >
                    {number}
                </button>
                <span hidden=is_page aria-hidden="true" data-scope=h::SCOPE data-part="ellipsis">
                    "…"
                </span>
            </li>
        }
    };
    view! {
        <nav aria-label=label data-scope=h::SCOPE data-part="root">
            <ul data-scope=h::SCOPE data-part="list">
                <li data-scope=h::SCOPE data-part="item">
                    <button
                        type="button"
                        aria-label="Previous page"
                        disabled=move || !h::has_prev(current.get())
                        data-scope=h::SCOPE
                        data-part="prev"
                        on:click=move |_| go(current.get_untracked() - 1)
                    >
                        <Icon name=IconName::ChevronRight size=14 />
                    </button>
                </li>
                <For each=move || 0..items.with(Vec::len) key=|i| *i let:i>
                    {slot(i)}
                </For>
                <li data-scope=h::SCOPE data-part="item">
                    <button
                        type="button"
                        aria-label="Next page"
                        disabled=move || !h::has_next(current.get(), count.get())
                        data-scope=h::SCOPE
                        data-part="next"
                        on:click=move |_| go(current.get_untracked() + 1)
                    >
                        <Icon name=IconName::ChevronRight size=14 />
                    </button>
                </li>
            </ul>
        </nav>
    }
}
