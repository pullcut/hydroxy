use hydroxy_leptos::reaction_bar::{Reaction, ReactionBar, toggle};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let reactions = RwSignal::new(vec![
        Reaction::new("👍", "thumbs up", &["Jonas Berg", "Priya Raman", "Leo Martins"]).mine(),
        Reaction::new("🎉", "party popper", &["Sana Haddad"]),
        Reaction::new("👀", "eyes", &["Kofi Mensah", "Inès Laurent"]),
    ]);
    // A real app would store these per user, say in localStorage or the profile.
    let recent = RwSignal::new(vec!["🚀".to_string(), "🙏".to_string(), "✅".to_string()]);

    view! {
        <div style="display: grid; gap: var(--space-3); max-width: 460px; padding: var(--space-4); border-radius: var(--radius-lg); background: var(--surface-sunken);">
            <p style="margin: 0; font: var(--text-body); color: var(--text-1);">
                "Release 2.4 is out. Thanks everyone who tested the beta over the weekend."
            </p>
            <ReactionBar
                reactions=reactions
                on_toggle=move |(emoji, name): (String, String)| reactions.update(|r| toggle(r, &emoji, &name))
                recent=recent
                on_recent_change=move |list| recent.set(list)
            />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "reaction-bar",
    name: "Reaction bar",
    group: "Activity",
    summary: "Emoji reactions with counts. Press one to add or take back yours; hover or focus to see who reacted.",
    demo,
    leptos: include_str!("reaction_bar.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/reaction_bar.rs"),
    parts: &[
        &["ReactionBar", "root", "role=group, named \"Reactions\""],
        &["ReactionBar", "chip", "Toggle button with aria-pressed; named \"thumbs up, 3 reactions\""],
        &["ReactionBar", "tip", "role=tooltip describing the chip: \"You, Jonas and 2 others reacted with thumbs up\""],
        &["ReactionBar", "add", "Opens the emoji picker in a native popover"],
        &["ReactionBar", "picker", "Popover holding an EmojiPicker; focus starts in its search"],
        &["ReactionBar", "status", "Polite live region: \"Added party popper\""],
    ],
    keys: &[
        &["Enter / Space", "On a chip: add or remove your reaction"],
        &["Tab", "Move between chips and the add button"],
        &["Escape", "Close the picker and return to the add button"],
    ],
    props: &[
        &["reactions", "Vec<Reaction>", "Required. Emoji, name, other people, whether you reacted"],
        &[
            "on_toggle",
            "Callback<(String, String)>",
            "Emoji and name to toggle. reaction_bar::toggle applies it to a Vec",
        ],
        &["recent", "Vec<String>", "Recently used emoji for the picker"],
        &["on_recent_change", "Callback<Vec<String>>", "Store the picker's new recent list"],
        &["skin_tone", "Option<SkinTone>", "Controlled skin tone for the picker"],
        &["on_skin_tone_change", "Callback<SkinTone>", "Store the chosen tone"],
        &["label", "String", "Group name. Default \"Reactions\""],
    ],
    events: &[],
};
