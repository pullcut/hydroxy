//! `<PresenceBar>`: an overlapping stack of avatars for who's here. Active people
//! come first; idle ones dim and away ones fade further. Each avatar shows the
//! name on hover or focus. People past `max` sit behind a "+N" button whose
//! popover lists them.

use hydroxy_headless::presence_bar as h;
use leptos::prelude::*;

use crate::avatar::Avatar;

pub use hydroxy_headless::presence_bar::{Collaborator, PresenceStatus};

/// One avatar in the stack, with its status dot and name tip.
fn item(root: &str, person: &Collaborator, size: u16) -> impl IntoView + use<> {
    view! {
        <li
            data-scope=h::SCOPE
            data-part="item"
            data-status=person.status.as_str()
            data-self=person.is_self.then_some("")
            tabindex="0"
            aria-label=person.label()
            style=h::item_style(root, &person.id)
        >
            <span data-scope=h::SCOPE data-part="avatar" aria-hidden="true">
                <Avatar initials=h::initials(&person.name) jewel=person.tint() size=size title=person.name.clone() />
            </span>
            <span data-scope=h::SCOPE data-part="dot" aria-hidden="true"></span>
            <span data-scope=h::SCOPE data-part="tip" aria-hidden="true" style=h::tip_style(root, &person.id)>
                <span data-scope=h::SCOPE data-part="tip-name">{h::display_name(person)}</span>
                <span data-scope=h::SCOPE data-part="tip-status">{person.status.as_str()}</span>
            </span>
        </li>
    }
}

/// Who's here, as an avatar stack.
#[component]
pub fn PresenceBar(
    /// Everyone in the room, in the host's order (say, join time).
    #[prop(into)]
    people: Signal<Vec<Collaborator>>,
    /// Most avatars before the "+N" button takes the last slot. Default 5.
    #[prop(default = 5)]
    max: usize,
    /// Avatar size in px. Default 28.
    #[prop(default = 28)]
    size: u16,
) -> impl IntoView {
    let root = crate::id::next_id(h::SCOPE);
    let ids = h::ids(&root);
    let layout = Memo::new(move |_| people.with(|p| h::layout(p, max)));
    let root_key = StoredValue::new(root);
    let (list_id, list_style) = (ids.list.clone(), h::list_style(&ids));

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            role="group"
            aria-label=move || layout.with(|l| l.summary.clone())
            style=format!("--presence-size: {size}px;")
        >
            <ul data-scope=h::SCOPE data-part="stack">
                <For
                    each=move || layout.with(|l| l.visible.clone())
                    key=|p| (p.id.clone(), p.name.clone(), p.status, p.is_self, p.jewel.map(hydroxy_headless::jewel::Jewel::as_str))
                    children=move |p| item(&root_key.get_value(), &p, size)
                />
                {move || {
                    let hidden = layout.with(|l| l.overflow.len());
                    (hidden > 0).then(|| view! {
                        <li data-scope=h::SCOPE data-part="overflow-item">
                            <button
                                type="button"
                                data-scope=h::SCOPE
                                data-part="overflow"
                                id=ids.overflow.clone()
                                popovertarget=ids.list.clone()
                                aria-label=h::overflow_label(hidden)
                                style=h::overflow_style(&ids)
                            >
                                {format!("+{hidden}")}
                            </button>
                        </li>
                    })
                }}
            </ul>
            <div
                id=list_id
                popover="auto"
                data-overlay=""
                data-side="bottom"
                data-scope=h::SCOPE
                data-part="list"
                style=list_style
            >
                <ul data-scope=h::SCOPE data-part="list-items" aria-label="Also here">
                    <For
                        each=move || layout.with(|l| l.overflow.clone())
                        key=|p| (p.id.clone(), p.name.clone(), p.status, p.is_self, p.jewel.map(hydroxy_headless::jewel::Jewel::as_str))
                        let:p
                    >
                        <li data-scope=h::SCOPE data-part="list-item" data-status=p.status.as_str()>
                            <span data-scope=h::SCOPE data-part="avatar" aria-hidden="true">
                                <Avatar initials=h::initials(&p.name) jewel=p.tint() size=24 />
                                <span data-scope=h::SCOPE data-part="dot"></span>
                            </span>
                            <span data-scope=h::SCOPE data-part="list-name">
                                {h::display_name(&p)}
                            </span>
                            <span data-scope=h::SCOPE data-part="list-status">{p.status.as_str()}</span>
                        </li>
                    </For>
                </ul>
            </div>
        </div>
    }
}
