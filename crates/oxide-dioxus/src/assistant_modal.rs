//! `AssistantModal`: a floating chat panel opened from a round button at the bottom-right
//! of its container. Put a thread (and composer) in the panel.

use std::rc::Rc;

use dioxus::prelude::*;
use oxide_headless::assistant_thread::SCOPE_MODAL;
use oxide_headless::dialog::{self as d, CloseReason, DialogCommand, DialogEvent, DialogInput, DialogMachine};
use oxide_statechart::Machine;

use crate::icon::{Icon, IconName};

/// A floating assistant panel. The container it sits in needs `position: relative`.
/// The panel stays mounted while closed, so the conversation inside keeps its state.
/// Escape closes it and returns focus to the trigger.
#[component]
pub fn AssistantModal(
    /// Controlled open state. Pair with `on_open_change`.
    #[props(default)]
    open: ReadSignal<Option<bool>>,
    /// Initial open state when uncontrolled.
    #[props(default)]
    default_open: bool,
    /// Called with the new open state whenever it changes or is requested.
    on_open_change: Option<EventHandler<bool>>,
    /// Heading of the panel, also its accessible name.
    #[props(default = "Assistant".to_string())]
    title: String,
    /// Accessible name of the round button while closed.
    #[props(default = "Open assistant".to_string())]
    trigger_label: String,
    /// The panel's contents.
    children: Element,
) -> Element {
    let input = use_memo(move || DialogInput { open: open(), default_open, alert: false });
    let mut state = use_signal(|| DialogMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = DialogMachine::sync(&state.peek(), &input.read());
        state.set(synced);
    });
    let root_id = use_hook(|| oxide_headless::next_id(SCOPE_MODAL));
    let mut panel = use_signal(|| None::<Rc<MountedData>>);
    let mut trigger = use_signal(|| None::<Rc<MountedData>>);

    let mut send = move |event: DialogEvent| {
        let step = DialogMachine::transition(&state.peek(), event, &input.peek());
        state.set(step.state);
        for DialogCommand::OpenChange(open) in step.commands {
            if let Some(handler) = on_open_change {
                handler.call(open);
            }
            let target = if open { panel.peek().clone() } else { trigger.peek().clone() };
            if let Some(el) = target {
                spawn(async move {
                    let _ = el.set_focus(true).await;
                });
            }
        }
    };

    let is_open = state.read().open;
    let t = d::trigger_attrs(&root_id, &state.read());
    rsx! {
        div { "data-scope": SCOPE_MODAL, "data-part": "root", "data-state": d::data_state(is_open),
            div {
                id: d::content_id(&root_id),
                role: "dialog",
                "aria-label": "{title}",
                tabindex: "-1",
                hidden: !is_open,
                "data-scope": SCOPE_MODAL,
                "data-part": "panel",
                "data-state": d::data_state(is_open),
                onmounted: move |e| panel.set(Some(e.data())),
                onkeydown: move |e| {
                    let key = e.key().to_string();
                    if let Some(event) = d::keydown(&key) {
                        e.prevent_default();
                        send(event);
                    }
                },
                div { "data-scope": SCOPE_MODAL, "data-part": "header",
                    span { "data-scope": SCOPE_MODAL, "data-part": "title", "{title}" }
                    button {
                        r#type: "button",
                        "data-scope": SCOPE_MODAL,
                        "data-part": "close",
                        "aria-label": "Close assistant",
                        onclick: move |_| send(DialogEvent::Close(CloseReason::CloseTrigger)),
                        Icon { name: IconName::X, size: 16 }
                    }
                }
                div { "data-scope": SCOPE_MODAL, "data-part": "body", {children} }
            }
            button {
                r#type: "button",
                "data-scope": SCOPE_MODAL,
                "data-part": "trigger",
                "aria-label": if is_open { "Close assistant".to_string() } else { trigger_label },
                "aria-haspopup": t.aria_haspopup,
                "aria-expanded": t.aria_expanded.to_string(),
                "aria-controls": t.aria_controls,
                "data-state": t.data_state,
                onmounted: move |e| trigger.set(Some(e.data())),
                onclick: move |_| {
                    send(if is_open { DialogEvent::Close(CloseReason::CloseTrigger) } else { DialogEvent::Open });
                },
                Icon { name: if is_open { IconName::X } else { IconName::Mark }, size: 22 }
            }
        }
    }
}
