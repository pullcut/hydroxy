//! `<TableOfContents>`: an in-page outline of real `#hash` links, with an optional scroll spy.

use leptos::prelude::*;
use hydroxy_headless::table_of_contents as h;

pub use h::Item;

/// A heading outline. Links are plain anchors, so it works without script. The
/// host passes `active_id`, or turns on `spy` and the outline follows the page.
#[component]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
pub fn TableOfContents(
    /// Headings in reading order: id, label and depth.
    #[prop(into)]
    items: Signal<Vec<Item>>,
    /// The current heading, owned by the host. Wins over the spy.
    #[prop(optional, into)]
    active_id: MaybeProp<String>,
    /// Accessible name for the outline. Default "On this page".
    #[prop(optional, into)]
    label: Option<String>,
    /// Pixels of sticky-header clearance: the spy's reading line. Default 72.
    #[prop(default = h::DEFAULT_HEADING_OFFSET)]
    heading_offset: f64,
    /// Follow the page's scroll position to find the current heading.
    #[prop(optional)]
    spy: bool,
    /// Called with the heading id when a link is followed.
    #[prop(optional, into)]
    on_navigate: Option<Callback<String>>,
) -> impl IntoView {
    let label = label.unwrap_or_else(|| h::LABEL.to_string());
    let spied = RwSignal::new(None::<String>);

    let compute = move || {
        let doc = document();
        let ids: Vec<String> = items.with_untracked(|list| h::resolve(list, None).0.into_iter().map(|i| i.id).collect());
        let (mut tops, mut found) = (Vec::new(), Vec::new());
        for id in ids {
            if let Some(el) = doc.get_element_by_id(&id) {
                tops.push(el.get_bounding_client_rect().top());
                found.push(id);
            }
        }
        let at_bottom = web_sys::window()
            .zip(doc.document_element())
            .is_some_and(|(win, root)| {
                let inner = win.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
                let scrolled = win.scroll_y().unwrap_or(0.0);
                let total = f64::from(root.scroll_height());
                total > inner && total - (scrolled + inner) <= 2.0
            });
        spied.set(h::pick_active(&tops, heading_offset, at_bottom).map(|i| found[i].clone()));
    };
    if spy {
        Effect::new(move |_| {
            items.track();
            compute();
        });
        let scroll = window_event_listener(leptos::ev::scroll, move |_| compute());
        let resize = window_event_listener(leptos::ev::resize, move |_| compute());
        on_cleanup(move || {
            scroll.remove();
            resize.remove();
        });
    }

    let current = move || active_id.get().or_else(|| spied.get());
    let resolved = move || items.with(|list| h::resolve(list, current().as_deref()).0);

    view! {
        <nav data-scope=h::SCOPE data-part="root" aria-label=label style=h::root_style(heading_offset)>
            <ol data-scope=h::SCOPE data-part="list">
                <For each=resolved key=|i| (i.id.clone(), i.active) let:item>
                    <li data-scope=h::SCOPE data-part="item" data-level=item.level.to_string() style=format!("--ox-toc-level: {}", item.level)>
                        <a
                            data-scope=h::SCOPE
                            data-part="link"
                            data-table-of-contents-item=item.id.clone()
                            data-active=item.active.then_some("")
                            aria-current=h::aria_current(item.active)
                            href=item.href.clone()
                            on:click={
                                let id = item.id.clone();
                                move |_| {
                                    if let Some(cb) = on_navigate {
                                        cb.run(id.clone());
                                    }
                                }
                            }
                        >
                            {item.label.clone()}
                        </a>
                    </li>
                </For>
            </ol>
        </nav>
    }
}
