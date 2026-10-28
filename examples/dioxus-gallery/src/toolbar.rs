use dioxus::prelude::*;
use oxide_dioxus::icon::IconName;
use oxide_dioxus::toolbar::{Toolbar, ToolbarItem};

#[component]
pub fn ToolbarDemo() -> Element {
    let mut last = use_signal(|| String::from("nothing yet"));
    // Higher priority stays in the bar longer; undo and redo go last.
    let items = vec![
        ToolbarItem::new("undo", "Undo").icon(IconName::ArrowUp).shortcut("⌘Z").priority(3),
        ToolbarItem::new("redo", "Redo").icon(IconName::ArrowRight).shortcut("⇧⌘Z").priority(3),
        ToolbarItem::new("bold", "Bold").toggle().group(1).shortcut("⌘B").priority(2),
        ToolbarItem::new("italic", "Italic").toggle().group(1).shortcut("⌘I").priority(2),
        ToolbarItem::new("strike", "Strikethrough").toggle().group(1),
        ToolbarItem::new("link", "Link").group(2).shortcut("⌘K"),
        ToolbarItem::new("quote", "Quote").group(2),
        ToolbarItem::new("code", "Code block").group(2),
        ToolbarItem::new("comment", "Comment").icon(IconName::Plus).group(3).priority(1),
        ToolbarItem::new("delete", "Delete block").icon(IconName::Trash).group(3).disabled(),
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-3);",
            // Drag the corner to narrow it: items move into "More".
            div { style: "resize: horizontal; overflow: hidden; min-width: 160px; max-width: 100%; width: 620px;",
                Toolbar {
                    label: "Text formatting",
                    items,
                    default_pressed: vec!["bold".to_string()],
                    on_action: move |v| last.set(v),
                    on_toggle: move |(v, on): (String, bool)| last.set(format!("{v} {}", if on { "on" } else { "off" })),
                }
            }
            p { style: "margin: 0; font: var(--text-xs); color: var(--text-3);", "Last: {last}" }
        }
    }
}
