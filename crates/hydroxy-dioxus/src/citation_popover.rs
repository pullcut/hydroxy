//! `CitationMarker` and `CitedText`: numbered citation markers that preview
//! their source on hover or focus: title, icon or initial, host, the quoted
//! passage with the cited part highlighted, and a link to open it. `CitedText`
//! renders prose with markers placed after the cited spans.

use dioxus::prelude::*;
use hydroxy_headless::citation_popover::{
    self as h, HoverCardCommand, HoverCardEvent, HoverCardMachine, HoverCardState, Run,
};
use hydroxy_headless::markdown::safe_url;
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::focus::focus_id;
use crate::js;
use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::citation_popover::{CitedSpan, Source};

fn send(state: Signal<HoverCardState>, input: Signal<h::HoverCardInput>, event: HoverCardEvent) {
    for command in drive::<HoverCardMachine>(state, input, event) {
        // The clock runs through eval; stale waits are ignored by the machine.
        if let HoverCardCommand::Wait { id, ms } = command {
            spawn(async move {
                js::sleep(ms).await;
                send(state, input, HoverCardEvent::Elapsed(id));
            });
        }
    }
}

/// Blur, unless focus moved into the card (Tab from the marker reaches its link).
fn blur_unless_inside(state: Signal<HoverCardState>, input: Signal<h::HoverCardInput>, card: String) {
    spawn(async move {
        js::sleep(0).await;
        let script = js::call(
            &[("card", card.as_str())],
            "return !!document.getElementById(card)?.contains(document.activeElement);",
        );
        if let Ok(false) = document::eval(&script).join::<bool>().await {
            send(state, input, HoverCardEvent::Blur);
        }
    });
}

/// One citation marker, "[n]", with its preview card.
#[component]
pub fn CitationMarker(
    /// Zero-based source number; shown as `index + 1`.
    index: usize,
    /// The source it points to.
    source: Source,
) -> Element {
    let ids = use_hook(|| h::ids(&hydroxy_headless::next_id(h::SCOPE)));
    let input = use_signal(h::input);
    let state = use_signal(|| HoverCardMachine::initial(&h::input()));
    let open = use_memo(move || state.read().open.value);
    let card = ids.card.clone();
    use_effect(move || if open() { show_popover(&card) } else { hide_popover(&card) });

    let (before, quote, after) = h::highlight(&source);
    // Unsafe schemes (`javascript:` and friends) get no href, like markdown links.
    let href = safe_url(&source.url);
    let marker_tabindex = href.is_none().then_some("0");
    let data_state = if open() { "open" } else { "closed" };
    let (card_a, card_b, marker) = (ids.card.clone(), ids.card.clone(), ids.marker.clone());

    rsx! {
        sup { "data-scope": h::SCOPE, "data-part": "marker-wrap",
            a {
                href: href.clone(),
                tabindex: marker_tabindex,
                target: "_blank",
                rel: "noopener noreferrer",
                id: ids.marker.clone(),
                "aria-label": h::marker_label(index, &source),
                "aria-describedby": ids.snippet.clone(),
                "aria-expanded": open().to_string(),
                "aria-controls": ids.card.clone(),
                style: h::marker_style(&ids),
                "data-scope": h::SCOPE,
                "data-part": "marker",
                "data-state": data_state,
                onpointerenter: move |_| send(state, input, HoverCardEvent::PointerEnter),
                onpointerleave: move |_| send(state, input, HoverCardEvent::PointerLeave),
                onfocus: move |_| send(state, input, HoverCardEvent::Focus),
                onblur: move |_| blur_unless_inside(state, input, card_a.clone()),
                onkeydown: move |e| {
                    if let Some(event) = h::keydown(&e.key().to_string(), &state.peek()) {
                        e.prevent_default();
                        send(state, input, event);
                    }
                },
                {h::marker_text(index)}
            }
        }
        span {
            id: ids.card.clone(),
            "popover": "manual",
            "data-overlay": "",
            "data-side": "bottom",
            style: h::card_style(&ids),
            "data-scope": h::SCOPE,
            "data-part": "card",
            "data-state": data_state,
            onpointerenter: move |_| send(state, input, HoverCardEvent::PointerEnter),
            onpointerleave: move |_| send(state, input, HoverCardEvent::PointerLeave),
            onfocusout: move |_| blur_unless_inside(state, input, card_b.clone()),
            onkeydown: move |e| {
                if e.key().to_string() == "Escape" {
                    e.prevent_default();
                    // Back to the marker, which stays where the reader was.
                    focus_id(&marker);
                    send(state, input, HoverCardEvent::Escape);
                }
            },
            span { "data-scope": h::SCOPE, "data-part": "card-head",
                span { "data-scope": h::SCOPE, "data-part": "favicon", "aria-hidden": "true",
                    if let Some(src) = source.favicon.clone() {
                        img { src, alt: "", width: "16", height: "16", loading: "lazy" }
                    } else {
                        {h::initial(&source.url)}
                    }
                }
                span { "data-scope": h::SCOPE, "data-part": "source",
                    span { "data-scope": h::SCOPE, "data-part": "title", "{source.title}" }
                    span { "data-scope": h::SCOPE, "data-part": "host", {h::host(&source.url)} }
                }
            }
            span { "data-scope": h::SCOPE, "data-part": "snippet", id: ids.snippet.clone(),
                "{before}"
                if !quote.is_empty() {
                    mark { "data-scope": h::SCOPE, "data-part": "quote", "{quote}" }
                }
                "{after}"
            }
            a {
                href,
                target: "_blank",
                rel: "noopener noreferrer",
                "data-scope": h::SCOPE,
                "data-part": "open",
                "Open source"
                span { "aria-hidden": "true", " \u{2197}" }
            }
        }
    }
}

/// Prose with citation markers after each cited span.
#[component]
pub fn CitedText(
    /// The text.
    text: String,
    /// Which stretches of the text cite which sources (byte offsets).
    spans: Vec<CitedSpan>,
    /// The sources, numbered from 1 in this order.
    sources: Vec<Source>,
) -> Element {
    let nodes = h::runs(&text, &spans, sources.len()).into_iter().map(|run| match run {
        Run::Text(t) => rsx! { "{t}" },
        Run::Cited { text, sources: cited } => rsx! {
            span { "data-scope": h::SCOPE, "data-part": "cited", "{text}" }
            for i in cited {
                CitationMarker { index: i, source: sources[i].clone() }
            }
        },
    });
    rsx! {
        span { "data-scope": h::SCOPE, "data-part": "text",
            for node in nodes {
                {node}
            }
        }
    }
}
