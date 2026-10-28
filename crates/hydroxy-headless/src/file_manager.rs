//! File manager connector. Anatomy: root, sidebar (a [`TreeView`](crate::tree_view)
//! of folders), main, toolbar, breadcrumbs, crumb, view-toggle, count, items,
//! item, icon, name, rename, rename-error, modified, size and live. Every part
//! carries `data-scope="file-manager"` and its `data-part`. The root carries
//! `data-view` (`list` or `grid`).
//!
//! Items are `role="option"` rows in a multi-select `role="listbox"` with a
//! roving tab stop. A right click opens a [`ContextMenu`](crate::context_menu)
//! for open, rename and delete.

use crate::menu::MenuEntry;
use crate::tree_view::TreeNode;

pub use hydroxy_machines::file_manager::*;

pub use crate::attachment::{Kind, badge_text, kind_of};
pub use crate::file_dropzone::format_size;

/// The `data-scope` value on every file manager part.
pub const SCOPE: &str = "file-manager";

/// The inner SVG markup of the folder icon, for a 24 by 24 view box.
pub const FOLDER_ICON: &str = r#"<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>"#;

/// Smallest tile width in grid view, in px. The stylesheet's `minmax` must match.
pub const GRID_MIN: f64 = 120.0;
/// Gap between tiles in grid view, in px. The stylesheet's `gap` must match.
pub const GRID_GAP: f64 = 8.0;

/// How many tiles fit in a row of `width` px, which is what Up and Down skip over.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a small, non-negative count")]
pub fn grid_columns(width: f64) -> usize {
    let n = ((width + GRID_GAP) / (GRID_MIN + GRID_GAP)).floor();
    if n.is_finite() && n >= 1.0 { n as usize } else { 1 }
}

/// Attributes for an item row or tile (`<li role="option">`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemAttrs {
    /// `id`, so the adapter can focus it.
    pub id: String,
    /// `aria-selected`.
    pub selected: bool,
    /// `tabindex="0"` (the roving stop) rather than `-1`.
    pub tab_stop: bool,
    /// Its name is being edited.
    pub renaming: bool,
    /// `data-kind`: `folder`, or the file kind such as `pdf`.
    pub kind: &'static str,
}

/// The DOM id of item `item`.
#[must_use]
pub fn item_id(root: &str, item: &str) -> String {
    format!("{root}-item-{item}")
}

/// The DOM id of the rename field.
#[must_use]
pub fn rename_id(root: &str) -> String {
    format!("{root}-rename")
}

/// The DOM id of the rename error, which the field points at.
#[must_use]
pub fn rename_error_id(root: &str) -> String {
    format!("{root}-rename-error")
}

/// The DOM id of the live region.
#[must_use]
pub fn live_id(root: &str) -> String {
    format!("{root}-live")
}

/// Attributes for `item` among `shown`.
#[must_use]
pub fn item_attrs(root: &str, state: &FileManagerState, shown: &[FileItem], item: &FileItem) -> ItemAttrs {
    let stop = state.tab_stop(shown).is_some_and(|s| s.id == item.id);
    let kind = if item.is_folder() { "folder" } else { kind_of(&item.name, &item.mime).as_str() };
    ItemAttrs {
        id: item_id(root, &item.id),
        selected: state.selection.contains(&item.id),
        tab_stop: stop,
        renaming: state.renaming.as_ref().is_some_and(|r| r.id == item.id),
        kind,
    }
}

/// The text of the size column: `"2.4 MB"`, or a dash for folders.
#[must_use]
pub fn size_text(item: &FileItem) -> String {
    item.size.map_or_else(|| "—".to_string(), format_size)
}

/// How a click's modifier keys combine with the selection.
#[must_use]
pub const fn click_extend(ctrl_or_meta: bool, shift: bool) -> Extend {
    if shift {
        Extend::Range
    } else if ctrl_or_meta {
        Extend::Toggle
    } else {
        Extend::Replace
    }
}

