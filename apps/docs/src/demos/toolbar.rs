use leptos::prelude::*;
use oxide_leptos::icon::IconName;
use oxide_leptos::toolbar::{Toolbar, ToolbarItem};

pub fn demo() -> AnyView {
    let (last, set_last) = signal(String::from("nothing yet"));
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

    view! {
        <div style="display: grid; gap: var(--space-3);">
            // Drag the corner to narrow it: items move into "More".
            <div style="resize: horizontal; overflow: hidden; min-width: 160px; max-width: 100%; width: 620px;">
                <Toolbar
                    label="Text formatting"
                    items=items
                    default_pressed=vec!["bold".to_string()]
                    on_action=move |v| set_last.set(v)
                    on_toggle=move |(v, on): (String, bool)| set_last.set(format!("{v} {}", if on { "on" } else { "off" }))
                />
            </div>
            <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">"Last: " {last}</p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "toolbar",
    name: "Toolbar",
    group: "Actions",
    summary: "A row of actions and toggles with one tab stop. Arrow keys move between items; whatever doesn't fit folds into a \"More\" menu, lowest priority first.",
    demo,
    leptos: include_str!("toolbar.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/toolbar.rs"),
    parts: &[
        &["Toolbar", "root", "role=\"toolbar\" with aria-label and aria-orientation. Fills its container"],
        &["Toolbar", "group", "role=\"group\": a run of items with the same group number"],
        &["Toolbar", "separator", "role=\"separator\" between groups that both show something"],
        &["Toolbar", "item", "<button>. Toggles carry aria-pressed; icon-only items get aria-label"],
        &["Toolbar", "more", "The \"More\" menu button, shown when items overflow"],
        &["Menu", "content / item", "The overflow menu (menu styles). Toggles are menuitemcheckbox"],
    ],
    keys: &[
        &["Tab", "Into and out of the toolbar (one stop)"],
        &["← / →", "Previous / next item, wrapping. ↑ / ↓ in a vertical toolbar"],
        &["Home / End", "First / last item"],
        &["Enter / Space", "Activate or toggle the focused item"],
        &["↓ on More", "Open the overflow menu"],
    ],
    props: &[
        &["label", "String", "Required. The toolbar's accessible name"],
        &["items", "Vec<ToolbarItem>", "Required. In order; see .toggle, .icon, .group, .priority, .shortcut, .disabled"],
        &["orientation", "Orientation", "Horizontal (default; overflows) or Vertical"],
        &["pressed", "Option<BTreeSet<String>>", "Controlled pressed toggles. Pair with on_toggle"],
        &["default_pressed", "Vec<String>", "Pressed toggles at first, when uncontrolled"],
        &["on_action", "Callback<String>", "An action was activated"],
        &["on_toggle", "Callback<(String, bool)>", "A toggle changed"],
    ],
};
