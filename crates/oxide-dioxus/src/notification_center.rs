//! `NotificationCenter`: a bell with an unread badge that opens a popover
//! list. Items can be read, dismissed or all marked read. The popover is a
//! native `popover="auto"`, so Esc, outside presses and focus return are the
//! browser's. The machine owns only the list.

use dioxus::prelude::*;
use oxide_headless::notification_center::{
    self as h, NotificationCenterCommand, NotificationCenterEvent, NotificationCenterInput,
    NotificationCenterMachine, NotificationCenterState,
};
use oxide_statechart::Machine;

use crate::button::BUTTON_CLASS;
use crate::focus::FocusMap;
use crate::icon::{Icon, IconName};
use crate::icon_button::ICON_BUTTON_CLASS;

pub use oxide_headless::notification_center::Notification;
pub use oxide_headless::overlay::{Align, Placement, Side};

/// `FocusMap` key for the popover itself.
const CONTENT_KEY: &str = "content";

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<NotificationCenterState>,
    input: Memo<NotificationCenterInput>,
    focus: FocusMap,
    on_items_change: Option<EventHandler<Vec<Notification>>>,
    on_select: Option<EventHandler<String>>,
}

impl Ctx {
    fn send(mut self, event: NotificationCenterEvent) {
        let step = NotificationCenterMachine::transition(&self.state.peek(), event, &self.input.peek());
        if *self.state.peek() != step.state {
            self.state.set(step.state);
        }
        for command in step.commands {
            match command {
                NotificationCenterCommand::ItemsChange(items) => {
                    if let Some(handler) = self.on_items_change {
                        handler.call(items);
                    }
                }
            }
        }
    }

    /// Keep focus inside the popover when the control that had it disappears or disables.
    fn keep_focus(self) {
        self.focus.focus(CONTENT_KEY);
    }
}

/// A bell button and the popover list it opens.
#[component]
pub fn NotificationCenter(
    /// Controlled notifications. Pair with `on_items_change`.
    #[props(default)]
    items: ReadSignal<Option<Vec<Notification>>>,
    /// Initial notifications when uncontrolled.
    #[props(default)]
    default_items: Vec<Notification>,
    /// Accessible name of the bell and heading of the popover.
    #[props(default = "Notifications".to_string())]
    label: String,
    /// Where the popover sits relative to the bell. Default below, end-aligned.
    #[props(default = h::DEFAULT_PLACEMENT)]
    placement: Placement,
    /// Called with the new list whenever it changes or is requested.
    on_items_change: Option<EventHandler<Vec<Notification>>>,
    /// Called with the id of a notification when it is opened.
    on_select: Option<EventHandler<String>>,
) -> Element {
    let input = use_memo(move || NotificationCenterInput { items: items(), default_items: default_items.clone() });
    let mut state = use_signal(|| NotificationCenterMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = NotificationCenterMachine::sync(&state.peek(), &input.read());
        if *state.peek() != synced {
            state.set(synced);
        }
    });
    let id = use_hook(|| CopyValue::new(oxide_headless::next_id(h::SCOPE)));
    let ctx = use_context_provider(|| Ctx { state, input, focus: FocusMap::new(), on_items_change, on_select });

    let unread = h::unread_count(&state.read().items);
    let ids: Vec<String> = state.read().items.iter().map(|n| n.id.clone()).collect();
    let root = id.read().clone();
    let trigger = h::trigger_attrs(&root, &label, unread);
    let content = h::content_attrs(&root, placement);

    rsx! {
        button {
            r#type: "button",
            id: trigger.id,
            "aria-label": trigger.aria_label,
            "aria-haspopup": "dialog",
            "aria-controls": trigger.aria_controls.clone(),
            "popovertarget": trigger.aria_controls,
            style: trigger.style,
            class: ICON_BUTTON_CLASS,
            "data-variant": "ghost",
            "data-scope": h::SCOPE,
            "data-part": "trigger",
            "data-unread": trigger.has_unread.then_some(""),
            Icon { name: IconName::Bell, size: 18 }
            if unread > 0 {
                span { "data-scope": h::SCOPE, "data-part": "badge", "aria-hidden": "true", {h::badge_text(unread)} }
            }
        }
        div {
            id: content.id,
            role: "dialog",
            "popover": "auto",
            tabindex: "-1",
            "aria-labelledby": content.aria_labelledby,
            style: content.style,
            "data-overlay": "",
            "data-side": content.data_side,
            "data-scope": h::SCOPE,
            "data-part": "content",
            onmounted: move |e| ctx.focus.register(CONTENT_KEY.to_string(), e.data()),
            div { "data-scope": h::SCOPE, "data-part": "header",
                h2 { id: h::title_id(&root), "data-scope": h::SCOPE, "data-part": "title", "{label}" }
                button {
                    r#type: "button",
                    class: BUTTON_CLASS,
                    "data-variant": "ghost",
                    "data-size": "sm",
                    "data-scope": h::SCOPE,
                    "data-part": "mark-all",
                    disabled: unread == 0,
                    onclick: move |_| {
                        ctx.send(NotificationCenterEvent::MarkAllRead);
                        ctx.keep_focus();
                    },
                    "Mark all read"
                }
            }
            if ids.is_empty() {
                div { "data-scope": h::SCOPE, "data-part": "empty",
                    Icon { name: IconName::Inbox, size: 24 }
                    p { "data-scope": h::SCOPE, "data-part": "empty-title", "You're all caught up" }
                    p { "data-scope": h::SCOPE, "data-part": "empty-description", "New activity shows up here." }
                }
            } else {
                ul { "data-scope": h::SCOPE, "data-part": "list",
                    for item_id in ids {
                        Row { key: "{item_id}", id: item_id.clone() }
                    }
                }
            }
        }
    }
}

/// One row. Reads its notification by id, so reading it doesn't remount the row.
#[component]
fn Row(id: String) -> Element {
    let ctx = try_use_context::<Ctx>().expect("Row must be rendered inside NotificationCenter");
    let Some(item) = ctx.state.read().items.iter().find(|n| n.id == id).cloned() else {
        return rsx! {};
    };
    let (opened, dismissed) = (id.clone(), id);
    rsx! {
        li { "data-scope": h::SCOPE, "data-part": "item", "data-unread": item.unread.then_some(""),
            button {
                r#type: "button",
                "data-scope": h::SCOPE,
                "data-part": "item-button",
                onclick: move |_| {
                    ctx.send(NotificationCenterEvent::MarkRead(opened.clone()));
                    if let Some(handler) = ctx.on_select {
                        handler.call(opened.clone());
                    }
                },
                span { "data-scope": h::SCOPE, "data-part": "dot", "aria-hidden": "true" }
                span { "data-scope": h::SCOPE, "data-part": "item-title", "{item.title}" }
                if let Some(description) = &item.description {
                    span { "data-scope": h::SCOPE, "data-part": "item-description", "{description}" }
                }
                span { "data-scope": h::SCOPE, "data-part": "item-time", "{item.time}" }
                if item.unread {
                    span { "data-scope": h::SCOPE, "data-part": "sr", ", unread" }
                }
            }
            button {
                r#type: "button",
                class: ICON_BUTTON_CLASS,
                "data-variant": "ghost",
                "data-size": "sm",
                "data-scope": h::SCOPE,
                "data-part": "dismiss",
                "aria-label": "Dismiss {item.title}",
                onclick: move |_| {
                    ctx.send(NotificationCenterEvent::Dismiss(dismissed.clone()));
                    ctx.keep_focus();
                },
                Icon { name: IconName::X, size: 14 }
            }
        }
    }
}
