use hydroxy_leptos::file_dropzone::FileDropzone;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let (count, set_count) = signal(0);

    view! {
        <div style="display: grid; gap: var(--space-3); max-width: 460px;">
            <FileDropzone
                hint="PDF or images, up to 5 MB, at most 3 files"
                accept=".pdf,image/*"
                max_size={5 * 1024 * 1024}
                max_files=3
                on_files_change={move |files: Vec<web_sys::File>| set_count.set(files.len())}
            />
            <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">
                {move || format!("{} attached", count.get())}
            </p>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "file-dropzone",
    name: "File dropzone",
    group: "Forms",
    summary: "Drop files or browse for them, then review and remove them. A native file input does the picking; type, size and count limits are checked before files are added.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("file_dropzone.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/file_dropzone.rs"),
    parts: &[
        &["FileDropzone", "root", "Wrapper. data-disabled"],
        &["(inside)", "dropzone", "<label> that takes drops and opens the picker. data-state=dragging|idle"],
        &["(inside)", "input", "Native <input type=file>, visually hidden but focusable. accept, multiple"],
        &["(inside)", "icon / label / hint", "Drop area content; also the input's accessible name"],
        &["(inside)", "error", "role=alert. One line per rejected file and why"],
        &["(inside)", "list", "<ul> of picked files, shown once there is one"],
        &["(inside)", "item", "One file: icon, name, size and remove"],
        &["(inside)", "remove", "<button aria-label=\"Remove name\">. Focus returns to the input"],
    ],
    keys: &[
        &["Space / Enter", "Open the file picker from the focused drop area"],
        &["Tab", "Drop area, then each remove button"],
    ],
    props: &[
        &["label", "String", "Drop area text. Default \"Drop files here or click to browse\""],
        &["hint", "String", "Smaller text under the label"],
        &["accept", "String", "Allowed types like the native accept: .pdf,image/*"],
        &["max_size", "u64", "Largest file in bytes"],
        &["max_files", "usize", "Most files held at once"],
        &["multiple", "bool", "Hold many files. Off: a new pick replaces the file. Default true"],
        &["disabled", "bool", "Ignore drops, picks and removals"],
        &[
            "on_files_change",
            "Callback<Vec<File>>",
            "The whole list after every change. web_sys::File in Leptos, FileData in Dioxus",
        ],
        &["on_reject", "Callback<Vec<Rejection>>", "Turned-away files with a FileType, FileSize or FileCount reason"],
    ],
    events: &[],
};
