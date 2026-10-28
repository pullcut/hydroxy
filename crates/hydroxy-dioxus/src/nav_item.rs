//! `NavItem`: a sidebar row. A link with `href`, otherwise a button.

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};

/// One sidebar destination. `count` is a loud unread badge; `quiet_count` shows on hover.
#[component]
pub fn NavItem(
    /// The row's text. In `compact` mode it becomes the accessible name and tooltip.
    label: String,
    /// Leading icon.
    icon: Option<IconName>,
    /// Renders an `<a>`; without it the row is a button.
    href: Option<String>,
    /// The current page. Sets `aria-current="page"`.
    #[props(default)]
    active: ReadSignal<bool>,
    /// Unread count, as an amethyst badge.
    #[props(default)]
    count: ReadSignal<Option<u32>>,
    /// Total count, shown only on hover or focus.
    #[props(default)]
    quiet_count: ReadSignal<Option<u32>>,
    /// Indented child row.
    #[props(default)]
    sub: bool,
    /// Icon only.
    #[props(default)]
    compact: bool,
    /// Called on click.
    on_click: Option<EventHandler<()>>,
) -> Element {
    let unread = count().filter(|n| *n > 0);
    let aria_label = compact.then(|| unread.map_or_else(|| label.clone(), |n| format!("{label}, {n} unread")));
    let title = compact.then(|| label.clone());
    let current = active().then_some("page");
    let quiet = if compact { None } else { quiet_count() };
    let inner = rsx! {
        if let Some(name) = icon {
            Icon { name, size: if sub { 14 } else { 16 } }
        }
        if !compact {
            span { "data-part": "label", "{label}" }
        }
        if let Some(n) = unread {
            span { "data-part": "count",
                "{n}"
                span { "data-part": "sr", " unread" }
            }
        }
        if let Some(n) = quiet {
            span { "data-part": "quiet", "{n}" }
        }
    };
    let click = move |_| {
        if let Some(handler) = on_click {
            handler.call(());
        }
    };
    match href {
        Some(href) => rsx! {
            a {
                href,
                "data-scope": "nav-item",
                "data-sub": sub.then_some(""),
                "data-compact": compact.then_some(""),
                "aria-current": current,
                "aria-label": aria_label,
                title,
                onclick: click,
                {inner}
            }
        },
        None => rsx! {
            button {
                r#type: "button",
                "data-scope": "nav-item",
                "data-sub": sub.then_some(""),
                "data-compact": compact.then_some(""),
                "aria-current": current,
                "aria-label": aria_label,
                title,
                onclick: click,
                {inner}
            }
        },
    }
}
