use hydroxy_leptos::emoji_picker::{EmojiPicker, SkinTone};
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let recent = RwSignal::new(vec!["👍".to_string(), "🔥".to_string(), "😂".to_string()]);
    let tone = RwSignal::new(SkinTone::Default);
    let picked = RwSignal::new(None::<(String, String)>);

    view! {
        <div style="display: grid; gap: var(--space-3); justify-items: start;">
            <div style="border: 1px solid var(--border-1); border-radius: var(--radius-lg); background: var(--surface-raised); box-shadow: var(--shadow-2);">
                <EmojiPicker
                    on_select=move |p| picked.set(Some(p))
                    recent=recent
                    on_recent_change=move |list| recent.set(list)
                    skin_tone=Signal::derive(move || Some(tone.get()))
                    on_skin_tone_change=move |t| tone.set(t)
                />
            </div>
            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">
                {move || picked.get().map_or_else(|| "Nothing picked yet".to_string(), |(e, name)| format!("Picked {e} ({name})"))}
            </p>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "emoji-picker",
    name: "Emoji picker",
    group: "Chat",
    summary: "Search 1,600 emoji by name or keyword, browse by category, pick a skin tone. Recently used comes back from you.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("emoji_picker.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/emoji_picker.rs"),
    parts: &[
        &["EmojiPicker", "root", "Wrapper. --emoji-columns sets the grid width"],
        &["EmojiPicker", "search", "SearchField. Arrow Down moves into the grid, Enter picks the first match"],
        &["EmojiPicker", "tabs", "role=tablist of categories, with Recently used first when there is history"],
        &["EmojiPicker", "grid", "role=grid named by the tab or \"Search results\"; one tab stop"],
        &["EmojiPicker", "cell", "role=gridcell button named by the emoji's CLDR name"],
        &["EmojiPicker", "preview", "The active emoji and its name"],
        &["EmojiPicker", "tones", "role=radiogroup of six skin tones"],
        &["EmojiPicker", "status", "Polite live region: \"12 emoji found\""],
    ],
    keys: &[
        &["Arrow keys", "In the grid: move by cell and row. Up from the first row returns to search"],
        &["Home / End", "Start or end of the row; with Ctrl, first or last emoji"],
        &["Page Up / Page Down", "Jump four rows"],
        &["Enter / Space", "Pick the emoji"],
        &["Arrow Left / Right", "On the tabs or tones: move and select"],
    ],
    props: &[
        &["on_select", "Callback<(String, String)>", "The emoji in the chosen tone, and its name"],
        &["recent", "Vec<String>", "Recently used, newest first"],
        &["on_recent_change", "Callback<Vec<String>>", "Store the new recent list"],
        &["skin_tone", "Option<SkinTone>", "Controlled tone"],
        &["default_skin_tone", "SkinTone", "Initial tone when uncontrolled"],
        &["on_skin_tone_change", "Callback<SkinTone>", "Store the chosen tone"],
        &["columns", "usize", "Emoji per row. Default 9"],
        &["max_recent", "usize", "Recent list length. Default 27"],
    ],
    events: &[],
};
