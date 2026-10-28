use dioxus::prelude::*;
use hydroxy_dioxus::file_dropzone::FileDropzone;

#[component]
pub fn FileDropzoneDemo() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        div { style: "display: grid; gap: var(--space-3); max-width: 460px;",
            FileDropzone {
                hint: "PDF or images, up to 5 MB, at most 3 files",
                accept: ".pdf,image/*",
                max_size: 5 * 1024 * 1024,
                max_files: 3,
                on_files_change: move |files: Vec<_>| count.set(files.len()),
            }
            p { style: "margin: 0; font: var(--text-xs); color: var(--text-3);", "{count} attached" }
        }
    }
}