/// Map a key pressed on item `item` to a machine event. `columns` is the number
/// of tiles per row in grid view (1 in list view). `Some` means: prevent default.
#[must_use]
pub fn item_keydown(
    key: &str,
    ctrl_or_meta: bool,
    shift: bool,
    alt: bool,
    columns: usize,
    item: &str,
    view: ViewMode,
) -> Option<FileManagerEvent> {
    let cols = isize::try_from(columns.max(1)).unwrap_or(1);
    let grid = view == ViewMode::Grid;
    let go = |step| Some(FileManagerEvent::Move { step, extend: shift });
    match key {
        "ArrowUp" if alt => Some(FileManagerEvent::Up),
        "ArrowDown" => go(Step::By(cols)),
        "ArrowUp" => go(Step::By(-cols)),
        "ArrowRight" if grid => go(Step::By(1)),
        "ArrowLeft" if grid => go(Step::By(-1)),
        "Home" => go(Step::First),
        "End" => go(Step::Last),
        "Enter" => Some(FileManagerEvent::Activate(item.to_string())),
        "F2" => Some(FileManagerEvent::StartRename(item.to_string())),
        " " => Some(FileManagerEvent::Click { id: item.to_string(), extend: Extend::Toggle }),
        "a" | "A" if ctrl_or_meta => Some(FileManagerEvent::SelectAll),
        "Delete" => Some(FileManagerEvent::DeleteSelected),
        "Backspace" if ctrl_or_meta => Some(FileManagerEvent::DeleteSelected),
        "Escape" => Some(FileManagerEvent::ClearSelection),
        _ => None,
    }
}

/// Map a key pressed in the rename field to a machine event.
#[must_use]
pub fn rename_keydown(key: &str) -> Option<FileManagerEvent> {
    match key {
        "Enter" => Some(FileManagerEvent::CommitRename),
        "Escape" => Some(FileManagerEvent::CancelRename),
        _ => None,
    }
}

/// Context menu value: open the selection.
pub const OPEN: &str = "open";
/// Context menu value: rename the selected item.
pub const RENAME: &str = "rename";
/// Context menu value: delete the selection.
pub const DELETE: &str = "delete";

/// The context menu's entries. Rename needs exactly one item selected.
#[must_use]
pub fn menu_entries(state: &FileManagerState) -> Vec<MenuEntry> {
    let some = !state.selection.is_empty();
    let one = state.selection.len() == 1;
    let gate = |value: &str, on: bool| if on { MenuEntry::new(value) } else { MenuEntry::new(value).disabled() };
    vec![gate(OPEN, some), gate(RENAME, one), gate(DELETE, some)]
}

/// The machine event for an activated context menu value.
#[must_use]
pub fn menu_event(value: &str, state: &FileManagerState) -> Option<FileManagerEvent> {
    let first = state.selection.first()?.clone();
    match value {
        OPEN => Some(FileManagerEvent::Activate(first)),
        RENAME if state.selection.len() == 1 => Some(FileManagerEvent::StartRename(first)),
        DELETE => Some(FileManagerEvent::DeleteSelected),
        _ => None,
    }
}

/// The toolbar's selection text: `"2 selected"`, or the item count when nothing is.
#[must_use]
pub fn count_text(state: &FileManagerState, shown: usize) -> String {
    let n = state.selection.len();
    if n > 0 {
        return format!("{n} selected");
    }
    match shown {
        0 => "Empty folder".to_string(),
        1 => "1 item".to_string(),
        n => format!("{n} items"),
    }
}

/// What the live region says after a folder change.
#[must_use]
pub fn folder_announcement(items: &[FileItem], folder: &str) -> String {
    let name = items.iter().find(|i| i.id == folder).map_or("Files", |i| i.name.as_str());
    let n = items.iter().filter(|i| i.parent == folder).count();
    format!("{name}, {n} {}", if n == 1 { "item" } else { "items" })
}

/// The tree node id for folder `folder`. The root has one too.
#[must_use]
pub fn tree_id(folder: &str) -> String {
    format!("f:{folder}")
}

/// The folder a tree node id stands for.
#[must_use]
pub fn folder_of_tree_id(node: &str) -> Option<&str> {
    node.strip_prefix("f:")
}

/// The sidebar tree: the root, then every folder under it. Files are left out.
#[must_use]
pub fn tree_nodes(items: &[FileItem], root_label: &str) -> Vec<TreeNode> {
    fn children(items: &[FileItem], parent: &str, depth: usize) -> Vec<TreeNode> {
        if depth > items.len() {
            return Vec::new();
        }
        visible(items, parent)
            .into_iter()
            .filter(FileItem::is_folder)
            .map(|f| TreeNode::new(tree_id(&f.id), f.name.clone()).children(children(items, &f.id, depth + 1)))
            .collect()
    }
    vec![TreeNode::new(tree_id(ROOT), root_label).children(children(items, ROOT, 0))]
}

