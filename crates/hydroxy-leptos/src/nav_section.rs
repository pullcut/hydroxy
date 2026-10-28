//! `<NavSection>`: a collapsible sidebar group, built on the Collapsible machine.

use crate::drive::drive;
use hydroxy_headless::collapsible::{self as h, CollapsibleEvent, CollapsibleInput, CollapsibleMachine};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::icon::{Icon, IconName};

/// A sidebar group with a mono label. Shows its count while folded.
#[component]
pub fn NavSection(
    /// The group's name.
    #[prop(into)]
    label: String,
    /// How many items it holds. Shown while folded; 0 shows `empty`.
    #[prop(optional, into)]
    count: MaybeProp<u32>,
    /// Controlled open state. Pair with `on_open_change`.
    #[prop(optional, into)]
    open: MaybeProp<bool>,
    /// Initial open state when uncontrolled.
    #[prop(optional)]
    default_open: bool,
    /// Shown inside an open group whose `count` is 0.
    #[prop(default = "Nothing here yet".into(), into)]
    empty: String,
    /// Called with the new open state.
    #[prop(optional, into)]
    on_open_change: Option<Callback<bool>>,
    /// The group's rows, usually `NavItem`s.
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| CollapsibleInput { open: open.get(), default_open, disabled: false });
    let state = RwSignal::new(input.with_untracked(CollapsibleMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| CollapsibleMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let id = crate::id::next_id("nav-section");
    let trigger = Memo::new({
        let id = id.clone();
        move |_| input.with(|i| state.with(|s| h::trigger_attrs(&id, s, i)))
    });
    let content = Memo::new(move |_| state.with(|s| h::content_attrs(&id, s)));
    let toggle = move |_| {
        drive::<CollapsibleMachine, _>(state, input, CollapsibleEvent::Toggle);
        if let Some(cb) = on_open_change {
            cb.run(state.get_untracked().open.value);
        }
    };
    let is_empty = move || count.get() == Some(0);

    view! {
        <div data-scope="nav-section" data-state=move || h::data_state(state.get().open.value)>
            <button
                type="button"
                data-part="head"
                id=move || trigger.get().id
                aria-expanded=move || trigger.get().aria_expanded.to_string()
                aria-controls=move || trigger.get().aria_controls
                on:click=toggle
            >
                <Icon name=IconName::ChevronRight size=12 />
                <span>{label}</span>
                {move || (!state.get().open.value).then(|| count.get().filter(|n| *n > 0).map(|n| view! { <span data-part="count">{n}</span> })).flatten()}
            </button>
            <div
                data-part="body"
                id=move || content.get().id
                role="group"
                aria-labelledby=move || content.get().aria_labelledby
                hidden=move || content.get().hidden
            >
                {children()}
                {move || is_empty().then(|| view! { <span data-part="empty">{empty.clone()}</span> })}
            </div>
        </div>
    }
}
