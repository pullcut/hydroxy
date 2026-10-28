use dioxus::prelude::*;
use oxide_dioxus::emoji_picker::{EmojiPicker, SkinTone};

#[component]
pub fn EmojiPickerDemo() -> Element {
    let mut recent = use_signal(|| vec!["👍".to_string(), "🔥".to_string(), "😂".to_string()]);
    let mut tone = use_signal(|| SkinTone::Default);
    let mut picked = use_signal(|| None::<(String, String)>);
    let text = picked().map_or_else(|| "Nothing picked yet".to_string(), |(e, name)| format!("Picked {e} ({name})"));

    rsx! {
        div { style: "display: grid; gap: var(--space-3); justify-items: start;",
            div { style: "border: 1px solid var(--border-1); border-radius: var(--radius-lg); background: var(--surface-raised); box-shadow: var(--shadow-2);",
                EmojiPicker {
                    on_select: move |p| picked.set(Some(p)),
                    recent: recent(),
                    on_recent_change: move |list| recent.set(list),
                    skin_tone: Some(tone()),
                    on_skin_tone_change: move |t| tone.set(t),
                }
            }
            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);", "{text}" }
        }
    }
}
