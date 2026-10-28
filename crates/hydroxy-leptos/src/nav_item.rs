//! `<NavItem>`: a sidebar row. A link with `href`, otherwise a button.

use leptos::either::Either;
use leptos::prelude::*;

use crate::icon::{Icon, IconName};

/// One sidebar destination. `count` is a loud unread badge; `quiet_count` shows on hover.
#[component]
pub fn NavItem(
    /// The row's text. In `compact` mode it becomes the accessible name and tooltip.
    #[prop(into)]
    label: String,
    /// Leading icon.
    #[prop(optional, into)]
    icon: Option<IconName>,
    /// Renders an `<a>`; without it the row is a button.
    #[prop(optional, into)]
    href: Option<String>,
    /// The current page. Sets `aria-current="page"`.
    #[prop(optional, into)]
    active: Signal<bool>,
    /// Unread count, as an amethyst badge.
    #[prop(optional, into)]
    count: MaybeProp<u32>,
    /// Total count, shown only on hover or focus.
    #[prop(optional, into)]
    quiet_count: MaybeProp<u32>,
    /// Indented child row.
    #[prop(optional)]
    sub: bool,
    /// Icon only.
    #[prop(optional)]
    compact: bool,
    /// Called on click.
    #[prop(optional, into)]
    on_click: Option<Callback<()>>,
) -> impl IntoView {
    let unread = move || count.get().filter(|n| *n > 0);
    let aria_label = {
        let label = label.clone();
        move || compact.then(|| unread().map_or_else(|| label.clone(), |n| format!("{label}, {n} unread")))
    };
    let title = compact.then(|| label.clone());
    let inner = move || {
        view! {
            {icon.map(|name| view! { <Icon name size=if sub { 14 } else { 16 } /> })}
            {(!compact).then(|| view! { <span data-part="label">{label.clone()}</span> })}
            {move || {
                unread().map(|n| {
                    view! {
                        <span data-part="count">
                            {n}
                            <span data-part="sr">" unread"</span>
                        </span>
                    }
                })
            }}
            {move || (!compact).then(|| quiet_count.get().map(|n| view! { <span data-part="quiet">{n}</span> })).flatten()}
        }
    };
    let current = move || active.get().then_some("page");
    let click = move |_| {
        if let Some(cb) = on_click {
            cb.run(());
        }
    };
    match href {
        Some(href) => Either::Left(view! {
            <a
                href=href
                data-scope="nav-item"
                data-sub=sub.then_some("")
                data-compact=compact.then_some("")
                aria-current=current
                aria-label=aria_label
                title=title
                on:click=click
            >
                {inner()}
            </a>
        }),
        None => Either::Right(view! {
            <button
                type="button"
                data-scope="nav-item"
                data-sub=sub.then_some("")
                data-compact=compact.then_some("")
                aria-current=current
                aria-label=aria_label
                title=title
                on:click=click
            >
                {inner()}
            </button>
        }),
    }
}
