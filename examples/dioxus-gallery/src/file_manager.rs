use dioxus::prelude::*;
use oxide_dioxus::file_manager::{FileItem, FileManager, ROOT, apply_delete, apply_rename};

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
pub fn FileManagerDemo() -> Element {
    // The component never edits your items: renames and deletes come back as
    // callbacks, and you pass the new list in.
    let mut items = use_signal(files);
    let mut last = use_signal(|| String::from("Double-click a file, press F2 to rename, Delete to remove."));

    rsx! {
        div { style: "display: grid; gap: var(--space-3);",
            FileManager {
                items: items(),
                default_folder: "research",
                on_open: move |id: String| last.set(format!("Opened {id}")),
                on_rename: move |(id, name): (String, String)| apply_rename(&mut items.write(), &id, &name),
                on_delete: move |ids: Vec<String>| apply_delete(&mut items.write(), &ids),
            }
            p { style: "margin: 0; font: var(--text-xs); color: var(--text-3);", "{last}" }
        }
    }
}
