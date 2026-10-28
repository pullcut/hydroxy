use leptos::prelude::*;
use oxide_leptos::presence_bar::{Collaborator, PresenceBar, PresenceStatus};

pub fn demo() -> AnyView {
    let people = RwSignal::new(vec![
        Collaborator::new("u-maya", "Maya Okafor").me(),
        Collaborator::new("u-jonas", "Jonas Berg"),
        Collaborator::new("u-priya", "Priya Raman").status(PresenceStatus::Idle),
        Collaborator::new("u-leo", "Leo Martins"),
        Collaborator::new("u-sana", "Sana Haddad").status(PresenceStatus::Away),
        Collaborator::new("u-tom", "Tomás Ruiz"),
        Collaborator::new("u-ines", "Inès Laurent").status(PresenceStatus::Idle),
        Collaborator::new("u-kofi", "Kofi Mensah"),
    ]);
    // Someone joins or leaves, as a websocket message would report.
    let toggle_guest = move |_| {
        people.update(|p| {
            if let Some(i) = p.iter().position(|c| c.id == "u-ana") {
                p.remove(i);
            } else {
                p.push(Collaborator::new("u-ana", "Ana Lima"));
            }
        });
    };

    view! {
        <div style="display: grid; gap: var(--space-5);">
            <div style="display: flex; align-items: center; justify-content: space-between; gap: var(--space-4); padding: var(--space-3) var(--space-4); border: 1px solid var(--border-1); border-radius: var(--radius-lg); background: var(--surface-card);">
                <span style="font: var(--text-ui); color: var(--text-1);">"Q3 planning doc"</span>
                <PresenceBar people=people max=5 />
            </div>
            <button type="button" class="ox-button" data-variant="secondary" data-size="sm" style="justify-self: start;" on:click=toggle_guest>
                {move || if people.with(|p| p.iter().any(|c| c.id == "u-ana")) { "Ana leaves" } else { "Ana joins" }}
            </button>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "presence-bar",
    name: "Presence bar",
    group: "Activity",
    summary: "Who's here right now: an overlapping avatar stack, active people first, with the rest behind a +N list.",
    demo,
    leptos: include_str!("presence_bar.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/presence_bar.rs"),
    parts: &[
        &["PresenceBar", "root", "role=group named by a summary: \"8 people here: Maya, Jonas, Leo and 5 others\""],
        &["PresenceBar", "item", "One avatar. Focusable, named \"Name, status\". data-status active|idle|away"],
        &["PresenceBar", "dot", "Status dot: green active, amber idle, hollow away"],
        &["PresenceBar", "tip", "Name and status, anchored below on hover or focus"],
        &["PresenceBar", "overflow", "\"+N\" button that opens the list (native popover)"],
        &["PresenceBar", "list", "Popover listing the people who didn't fit"],
    ],
    keys: &[
        &["Tab", "Move through the avatars, then the +N button"],
        &["Enter / Space", "On +N: open or close the list"],
        &["Escape", "Close the list"],
    ],
    props: &[
        &["people", "Vec<Collaborator>", "Required. Id, name, status, optional jewel, is_self"],
        &["max", "usize", "Avatar slots including +N. Default 5"],
        &["size", "u16", "Avatar size in px. Default 28"],
    ],
};
