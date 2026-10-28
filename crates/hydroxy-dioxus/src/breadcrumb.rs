//! `Breadcrumb`: the trail of pages leading to this one. When it runs out of
//! room, middle pages fold into a "…" button that lists them; the first page and
//! the last two always show. Any crumb can also list the pages beside it.

use dioxus::prelude::*;
use hydroxy_headless::breadcrumb::{self as h, Crumb, Link};
use hydroxy_headless::overlay::{Align, Placement, Side};

use crate::button::ButtonVariant;
use crate::icon::{Icon, IconName};
use crate::js;
use crate::popover::{Popover, PopoverContent, PopoverTrigger};

pub use hydroxy_headless::breadcrumb::{Crumb as BreadcrumbItem, Link as BreadcrumbLink};

const MENU_PLACEMENT: Placement = Placement::new(Side::Bottom, Align::Start);

fn separator() -> Element {
    rsx! {
        span { "data-scope": h::SCOPE, "data-part": "separator", "aria-hidden": "true",
            Icon { name: IconName::ChevronRight, size: 14 }
        }
    }
}

/// A popover listing links, opened by a small button named `label`.
fn link_menu(label: &str, glyph: Element, links: Vec<Link>) -> Element {
    rsx! {
        span { "data-scope": h::SCOPE, "data-part": "menu-trigger",
            Popover { placement: MENU_PLACEMENT,
                PopoverTrigger { variant: ButtonVariant::Ghost,
                    {glyph}
                    span { "data-scope": h::SCOPE, "data-part": "sr", "{label}" }
                }
                PopoverContent {
                    ul { "data-scope": h::SCOPE, "data-part": "menu",
                        for l in links {
                            li { a { "data-scope": h::SCOPE, "data-part": "menu-link", href: l.href, "{l.label}" } }
                        }
                    }
                }
            }
        }
    }
}

fn crumb(c: &Crumb, first: bool, current: bool) -> Element {
    let siblings = (!c.siblings.is_empty()).then(|| {
        let glyph = rsx! { Icon { name: IconName::ChevronDown, size: 12 } };
        link_menu(&h::siblings_label(&c.label), glyph, c.siblings.clone())
    });
    let aria_current = current.then_some("page");
    let inner = rsx! {
        if let Some(i) = c.icon { Icon { name: i, size: 14 } }
        span { "data-scope": h::SCOPE, "data-part": "label", "{c.label}" }
    };
    rsx! {
        li { "data-scope": h::SCOPE, "data-part": "item", "data-current": current.then_some(""),
            if !first { {separator()} }
            if let Some(href) = c.href.clone() {
                a { "data-scope": h::SCOPE, "data-part": "link", href, "aria-current": aria_current, {inner} }
            } else {
                span { "data-scope": h::SCOPE, "data-part": "link", "aria-current": aria_current, {inner} }
            }
            {siblings}
        }
    }
}

/// The trail. The last item is the current page.
#[component]
pub fn Breadcrumb(
    /// The pages from the root to here.
    items: ReadSignal<Vec<Crumb>>,
) -> Element {
    let id = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let mut folded = use_signal(|| (0usize, 0usize));
    let watch = js::use_watch(h::MEASURE_JS, id.clone(), move |report| {
        if let Some((available, ellipsis, widths)) = h::parse_measure(&report) {
            let r = h::collapse(&widths, available, ellipsis);
            if *folded.peek() != (r.start, r.end) {
                folded.set((r.start, r.end));
            }
        }
    });

    let list = items();
    let (start, end) = folded();
    let len = list.len();
    let last = len.saturating_sub(1);
    let hidden: Vec<Link> = list[start.min(len)..end.min(len)]
        .iter()
        .filter_map(|c| c.href.clone().map(|href| Link::new(c.label.clone(), href)))
        .collect();
    let mut visible: Vec<Element> = Vec::new();
    for (i, c) in list.iter().enumerate() {
        if i == start && end > start {
            let glyph = rsx! { span { "aria-hidden": "true", "…" } };
            let menu = link_menu(&h::ellipsis_label(end - start), glyph, hidden.clone());
            visible.push(rsx! {
                li { "data-scope": h::SCOPE, "data-part": "item", "data-ellipsis": "",
                    {separator()}
                    {menu}
                }
            });
        }
        if (start..end).contains(&i) {
            continue;
        }
        visible.push(crumb(c, i == 0, i == last));
    }

    rsx! {
        nav {
            id: "{id}",
            "aria-label": h::LABEL,
            "data-scope": h::SCOPE,
            "data-part": "root",
            onmounted: move |_| {
                let _ = watch.send("start");
            },
            ol { "data-scope": h::SCOPE, "data-part": "list",
                for v in visible { {v} }
            }
            // An invisible copy of the whole trail, measured to decide what folds.
            ol { "data-scope": h::SCOPE, "data-part": "measure", "aria-hidden": "true", "inert": "true",
                for (i, c) in list.into_iter().enumerate() {
                    li { "data-scope": h::SCOPE, "data-part": "item", "data-current": (i == last).then_some(""),
                        if i > 0 { {separator()} }
                        span { "data-scope": h::SCOPE, "data-part": "link",
                            if let Some(ic) = c.icon { Icon { name: ic, size: 14 } }
                            span { "data-scope": h::SCOPE, "data-part": "label", "{c.label}" }
                        }
                        if !c.siblings.is_empty() {
                            span { "data-scope": h::SCOPE, "data-part": "menu-trigger", "data-measure": "" }
                        }
                    }
                }
                li { "data-scope": h::SCOPE, "data-part": "ellipsis-item",
                    {separator()}
                    span { "data-scope": h::SCOPE, "data-part": "menu-trigger", "data-measure": "" }
                }
            }
        }
    }
}
