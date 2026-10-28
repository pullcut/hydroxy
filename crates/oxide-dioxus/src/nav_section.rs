//! `NavSection`: a collapsible sidebar group, built on the Collapsible machine.

use dioxus::prelude::*;
use oxide_headless::collapsible::{self as h, CollapsibleEvent, CollapsibleInput, CollapsibleMachine};
use oxide_statechart::Machine;

use crate::icon::{Icon, IconName};

/// A sidebar group with a mono label. Shows its count while folded.
#[component]
pub fn NavSection(
    /// The group's name.
    label: String,
    /// How many items it holds. Shown while folded; 0 shows `empty`.
    #[props(default)]
    count: ReadSignal<Option<u32>>,
    /// Controlled open state. Pair with `on_open_change`.
    #[props(default)]
    open: ReadSignal<Option<bool>>,
    /// Initial open state when uncontrolled.
    #[props(default)]
    default_open: bool,
    /// Shown inside an open group whose `count` is 0.
    #[props(default = "Nothing here yet".to_string())]
    empty: String,
    /// Called with the new open state.
    on_open_change: Option<EventHandler<bool>>,
    /// The group's rows, usually `NavItem`s.
    children: Element,
) -> Element {
    let input = use_memo(move || CollapsibleInput { open: open(), default_open, disabled: false });
    let mut state = use_signal(|| CollapsibleMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = CollapsibleMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let id = use_hook(|| oxide_headless::next_id("nav-section"));
    let s = state.read().clone();
    let trigger = h::trigger_attrs(&id, &s, &input.read());
    let content = h::content_attrs(&id, &s);
    let folded_count = if s.open { None } else { count().filter(|n| *n > 0) };
    let is_empty = count() == Some(0);

    rsx! {
        div { "data-scope": "nav-section", "data-state": h::data_state(s.open),
            button {
                r#type: "button",
                "data-part": "head",
                id: "{trigger.id}",
                "aria-expanded": trigger.aria_expanded.to_string(),
                "aria-controls": "{trigger.aria_controls}",
                onclick: move |_| {
                    let step = CollapsibleMachine::transition(&state.peek(), CollapsibleEvent::Toggle, &input.peek());
                    let now_open = step.state.as_ref().map_or_else(|| state.peek().open, |s| s.open);
                    if let Some(next) = step.state {
                        state.set(next);
                    }
                    if let Some(handler) = on_open_change {
                        handler.call(now_open);
                    }
                },
                Icon { name: IconName::ChevronRight, size: 12 }
                span { "{label}" }
                if let Some(n) = folded_count {
                    span { "data-part": "count", "{n}" }
                }
            }
            div {
                "data-part": "body",
                id: "{content.id}",
                role: "group",
                "aria-labelledby": "{content.aria_labelledby}",
                hidden: content.hidden,
                {children}
                if is_empty {
                    span { "data-part": "empty", "{empty}" }
                }
            }
        }
    }
}
