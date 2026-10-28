//! `<MessageScroller>`: a scrolling message list that follows new messages until the
//! reader scrolls up, then offers a "jump to latest" button with a count of what they
//! missed. Older messages added above keep the reader's place.

use std::time::Duration;

use hydroxy_headless::message_scroller::{
    self as h, MessageScrollerCommand, MessageScrollerEvent, MessageScrollerInput, MessageScrollerMachine,
};
use hydroxy_statechart::Machine;
use leptos::html::Div;
use leptos::prelude::*;

/// A scrolling list. Tell it how the list changes with `count`, `last_key` and `revision`.
#[component]
pub fn MessageScroller(
    /// How many messages the list holds.
    #[prop(optional, into)]
    count: Signal<usize>,
    /// Key of the newest message. A new key means messages were appended; the same key
    /// with a larger `count` means older ones were prepended.
    #[prop(optional, into)]
    last_key: MaybeProp<String>,
    /// Bump when the newest message grows in place, such as streaming text.
    #[prop(optional, into)]
    revision: Signal<usize>,
    /// Distance from the bottom (px) that still counts as being at the bottom.
    #[prop(default = 48.0)]
    threshold: f64,
    /// Accessible name for the list.
    #[prop(default = "Messages".into(), into)]
    label: String,
    /// The messages.
    children: Children,
) -> impl IntoView {
    let input = MessageScrollerInput { threshold };
    let state = RwSignal::new(MessageScrollerMachine::initial(&input));
    let viewport = NodeRef::<Div>::new();
    // Height at the last scroll or update: the baseline for keeping place on prepend.
    let known_height = StoredValue::new(0.0_f64);

    let apply = move |commands: Vec<MessageScrollerCommand>| {
        // Wait for the list to render before measuring.
        set_timeout(
            move || {
                let Some(el) = viewport.try_get_untracked().flatten() else { return };
                for command in commands {
                    match command {
                        MessageScrollerCommand::ScrollToBottom => el.set_scroll_top(el.scroll_height()),
                        MessageScrollerCommand::KeepPosition => {
                            #[allow(clippy::cast_possible_truncation, reason = "pixel deltas fit in i32")]
                            let grown = (f64::from(el.scroll_height())
                                - known_height.try_get_value().unwrap_or_default())
                                as i32;
                            el.set_scroll_top(el.scroll_top() + grown);
                        }
                    }
                }
                known_height.try_set_value(f64::from(el.scroll_height()));
            },
            Duration::ZERO,
        );
    };
    let send = move |event: MessageScrollerEvent| {
        let Some(step) = state.try_with_untracked(|s| MessageScrollerMachine::transition(s, event, &input)) else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        if !step.commands.is_empty() {
            apply(step.commands);
        }
    };

    Effect::new(move |prev: Option<()>| {
        send(MessageScrollerEvent::Items { count: count.get(), last_key: last_key.get() });
        if prev.is_none() {
            apply(vec![MessageScrollerCommand::ScrollToBottom]);
        } else {
            // Re-measure so a later prepend has a current baseline.
            apply(Vec::new());
        }
    });
    Effect::new(move |prev: Option<()>| {
        revision.track();
        if prev.is_some() {
            send(MessageScrollerEvent::Grew);
            apply(Vec::new());
        }
    });

    let attrs = Memo::new(move |_| state.with(h::viewport_attrs));
    view! {
        <div data-scope=h::SCOPE data-part="root" data-state=move || attrs.get().data_state>
            <div
                node_ref=viewport
                data-scope=h::SCOPE
                data-part="viewport"
                data-state=move || attrs.get().data_state
                role=move || attrs.get().role
                aria-live=move || attrs.get().aria_live
                aria-label=label
                tabindex=move || attrs.get().tabindex
                on:scroll=move |_| {
                    let Some(el) = viewport.get_untracked() else { return };
                    let (height, top, view) = (
                        f64::from(el.scroll_height()),
                        f64::from(el.scroll_top()),
                        f64::from(el.client_height()),
                    );
                    known_height.set_value(height);
                    send(MessageScrollerEvent::Scrolled { from_bottom: h::from_bottom(height, top, view) });
                }
            >
                {children()}
            </div>
            {move || {
                state
                    .with(h::jump_visible)
                    .then(|| {
                        view! {
                            <button
                                type="button"
                                data-scope=h::SCOPE
                                data-part="jump"
                                on:click=move |_| send(MessageScrollerEvent::JumpToLatest)
                            >
                                {move || h::jump_label(state.with(|s| s.unread))}
                            </button>
                        }
                    })
            }}
        </div>
    }
}
