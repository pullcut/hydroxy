//! `<NotificationCenter>`: a bell with an unread badge that opens a popover
//! list. Items can be read, dismissed or all marked read. The popover is a
//! native `popover="auto"`, so Esc, outside presses and focus return are the
//! browser's. The machine owns only the list.

use leptos::prelude::*;
use hydroxy_headless::notification_center::{
    self as h, NotificationCenterCommand, NotificationCenterEvent, NotificationCenterInput,
    NotificationCenterMachine, NotificationCenterState,
};
use hydroxy_statechart::Machine;

use crate::button::BUTTON_CLASS;
use crate::focus::focus_id;
use crate::icon::{Icon, IconName};
use crate::icon_button::ICON_BUTTON_CLASS;

pub use hydroxy_headless::notification_center::Notification;
pub use hydroxy_headless::overlay::{Align, Placement, Side};

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<NotificationCenterState>,
    input: Memo<NotificationCenterInput>,
    on_items_change: Option<Callback<Vec<Notification>>>,
}

impl Ctx {
    fn send(self, event: NotificationCenterEvent) {
        // Events can still arrive after unmount.
        let Some(step) = self
            .input
            .try_with_untracked(|input| {
                self.state.try_with_untracked(|state| NotificationCenterMachine::transition(state, event, input))
            })
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state
            && self.state.with_untracked(|s| *s != next)
        {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                NotificationCenterCommand::ItemsChange(items) => {
                    if let Some(cb) = self.on_items_change {
                        cb.run(items);
                    }
                }
            }
        }
    }

    /// Keep focus inside the popover when the control that had it disappears or disables.
    fn keep_focus(self) {
        focus_id(&h::content_id(&self.id.get_value()));
    }
}

/// A bell button and the popover list it opens.
#[component]
pub fn NotificationCenter(
    /// Controlled notifications. Pair with `on_items_change`.
    #[prop(optional, into)]
    items: MaybeProp<Vec<Notification>>,
    /// Initial notifications when uncontrolled.
    #[prop(optional)]
    default_items: Vec<Notification>,
    /// Accessible name of the bell and heading of the popover.
    #[prop(default = "Notifications".into(), into)]
    label: String,
    /// Where the popover sits relative to the bell. Default below, end-aligned.
    #[prop(default = h::DEFAULT_PLACEMENT)]
    placement: Placement,
    /// Called with the new list whenever it changes or is requested.
    #[prop(optional, into)]
    on_items_change: Option<Callback<Vec<Notification>>>,
    /// Called with the id of a notification when it is opened.
    #[prop(optional, into)]
    on_select: Option<Callback<String>>,
) -> impl IntoView {
    let input = Memo::new(move |_| NotificationCenterInput { items: items.get(), default_items: default_items.clone() });
    let state = RwSignal::new(input.with_untracked(NotificationCenterMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| NotificationCenterMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });
    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let ctx = Ctx { id, state, input, on_items_change };

    let unread = Memo::new(move |_| state.with(|s| h::unread_count(&s.items)));
    let has_unread = move || unread.get() > 0;
    let ids = Memo::new(move |_| state.with(|s| s.items.iter().map(|n| n.id.clone()).collect::<Vec<_>>()));
    let trigger = {
        let label = label.clone();
        Memo::new(move |_| h::trigger_attrs(&id.get_value(), &label, unread.get()))
    };
    let content = Memo::new(move |_| h::content_attrs(&id.get_value(), placement));

    view! {
        <button
            type="button"
            id=move || trigger.get().id
            aria-label=move || trigger.get().aria_label
            aria-haspopup="dialog"
            aria-controls=move || trigger.get().aria_controls
            popovertarget=move || trigger.get().aria_controls
            style=move || trigger.get().style
            class=ICON_BUTTON_CLASS
            data-variant="ghost"
            data-scope=h::SCOPE
            data-part="trigger"
            data-unread=move || trigger.get().has_unread.then_some("")
        >
            <Icon name=IconName::Bell size=18 />
            <Show when=has_unread>
                <span data-scope=h::SCOPE data-part="badge" aria-hidden="true">{move || h::badge_text(unread.get())}</span>
            </Show>
        </button>
        <div
            id=move || content.get().id
            role="dialog"
            popover="auto"
            tabindex="-1"
            aria-labelledby=move || content.get().aria_labelledby
            style=move || content.get().style
            data-overlay=""
            data-side=move || content.get().data_side
            data-scope=h::SCOPE
            data-part="content"
        >
            <div data-scope=h::SCOPE data-part="header">
                <h2 id=h::title_id(&id.get_value()) data-scope=h::SCOPE data-part="title">{label}</h2>
                <button
                    type="button"
                    class=BUTTON_CLASS
                    data-variant="ghost"
                    data-size="sm"
                    data-scope=h::SCOPE
                    data-part="mark-all"
                    disabled=move || unread.get() == 0
                    on:click=move |_| {
                        ctx.send(NotificationCenterEvent::MarkAllRead);
                        ctx.keep_focus();
                    }
                >
                    "Mark all read"
                </button>
            </div>
            <Show
                when=move || !ids.with(Vec::is_empty)
                fallback=|| {
                    view! {
                        <div data-scope=h::SCOPE data-part="empty">
                            <Icon name=IconName::Inbox size=24 />
                            <p data-scope=h::SCOPE data-part="empty-title">"You're all caught up"</p>
                            <p data-scope=h::SCOPE data-part="empty-description">"New activity shows up here."</p>
                        </div>
                    }
                }
            >
                <ul data-scope=h::SCOPE data-part="list">
                    <For each=move || ids.get() key=Clone::clone let(item_id)>
                        <Row id=item_id ctx on_select />
                    </For>
                </ul>
            </Show>
        </div>
    }
}

