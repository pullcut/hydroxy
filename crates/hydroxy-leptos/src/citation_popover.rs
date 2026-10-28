//! `<CitationMarker>` and `<CitedText>`: numbered citation markers that preview
//! their source on hover or focus: title, icon or initial, host, the quoted
//! passage with the cited part highlighted, and a link to open it. `CitedText`
//! renders prose with markers placed after the cited spans.

use std::time::Duration;

use leptos::prelude::*;
use hydroxy_headless::citation_popover::{self as h, HoverCardCommand, HoverCardEvent, HoverCardMachine, HoverCardState, Run};
use hydroxy_statechart::Machine;

use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::citation_popover::{CitedSpan, Source};

/// Whether `node` sits inside the element with id `id`.
fn inside(id: &str, node: Option<web_sys::EventTarget>) -> bool {
    let node = node.and_then(|n| leptos::wasm_bindgen::JsCast::dyn_into::<web_sys::Node>(n).ok());
    document().get_element_by_id(id).is_some_and(|el| el.contains(node.as_ref()))
}

/// One citation marker, "[n]", with its preview card.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn CitationMarker(
    /// Zero-based source number; shown as `index + 1`.
    index: usize,
    /// The source it points to.
    source: Source,
) -> impl IntoView {
    let ids = h::ids(&crate::id::next_id(h::SCOPE));
    let state = RwSignal::new(HoverCardMachine::initial(&h::input()));
    let send = move |event: HoverCardEvent| {
        fn run(state: RwSignal<HoverCardState>, event: HoverCardEvent) {
            let Some(step) = state.try_with_untracked(|s| HoverCardMachine::transition(s, event, &h::input())) else {
                return;
            };
            if let Some(next) = step.state
                && state.with_untracked(|s| *s != next)
            {
                state.set(next);
            }
            for command in step.commands {
                if let HoverCardCommand::Wait { id, ms } = command {
                    // Stale waits are ignored by the machine, so there is nothing to cancel.
                    set_timeout(move || run(state, HoverCardEvent::Elapsed(id)), Duration::from_millis(ms.into()));
                }
            }
        }
        run(state, event);
    };
    let open = Memo::new(move |_| state.with(|s| s.open));
    let card_id = StoredValue::new(ids.card.clone());
    let marker_id = StoredValue::new(ids.marker.clone());
    Effect::new(move |_| {
        let id = card_id.get_value();
        if open.get() { show_popover(&id) } else { hide_popover(&id) }
    });

    let (before, quote, after) = h::highlight(&source);
    let host = h::host(&source.url);
    let label = h::marker_label(index, &source);
    let favicon = match source.favicon.clone() {
        Some(src) => view! { <img src=src alt="" width="16" height="16" loading="lazy" /> }.into_any(),
        None => h::initial(&source.url).into_any(),
    };

    view! {
        <sup data-scope=h::SCOPE data-part="marker-wrap">
            <a
                href=source.url.clone()
                target="_blank"
                rel="noopener noreferrer"
                id=ids.marker.clone()
                aria-label=label
                aria-describedby=ids.snippet.clone()
                aria-expanded=move || open.get().to_string()
                aria-controls=ids.card.clone()
                style=h::marker_style(&ids)
                data-scope=h::SCOPE
                data-part="marker"
                data-state=move || if open.get() { "open" } else { "closed" }
                on:pointerenter=move |_| send(HoverCardEvent::PointerEnter)
                on:pointerleave=move |_| send(HoverCardEvent::PointerLeave)
                on:focus=move |_| send(HoverCardEvent::Focus)
                on:blur=move |ev| {
                    // Tab from the marker lands on the card's link: that's staying, not leaving.
                    if !inside(&card_id.get_value(), ev.related_target()) {
                        send(HoverCardEvent::Blur);
                    }
                }
                on:keydown=move |ev| {
                    if let Some(event) = state.with_untracked(|s| h::keydown(&ev.key(), s)) {
                        ev.prevent_default();
                        send(event);
                    }
                }
            >
                {h::marker_text(index)}
            </a>
        </sup>
        <span
            id=ids.card.clone()
            popover="manual"
            data-overlay=""
            data-side="bottom"
            style=h::card_style(&ids)
            data-scope=h::SCOPE
            data-part="card"
            data-state=move || if open.get() { "open" } else { "closed" }
            on:pointerenter=move |_| send(HoverCardEvent::PointerEnter)
            on:pointerleave=move |_| send(HoverCardEvent::PointerLeave)
            on:focusout=move |ev| {
                if !inside(&card_id.get_value(), ev.related_target()) {
                    send(HoverCardEvent::Blur);
                }
            }
            on:keydown=move |ev| {
                if ev.key() == "Escape" {
                    ev.prevent_default();
                    // Back to the marker, which stays where the reader was.
                    crate::focus::focus_id(&marker_id.get_value());
                    send(HoverCardEvent::Escape);
                }
            }
        >
            <span data-scope=h::SCOPE data-part="card-head">
                <span data-scope=h::SCOPE data-part="favicon" aria-hidden="true">{favicon}</span>
                <span data-scope=h::SCOPE data-part="source">
                    <span data-scope=h::SCOPE data-part="title">{source.title.clone()}</span>
                    <span data-scope=h::SCOPE data-part="host">{host}</span>
                </span>
            </span>
            <span data-scope=h::SCOPE data-part="snippet" id=ids.snippet.clone()>
                {before}
                {(!quote.is_empty()).then(|| view! { <mark data-scope=h::SCOPE data-part="quote">{quote}</mark> })}
                {after}
            </span>
            <a
                href=source.url
                target="_blank"
                rel="noopener noreferrer"
                data-scope=h::SCOPE
                data-part="open"
            >
                "Open source"
                <span aria-hidden="true">" \u{2197}"</span>
            </a>
        </span>
    }
}

/// Prose with citation markers after each cited span.
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn CitedText(
    /// The text.
    #[prop(into)]
    text: String,
    /// Which stretches of the text cite which sources (byte offsets).
    spans: Vec<CitedSpan>,
    /// The sources, numbered from 1 in this order.
    sources: Vec<Source>,
) -> impl IntoView {
    let runs = h::runs(&text, &spans, sources.len())
        .into_iter()
        .map(|run| match run {
            Run::Text(t) => t.into_any(),
            Run::Cited { text, sources: cited } => {
                let markers = cited
                    .into_iter()
                    .map(|i| view! { <CitationMarker index=i source=sources[i].clone() /> })
                    .collect_view();
                view! { <span data-scope=h::SCOPE data-part="cited">{text}</span> {markers} }.into_any()
            }
        })
        .collect_view();
    view! { <span data-scope=h::SCOPE data-part="text">{runs}</span> }
}
