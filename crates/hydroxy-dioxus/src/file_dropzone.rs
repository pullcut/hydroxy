//! `FileDropzone`: drop files or browse for them, then see and remove them.
//! The drop area is a `<label>` around a visually hidden native
//! `<input type="file">`: clicking opens the picker, the input takes focus and
//! Enter or Space opens it. Type, size and count limits are checked before
//! files are added; rejected files are listed in an alert.
//!
//! Files arrive as Dioxus [`FileData`], which reads bytes on web and desktop alike.

use std::rc::Rc;

use dioxus::html::FileData;
use dioxus::prelude::*;
use hydroxy_headless::file_dropzone::{
    self as h, FileDropzoneCommand, FileDropzoneEvent, FileDropzoneInput, FileDropzoneMachine, PickedFile, Rejection,
};
use hydroxy_headless::icons::Icon as IconName;
use hydroxy_statechart::Machine;

use crate::icon::Icon;

type M = FileDropzoneMachine<FileData>;

fn picked(files: Vec<FileData>) -> Vec<PickedFile<FileData>> {
    files
        .into_iter()
        .map(|file| PickedFile {
            name: file.name(),
            size: file.size(),
            mime: file.content_type().unwrap_or_default(),
            file,
        })
        .collect()
}

/// A drop area and file list. Holds the files itself and reports every change.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn FileDropzone(
    /// Main text in the drop area.
    #[props(default = "Drop files here or click to browse".to_string())]
    label: String,
    /// Smaller text under the label, such as "PDF or images, up to 5 MB".
    hint: Option<String>,
    /// Allowed types, like the native `accept`: `.pdf,image/*`. Empty allows all.
    #[props(default)]
    accept: String,
    /// Largest file in bytes.
    max_size: Option<u64>,
    /// Most files held at once.
    max_files: Option<usize>,
    /// Hold many files. Off: a new pick replaces the current file.
    #[props(default = true)]
    multiple: bool,
    /// Ignore drops, picks and removals.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Called with the whole file list after every add or remove.
    on_files_change: Option<EventHandler<Vec<FileData>>>,
    /// Called with the files that were turned away and why.
    on_reject: Option<EventHandler<Vec<Rejection<FileData>>>>,
) -> Element {
    let input = {
        let accept = accept.clone();
        use_memo(move || FileDropzoneInput {
            accept: accept.clone(),
            max_size,
            max_files,
            multiple,
            disabled: disabled(),
        })
    };
    let mut state = use_signal(|| M::initial(&input.peek()));
    let mut errors = use_signal(Vec::<String>::new);
    let mut picker = use_signal(|| None::<Rc<MountedData>>);

    let mut send = move |event: FileDropzoneEvent<FileData>| {
        let step = M::transition(&state.peek(), event, &input.peek());
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                FileDropzoneCommand::FilesChange(files) => {
                    errors.set(Vec::new());
                    if let Some(handler) = on_files_change {
                        handler.call(files.into_iter().map(|f| f.file).collect());
                    }
                }
                FileDropzoneCommand::Reject(rejections) => {
                    errors.set(h::rejection_text(&rejections));
                    if let Some(handler) = on_reject {
                        handler.call(rejections);
                    }
                }
            }
        }
    };

    let s = state.read();
    let is_disabled = input.read().disabled;
    let files: Vec<(String, u64)> = s.files.iter().map(|f| (f.name.clone(), f.size)).collect();

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-disabled": is_disabled.then_some(""),
            label {
                "data-scope": h::SCOPE,
                "data-part": "dropzone",
                "data-state": h::data_state(&s),
                ondragenter: move |e| {
                    e.prevent_default();
                    send(FileDropzoneEvent::DragEnter);
                },
                ondragover: move |e| e.prevent_default(),
                ondragleave: move |_| send(FileDropzoneEvent::DragLeave),
                ondrop: move |e| {
                    e.prevent_default();
                    send(FileDropzoneEvent::Add(picked(e.data_transfer().files())));
                },
                input {
                    r#type: "file",
                    accept: (!accept.is_empty()).then_some(accept),
                    multiple,
                    disabled: is_disabled,
                    "data-scope": h::SCOPE,
                    "data-part": "input",
                    onmounted: move |e| picker.set(Some(e.data())),
                    // the input keeps its value, so re-picking the same file after a remove
                    // fires no change; clearing it needs a web-only API.
                    onchange: move |e| send(FileDropzoneEvent::Add(picked(e.files()))),
                }
                span { "data-scope": h::SCOPE, "data-part": "icon", "aria-hidden": "true",
                    Icon { name: IconName::ArrowUp, size: 18 }
                }
                span { "data-scope": h::SCOPE, "data-part": "label", {label} }
                if let Some(text) = hint {
                    span { "data-scope": h::SCOPE, "data-part": "hint", {text} }
                }
            }
            div { role: "alert", "data-scope": h::SCOPE, "data-part": "error",
                for line in errors() {
                    p { {line} }
                }
            }
            if !files.is_empty() {
                ul { "data-scope": h::SCOPE, "data-part": "list", "aria-label": "Selected files",
                    for (index, (name, size)) in files.into_iter().enumerate() {
                        li { key: "{index}-{name}", "data-scope": h::SCOPE, "data-part": "item",
                            Icon { name: IconName::FileText, size: 16 }
                            span { "data-scope": h::SCOPE, "data-part": "name", "{name}" }
                            span { "data-scope": h::SCOPE, "data-part": "size", {h::format_size(size)} }
                            button {
                                r#type: "button",
                                "aria-label": "Remove {name}",
                                disabled: is_disabled,
                                "data-scope": h::SCOPE,
                                "data-part": "remove",
                                onclick: move |_| {
                                    send(FileDropzoneEvent::Remove(index));
                                    // The removed button took focus with it.
                                    if let Some(el) = picker.peek().clone() {
                                        spawn(async move {
                                            let _ = el.set_focus(true).await;
                                        });
                                    }
                                },
                                Icon { name: IconName::X, size: 14 }
                            }
                        }
                    }
                }
            }
        }
    }
}
