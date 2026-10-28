//! `<Breadcrumb>`: the trail of pages leading to this one. When it runs out of
//! room, middle pages fold into a "…" button that lists them; the first page and
//! the last two always show. Any crumb can also list the pages beside it.

use hydroxy_headless::breadcrumb::{self as h, Crumb, Link};
use hydroxy_headless::overlay::{Align, Placement, Side};
use leptos::prelude::*;

use crate::button::ButtonVariant;
use crate::icon::{Icon, IconName};
use crate::js;
use crate::popover::{Popover, PopoverContent, PopoverTrigger};

pub use hydroxy_headless::breadcrumb::{Crumb as BreadcrumbItem, Link as BreadcrumbLink};

const MENU_PLACEMENT: Placement = Placement::new(Side::Bottom, Align::Start);

fn separator() -> impl IntoView {
    view! {
        <span data-scope=h::SCOPE data-part="separator" aria-hidden="true">
            <Icon name=IconName::ChevronRight size=14 />
        </span>
    }
}

/// A popover listing links, opened by a small button named `label`.
fn link_menu(label: String, glyph: AnyView, links: Vec<Link>) -> impl IntoView {
    view! {
        <span data-scope=h::SCOPE data-part="menu-trigger">
            <Popover placement=MENU_PLACEMENT>
                <PopoverTrigger variant=ButtonVariant::Ghost>
                    {glyph}
                    <span data-scope=h::SCOPE data-part="sr">{label}</span>
                </PopoverTrigger>
                <PopoverContent>
                    <ul data-scope=h::SCOPE data-part="menu">
                        {links
                            .into_iter()
                            .map(|l| view! { <li><a data-scope=h::SCOPE data-part="menu-link" href=l.href>{l.label}</a></li> })
                            .collect_view()}
                    </ul>
                </PopoverContent>
            </Popover>
        </span>
    }
}

fn crumb(c: Crumb, first: bool, current: bool) -> impl IntoView {
    let siblings = (!c.siblings.is_empty()).then(|| {
        let glyph = view! { <Icon name=IconName::ChevronDown size=12 /> }.into_any();
        link_menu(h::siblings_label(&c.label), glyph, c.siblings)
    });
    let inner = view! {
        {c.icon.map(|i| view! { <Icon name=i size=14 /> })}
        <span data-scope=h::SCOPE data-part="label">{c.label}</span>
    };
    let link = match c.href {
        Some(href) => view! {
            <a data-scope=h::SCOPE data-part="link" href=href aria-current=current.then_some("page")>{inner}</a>
        }
        .into_any(),
        None => view! {
            <span data-scope=h::SCOPE data-part="link" aria-current=current.then_some("page")>{inner}</span>
        }
        .into_any(),
    };
    view! {
        <li data-scope=h::SCOPE data-part="item" data-current=current.then_some("")>
            {(!first).then(separator)}
            {link}
            {siblings}
        </li>
    }
}

/// The trail. The last item is the current page.
#[component]
pub fn Breadcrumb(
    /// The pages from the root to here.
    #[prop(into)]
    items: Signal<Vec<Crumb>>,
) -> impl IntoView {
    let id = crate::id::next_id(h::SCOPE);
    let folded = RwSignal::new((0usize, 0usize));
    let nav = NodeRef::<leptos::html::Nav>::new();
    let watching = StoredValue::new_local(None::<js::Watch>);
    let root = id.clone();
    Effect::new(move |_| {
        if nav.get().is_none() || watching.with_value(Option::is_some) {
            return;
        }
        let w = js::watch(h::MEASURE_JS, &root, move |report| {
            if let Some((available, ellipsis, widths)) = h::parse_measure(&report) {
                let r = h::collapse(&widths, available, ellipsis);
                if folded.try_get_untracked() != Some((r.start, r.end)) {
                    let _ = folded.try_set((r.start, r.end));
                }
            }
        });
        js::keep(watching, w);
    });

    let visible = move || {
        let list = items.get();
        let (start, end) = folded.get();
        let final_index = list.len().saturating_sub(1);
        let hidden: Vec<Link> = list[start.min(list.len())..end.min(list.len())]
            .iter()
            .filter_map(|c| c.href.clone().map(|href| Link::new(c.label.clone(), href)))
            .collect();
        let mut out = Vec::new();
        for (i, c) in list.into_iter().enumerate() {
            if i == start && end > start {
                let glyph = view! { <span aria-hidden="true">"…"</span> }.into_any();
                let label = h::ellipsis_label(end - start);
                out.push(
                    view! {
                        <li data-scope=h::SCOPE data-part="item" data-ellipsis="">
                            {separator()}
                            {link_menu(label, glyph, hidden.clone())}
                        </li>
                    }
                    .into_any(),
                );
            }
            if (start..end).contains(&i) {
                continue;
            }
            out.push(crumb(c, i == 0, i == final_index).into_any());
        }
        out
    };

    // An invisible copy of the whole trail, measured to decide what folds.
    let measure = move || {
        let list = items.get();
        let final_index = list.len().saturating_sub(1);
        list.into_iter()
            .enumerate()
            .map(|(i, c)| {
                let has_siblings = !c.siblings.is_empty();
                view! {
                    <li data-scope=h::SCOPE data-part="item" data-current=(i == final_index).then_some("")>
                        {(i > 0).then(separator)}
                        <span data-scope=h::SCOPE data-part="link">
                            {c.icon.map(|ic| view! { <Icon name=ic size=14 /> })}
                            <span data-scope=h::SCOPE data-part="label">{c.label}</span>
                        </span>
                        {has_siblings.then(|| view! { <span data-scope=h::SCOPE data-part="menu-trigger" data-measure=""></span> })}
                    </li>
                }
            })
            .collect_view()
    };

    view! {
        <nav node_ref=nav id=id aria-label=h::LABEL data-scope=h::SCOPE data-part="root">
            <ol data-scope=h::SCOPE data-part="list">{visible}</ol>
            <ol data-scope=h::SCOPE data-part="measure" aria-hidden="true" inert>
                {measure}
                <li data-scope=h::SCOPE data-part="ellipsis-item">
                    {separator()}
                    <span data-scope=h::SCOPE data-part="menu-trigger" data-measure=""></span>
                </li>
            </ol>
        </nav>
    }
}
