use hydroxy_leptos::file_manager::*;
use leptos::prelude::*;

fn files() -> Vec<FileItem> {
    vec![
        FileItem::folder("research", "Research", ROOT).modified("Oct 12"),
        FileItem::folder("interviews", "Interviews", "research").modified("Oct 12"),
        FileItem::file("q3", "interviews-q3.pdf", "interviews", 2_516_582).modified("Oct 9"),
        FileItem::file("q4", "interviews-q4.pdf", "interviews", 1_940_000).modified("Oct 11"),
        FileItem::file("tickets", "tickets-sept.csv", "research", 540_000).modified("Oct 2"),
        FileItem::folder("summaries", "Summaries", ROOT).modified("Oct 20"),
        FileItem::file("weekly", "weekly-digest.md", "summaries", 4_300).modified("Oct 20"),
        FileItem::file("board", "board-update.md", "summaries", 12_800).modified("Oct 18"),
        FileItem::folder("archive", "Archive", ROOT).modified("Sep 1"),
        FileItem::file("readme", "README.md", ROOT, 1_200).modified("Aug 14"),
        FileItem::file("deck", "board-update.pptx", ROOT, 8_400_000).modified("Oct 18"),
    ]
}

#[component]
pub fn Demo() -> impl IntoView {
    // The component never edits your items: renames and deletes come back as
    // callbacks, and you pass the new list in.
    let items = RwSignal::new(files());
    let (last, set_last) = signal(String::from("Double-click a file, press F2 to rename, Delete to remove."));

    view! {
        <div style="display: grid; gap: var(--space-3);">
            <FileManager
                items=items
                default_folder="research"
                on_open=move |id: String| set_last.set(format!("Opened {id}"))
                on_rename={move |(id, name): (String, String)| items.update(|list| apply_rename(list, &id, &name))}
                on_delete={move |ids: Vec<String>| items.update(|list| apply_delete(list, &ids))}
            />
            <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">{last}</p>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "file-manager",
    name: "File manager",
    group: "Data",
    summary: "A folder tree, breadcrumbs and a list or grid of the open folder. Select with the mouse or keyboard, rename in place, delete, and right-click for a context menu. You keep the items; renames and deletes come back as callbacks.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("file_manager.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/file_manager.rs"),
    parts: &[
        &["FileManager", "root", "Grid of sidebar and main. data-view=list|grid"],
        &["(inside)", "sidebar", "Holds a TreeView of the folders"],
        &["(inside)", "toolbar", "Breadcrumbs, the count and the list/grid toggle"],
        &[
            "(inside)",
            "breadcrumbs, crumb",
            "<nav><ol> of buttons from the top folder down. aria-current=page on the open one",
        ],
        &["(inside)", "count, view-toggle", "\"3 selected\" or the item count, and two aria-pressed buttons"],
        &["(inside)", "items", "<ul role=listbox aria-multiselectable>, inside a ContextMenu trigger"],
        &["(inside)", "item", "<li role=option>. data-kind, data-selected, data-renaming. Roving tabindex"],
        &["(inside)", "icon, badge, name, modified, size", "Folder icon or file-type badge, then the details"],
        &["(inside)", "rename, rename-error", "Text field in place of the name while renaming, and its alert"],
        &["(inside)", "empty, live", "Note for an empty folder; a role=status region that announces folder changes"],
        &["ContextMenu", "trigger, content, item", "Open, Rename and Delete, from the ContextMenu component"],
    ],
    keys: &[
        &["ArrowUp / ArrowDown", "Moves focus to the previous or next item (a row up or down in the grid)"],
        &["ArrowLeft / ArrowRight", "Moves focus in the grid"],
        &["Shift + arrows", "Extends the selection from where it started"],
        &["Home / End", "First or last item"],
        &["Space", "Toggles the focused item in the selection"],
        &["Ctrl or Cmd + A", "Selects everything in the folder"],
        &["Enter", "Opens the folder, or the file"],
        &["Alt + ArrowUp", "Goes to the parent folder"],
        &["F2", "Renames the focused item. Enter saves, Escape cancels"],
        &["Delete / Cmd + Backspace", "Asks you to delete the selection"],
        &["Escape", "Clears the selection"],
        &["Shift + F10 / ContextMenu", "Opens the context menu"],
    ],
    props: &[
        &[
            "items",
            "Vec<FileItem>",
            "Every file and folder. FileItem::folder(id, name, parent) and FileItem::file(id, name, parent, size)",
        ],
        &["folder", "Option<String>", "Controlled open folder id. ROOT (\"\") is the top. Pair with on_folder_change"],
        &["default_folder", "String", "Folder to start in when uncontrolled"],
        &["default_view", "ViewMode", "List (default) or Grid"],
        &["root_label", "String", "Name of the top folder. Default \"All files\""],
        &["on_folder_change", "Callback<String>", "Another folder opened"],
        &["on_selection_change", "Callback<Vec<String>>", "The selected ids changed"],
        &["on_open", "Callback<String>", "A file was opened"],
        &["on_rename", "Callback<(String, String)>", "(id, new name). Apply with apply_rename"],
        &[
            "on_delete",
            "Callback<Vec<String>>",
            "Ids to delete. Apply with apply_delete, which also removes what's inside folders",
        ],
    ],
    events: &[],
};