/// One row. Reads its notification by id, so reading it doesn't remount the row.
#[component]
fn Row(id: String, ctx: Ctx, on_select: Option<Callback<String>>) -> impl IntoView {
    let item = {
        let id = id.clone();
        Memo::new(move |_| ctx.state.with(|s| s.items.iter().find(|n| n.id == id).cloned()))
    };
    let (open_id, dismiss_id) = (id.clone(), id);
    view! {
        <li
            data-scope=h::SCOPE
            data-part="item"
            data-unread=move || item.get().is_some_and(|n| n.unread).then_some("")
        >
            <button
                type="button"
                data-scope=h::SCOPE
                data-part="item-button"
                on:click=move |_| {
                    ctx.send(NotificationCenterEvent::MarkRead(open_id.clone()));
                    if let Some(cb) = on_select {
                        cb.run(open_id.clone());
                    }
                }
            >
                <span data-scope=h::SCOPE data-part="dot" aria-hidden="true"></span>
                <span data-scope=h::SCOPE data-part="item-title">{move || item.get().map(|n| n.title)}</span>
                {move || {
                    item.get()
                        .and_then(|n| n.description)
                        .map(|d| view! { <span data-scope=h::SCOPE data-part="item-description">{d}</span> })
                }}
                <span data-scope=h::SCOPE data-part="item-time">{move || item.get().map(|n| n.time)}</span>
                <Show when=move || item.get().is_some_and(|n| n.unread)>
                    <span data-scope=h::SCOPE data-part="sr">", unread"</span>
                </Show>
            </button>
            <button
                type="button"
                class=ICON_BUTTON_CLASS
                data-variant="ghost"
                data-size="sm"
                data-scope=h::SCOPE
                data-part="dismiss"
                aria-label=move || format!("Dismiss {}", item.get().map(|n| n.title).unwrap_or_default())
                on:click=move |_| {
                    ctx.send(NotificationCenterEvent::Dismiss(dismiss_id.clone()));
                    ctx.keep_focus();
                }
            >
                <Icon name=IconName::X size=14 />
            </button>
        </li>
    }
}
