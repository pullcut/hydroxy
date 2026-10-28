//! `<ReactionBar>`: emoji reactions under a message. Each chip toggles your
//! reaction and shows who reacted on hover or focus; the add button opens an
//! emoji picker. The host owns the reactions and applies `on_toggle`.

use leptos::prelude::*;
use oxide_headless::reaction_bar as h;

use crate::emoji_picker::{EmojiPicker, SkinTone, focus_picker_search};
use crate::focus::focus_id;
use crate::overlay::hide_popover;

pub use oxide_headless::reaction_bar::{Reaction, toggle};

/// The add-reaction glyph: a smiley with a plus.
const ADD_ICON: &str = r#"<circle cx="11" cy="12" r="8"/><path d="M8 14.5c.8 1 1.8 1.5 3 1.5s2.2-.5 3-1.5"/><path d="M8.5 9.5h.01M13.5 9.5h.01"/><path d="M20 3v5M17.5 5.5h5"/>"#;

/// A row of reaction chips with an add button.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn ReactionBar(
    /// The reactions, in the order they were first added.
    #[prop(into)]
    reactions: Signal<Vec<Reaction>>,
    /// Called with the emoji and its name when you toggle a chip or pick a new reaction.
    #[prop(optional, into)]
    on_toggle: Option<Callback<(String, String)>>,
    /// Recently used emoji for the picker. Store what `on_recent_change` gives you.
    #[prop(optional, into)]
    recent: Signal<Vec<String>>,
    /// Called with the picker's new recent list.
    #[prop(optional, into)]
    on_recent_change: Option<Callback<Vec<String>>>,
    /// Controlled skin tone for the picker.
    #[prop(optional, into)]
    skin_tone: MaybeProp<SkinTone>,
    /// Called when a skin tone is chosen in the picker.
    #[prop(optional, into)]
    on_skin_tone_change: Option<Callback<SkinTone>>,
    /// Accessible name for the group. Default "Reactions".
    #[prop(default = "Reactions".into(), into)]
    label: String,
) -> impl IntoView {
    let ids = h::ids(&crate::id::next_id(h::SCOPE));
    let ids_s = StoredValue::new(ids.clone());
    let announce = RwSignal::new(String::new());
    let toggle = move |r: &Reaction| {
        announce.set(h::toggle_announcement(r));
        if let Some(cb) = on_toggle {
            cb.run((r.emoji.clone(), r.name.clone()));
        }
    };
    let pick = move |(emoji, name): (String, String)| {
        let ids = ids_s.get_value();
        hide_popover(&ids.picker);
        focus_id(&ids.add);
        // Picking one you already hold leaves it alone rather than taking it back.
        let existing = reactions.with_untracked(|list| list.iter().find(|r| r.emoji == emoji).cloned());
        match existing {
            Some(r) if r.reacted => {}
            Some(r) => toggle(&r),
            None => toggle(&Reaction::new(emoji, name, &[])),
        }
    };

    view! {
        <div data-scope=h::SCOPE data-part="root" role="group" aria-label=label>
            // Keyed by emoji with live contents, so a toggled chip keeps its node and focus.
            <For
                each=move || reactions.with(|list| list.iter().map(|r| r.emoji.clone()).collect::<Vec<_>>())
                key=String::clone
                children=move |emoji| {
                    let ids = ids_s.get_value();
                    let r = Memo::new({
                        let emoji = emoji.clone();
                        move |_| {
                            reactions
                                .with(|list| list.iter().find(|r| r.emoji == emoji).cloned())
                                .unwrap_or_else(|| Reaction::new(emoji.clone(), "", &[]))
                        }
                    });
                    view! {
                        <span data-scope=h::SCOPE data-part="item">
                        <button
                            type="button"
                            id=h::chip_id(&ids, &emoji)
                            aria-pressed=move || r.with(|r| r.reacted.to_string())
                            aria-label=move || r.with(h::chip_label)
                            aria-describedby=h::tip_id(&ids, &emoji)
                            style=h::chip_style(&ids, &emoji)
                            data-scope=h::SCOPE
                            data-part="chip"
                            data-state=move || if r.with(|r| r.reacted) { "on" } else { "off" }
                            on:click=move |_| toggle(&r.get_untracked())
                        >
                            <span data-scope=h::SCOPE data-part="emoji" aria-hidden="true">{emoji.clone()}</span>
                            <span data-scope=h::SCOPE data-part="count" aria-hidden="true">{move || r.with(Reaction::count)}</span>
                        </button>
                        <span
                            id=h::tip_id(&ids, &emoji)
                            role="tooltip"
                            style=h::tip_style(&ids, &emoji)
                            data-scope=h::SCOPE
                            data-part="tip"
                        >
                            {move || r.with(|r| h::who(r, h::TIP_NAMES))}
                        </span>
                        </span>
                    }
                }
            />
            <button
                type="button"
                id=ids.add.clone()
                popovertarget=ids.picker.clone()
                aria-label="Add reaction"
                title="Add reaction"
                aria-haspopup="dialog"
                style=h::add_style(&ids)
                data-scope=h::SCOPE
                data-part="add"
            >
                <svg
                    viewBox="0 0 24 24"
                    width="18"
                    height="18"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.75"
                    stroke-linecap="round"
                    aria-hidden="true"
                    inner_html=ADD_ICON
                ></svg>
            </button>
            <div
                id=ids.picker.clone()
                popover="auto"
                role="dialog"
                aria-label="Add reaction"
                data-overlay=""
                data-side="bottom"
                style=h::picker_style(&ids)
                data-scope=h::SCOPE
                data-part="picker"
                on:toggle=move |ev| {
                    let open = event_target::<web_sys::Element>(&ev).matches(":popover-open").unwrap_or(false);
                    if open {
                        focus_picker_search(&ids_s.get_value().picker);
                    }
                }
            >
                <EmojiPicker
                    on_select=pick
                    recent=recent
                    on_recent_change=move |list| {
                        if let Some(cb) = on_recent_change {
                            cb.run(list);
                        }
                    }
                    skin_tone=skin_tone
                    on_skin_tone_change=move |t| {
                        if let Some(cb) = on_skin_tone_change {
                            cb.run(t);
                        }
                    }
                />
            </div>
            <span data-scope=h::SCOPE data-part="status" role="status">{move || announce.get()}</span>
        </div>
    }
}