/// The tree node ids that must be open to show `folder`: the root and every ancestor.
#[must_use]
pub fn tree_open_ids(items: &[FileItem], folder: &str) -> Vec<String> {
    let mut ids = vec![tree_id(ROOT)];
    ids.extend(path(items, folder).iter().map(|f| tree_id(&f.id)));
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<FileItem> {
        vec![
            FileItem::folder("docs", "Docs", ROOT),
            FileItem::folder("old", "Old", "docs"),
            FileItem::file("a", "a.pdf", ROOT, 2048),
        ]
    }

    #[test]
    fn keys_map_by_view() {
        let k = |key, shift, view| item_keydown(key, false, shift, false, 4, "a", view);
        assert_eq!(
            k("ArrowDown", false, ViewMode::Grid),
            Some(FileManagerEvent::Move { step: Step::By(4), extend: false })
        );
        assert_eq!(
            k("ArrowDown", false, ViewMode::List),
            Some(FileManagerEvent::Move { step: Step::By(4), extend: false })
        );
        assert_eq!(
            k("ArrowRight", true, ViewMode::Grid),
            Some(FileManagerEvent::Move { step: Step::By(1), extend: true })
        );
        assert_eq!(k("ArrowRight", false, ViewMode::List), None);
        assert_eq!(k("F2", false, ViewMode::List), Some(FileManagerEvent::StartRename("a".into())));
        assert_eq!(item_keydown("a", true, false, false, 1, "a", ViewMode::List), Some(FileManagerEvent::SelectAll));
        assert_eq!(item_keydown("ArrowUp", false, false, true, 1, "a", ViewMode::List), Some(FileManagerEvent::Up));
        assert_eq!(rename_keydown("Escape"), Some(FileManagerEvent::CancelRename));
        assert_eq!(click_extend(true, false), Extend::Toggle);
        assert_eq!(click_extend(true, true), Extend::Range);
    }

    #[test]
    fn menu_needs_a_selection_and_rename_exactly_one() {
        let mut s = FileManagerState::default();
        assert!(menu_entries(&s).iter().all(|e| e.disabled));
        assert_eq!(menu_event(OPEN, &s), None);
        s.selection = vec!["a".into()];
        assert!(menu_entries(&s).iter().all(|e| !e.disabled));
        assert_eq!(menu_event(RENAME, &s), Some(FileManagerEvent::StartRename("a".into())));
        s.selection.push("docs".into());
        assert!(menu_entries(&s)[1].disabled);
        assert_eq!(menu_event(RENAME, &s), None);
        assert_eq!(menu_event(DELETE, &s), Some(FileManagerEvent::DeleteSelected));
        assert_eq!(count_text(&s, 5), "2 selected");
        assert_eq!(count_text(&FileManagerState::default(), 0), "Empty folder");
    }

    #[test]
    fn tree_holds_folders_only_and_opens_the_path() {
        let nodes = tree_nodes(&items(), "All files");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].children.len(), 1);
        assert_eq!(nodes[0].children[0].children[0].id, "f:old");
        assert_eq!(tree_open_ids(&items(), "old"), ["f:", "f:docs", "f:old"]);
        assert_eq!(folder_of_tree_id("f:docs"), Some("docs"));
        assert_eq!(folder_of_tree_id("f:"), Some(ROOT));
    }

    #[test]
    fn grid_columns_follow_the_width() {
        assert_eq!(grid_columns(0.0), 1);
        assert_eq!(grid_columns(120.0), 1);
        assert_eq!(grid_columns(256.0), 2);
        assert_eq!(grid_columns(640.0), 5);
        assert_eq!(grid_columns(f64::NAN), 1);
    }

    #[test]
    fn attrs_and_text() {
        let list = visible(&items(), ROOT);
        let s = FileManagerState { selection: vec!["a".into()], ..FileManagerState::default() };
        let a = item_attrs("r", &s, &list, &list[1]);
        assert_eq!((a.selected, a.tab_stop, a.kind), (true, false, "pdf"));
        assert!(item_attrs("r", &s, &list, &list[0]).tab_stop);
        assert_eq!(size_text(&list[1]), "2.0 KB");
        assert_eq!(size_text(&list[0]), "—");
        assert_eq!(folder_announcement(&items(), "docs"), "Docs, 1 item");
    }
}
