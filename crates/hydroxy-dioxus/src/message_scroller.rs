//! `MessageScroller`: a scrolling message list that follows new messages until the
//! reader scrolls up, then offers a "jump to latest" button with a count of what they
//! missed. Older messages added above keep the reader's place.

use dioxus::prelude::*;
use hydroxy_headless::message_scroller::{
    self as h, MessageScrollerCommand, MessageScrollerEvent, MessageScrollerInput, MessageScrollerMachine,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;

/// A scrolling list. Tell it how the list changes with `count`, `last_key` and `revision`.
#[component]
pub fn MessageScroller(
    /// How many messages the list holds.
    #[props(default)]
    count: ReadSignal<usize>,
    /// Key of the newest message. A new key means messages were appended; the same key
    /// with a larger `count` means older ones were prepended.
    #[props(default)]
    last_key: ReadSignal<Option<String>>,
    /// Bump when the newest message grows in place, such as streaming text.
    #[props(default)]
    revision: ReadSignal<usize>,
    /// Distance from the bottom (px) that still counts as being at the bottom.
    #[props(default = 48.0)]
    threshold: f64,
    /// Accessible name for the list.
    #[props(default = "Messages".to_string())]
    label: String,
    /// The messages.
    children: Element,
) -> Element {
    let input = use_memo(use_reactive!(|threshold| MessageScrollerInput { threshold }));
    let state = use_signal(|| MessageScrollerMachine::initial(&input.peek()));
    let id = use_signal(|| hydroxy_headless::next_id(h::SCOPE));
    // Height at the last scroll or update: the baseline for keeping place on prepend.
    let mut known_height = use_signal(|| 0.0_f64);
    let mut mounted = use_signal(|| false);

    let apply = move |commands: Vec<MessageScrollerCommand>| {
        let dom_id = id.cloned();
        let baseline = *known_height.peek();
        spawn(async move {
            let mut body = String::new();
            for command in commands {
                body.push_str(match command {
                    MessageScrollerCommand::ScrollToBottom => "el.scrollTop = el.scrollHeight;",
                    MessageScrollerCommand::KeepPosition => "el.scrollTop += el.scrollHeight - BASE;",
                });
            }
            // Wait a tick for the list to render before measuring.
            let js = format!(
                "const el = document.getElementById({dom_id});
                if (el) {{
                    await new Promise((r) => setTimeout(r, 0));
                    const BASE = {baseline};
                    {body}
                    dioxus.send(el.scrollHeight);
                }}",
                dom_id = crate::js::js_string(&dom_id)
            );
            if let Ok(height) = document::eval(&js).recv::<f64>().await {
                known_height.set(height);
            }
        });
    };
    let send = move |event: MessageScrollerEvent| {
        let commands = drive::<MessageScrollerMachine>(state, input, event);
        if !commands.is_empty() {
            apply(commands);
        }
    };

    use_effect(move || {
        let (c, k) = (count(), last_key());
        let first = !*mounted.peek();
        send(MessageScrollerEvent::Items { count: c, last_key: k });
        if first {
            mounted.set(true);
            apply(vec![MessageScrollerCommand::ScrollToBottom]);
        } else {
            // Re-measure so a later prepend has a current baseline.
            apply(Vec::new());
        }
    });
    use_effect(move || {
        revision();
        if *mounted.peek() {
            send(MessageScrollerEvent::Grew);
            apply(Vec::new());
        }
    });

    let s = state.read();
    let a = h::viewport_attrs(&s);
    let unread = s.unread;

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-state": a.data_state,
            div {
                id: id.cloned(),
                "data-scope": h::SCOPE,
                "data-part": "viewport",
                "data-state": a.data_state,
                role: a.role,
                "aria-live": a.aria_live,
                "aria-label": "{label}",
                tabindex: a.tabindex,
                onscroll: move |e| {
                    let data = e.data();
                    let (height, top, view) = (
                        f64::from(data.scroll_height()),
                        data.scroll_top(),
                        f64::from(data.client_height()),
                    );
                    known_height.set(height);
                    send(MessageScrollerEvent::Scrolled { from_bottom: h::from_bottom(height, top, view) });
                },
                {children}
            }
            if h::jump_visible(&s) {
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "jump",
                    onclick: move |_| send(MessageScrollerEvent::JumpToLatest),
                    "{h::jump_label(unread)}"
                }
            }
        }
    }
}
