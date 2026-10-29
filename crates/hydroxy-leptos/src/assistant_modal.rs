//! `<AssistantModal>`: a floating chat panel opened from a round button at the bottom-right
//! of its container. Put a thread (and composer) in the panel.

use hydroxy_headless::assistant_thread::SCOPE_MODAL;
use hydroxy_headless::dialog::{self as d, CloseReason, DialogCommand, DialogEvent, DialogInput, DialogMachine};
use hydroxy_statechart::Machine;
use leptos::html::Div;
use leptos::prelude::*;

use crate::drive::drive;
use crate::icon::{Icon, IconName};

/// A floating assistant panel. The container it sits in needs `position: relative`.
/// The panel stays mounted while closed, so the conversation inside keeps its state.
/// Escape closes it and returns focus to the trigger.
#[component]
pub fn AssistantModal(
    /// Controlled open state. Pair with `on_open_change`.
    #[prop(optional, into)]
    open: MaybeProp<bool>,
    /// Initial open state when uncontrolled.
    #[prop(optional)]
    default_open: bool,
    /// Called with the new open state whenever it changes or is requested.
    #[prop(optional, into)]
    on_open_change: Option<Callback<bool>>,
    /// Heading of the panel, also its accessible name.
    #[prop(default = "Assistant".into(), into)]
    title: String,
    /// Accessible name of the round button while closed.
    #[prop(default = "Open assistant".into(), into)]
    trigger_label: String,
    /// The panel's contents.
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| DialogInput { open: open.get(), default_open, alert: false });
    let state = RwSignal::new(input.with_untracked(DialogMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| DialogMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let root_id = crate::id::next_id(SCOPE_MODAL);
    let panel = NodeRef::<Div>::new();
    let trigger = NodeRef::<leptos::html::Button>::new();

    let send = move |event: DialogEvent| {
        for DialogCommand::OpenChange(open) in drive::<DialogMachine, _>(state, input, event) {
            if let Some(cb) = on_open_change {
                cb.run(open);
            }
            let target =
                if open { panel.get_untracked().map(Into::into) } else { trigger.get_untracked().map(Into::into) };
            let target: Option<web_sys::HtmlElement> = target;
            if let Some(el) = target {
                // The panel is unhidden on the next render; focus after it.
                set_timeout(
                    move || {
                        let _ = el.focus();
                    },
                    std::time::Duration::ZERO,
                );
            }
        }
    };

    let open_now = move || state.with(|s| s.open.value);
    let panel_title = title.clone();
    let content_id = d::content_id(&root_id);
    let trigger_attrs = Memo::new(move |_| state.with(|s| d::trigger_attrs(&root_id, s)));
    view! {
        <div data-scope=SCOPE_MODAL data-part="root" data-state=move || d::data_state(open_now())>
            <div
                node_ref=panel
                id=content_id
                role="dialog"
                aria-label=panel_title
                tabindex="-1"
                hidden=move || !open_now()
                data-scope=SCOPE_MODAL
                data-part="panel"
                data-state=move || d::data_state(open_now())
                on:keydown=move |ev| {
                    if let Some(e) = d::keydown(&ev.key()) {
                        ev.prevent_default();
                        send(e);
                    }
                }
            >
                <div data-scope=SCOPE_MODAL data-part="header">
                    <span data-scope=SCOPE_MODAL data-part="title">{title}</span>
                    <button
                        type="button"
                        data-scope=SCOPE_MODAL
                        data-part="close"
                        aria-label="Close assistant"
                        on:click=move |_| send(DialogEvent::Close(CloseReason::CloseTrigger))
                    >
                        <Icon name=IconName::X size=16 />
                    </button>
                </div>
                <div data-scope=SCOPE_MODAL data-part="body">{children()}</div>
            </div>
            <button
                node_ref=trigger
                type="button"
                data-scope=SCOPE_MODAL
                data-part="trigger"
                aria-label=move || if open_now() { "Close assistant".to_string() } else { trigger_label.clone() }
                aria-haspopup=move || trigger_attrs.with(|x| x.aria_haspopup)
                aria-expanded=move || trigger_attrs.with(|x| x.aria_expanded).to_string()
                aria-controls=move || trigger_attrs.with(|x| x.aria_controls.clone())
                data-state=move || trigger_attrs.with(|x| x.data_state)
                on:click=move |_| send(if open_now() { DialogEvent::Close(CloseReason::CloseTrigger) } else { DialogEvent::Open })
            >
                {move || view! { <Icon name=if open_now() { IconName::X } else { IconName::Mark } size=22 /> }}
            </button>
        </div>
    }
}
