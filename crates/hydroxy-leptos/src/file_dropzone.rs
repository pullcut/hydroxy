//! `<FileDropzone>`: drop files or browse for them, then see and remove them.
//! The drop area is a `<label>` around a visually hidden native
//! `<input type="file">`: clicking opens the picker, the input takes focus and
//! Enter or Space opens it, with no script. Type, size and count limits are
//! checked before files are added; rejected files are listed in an alert.

use hydroxy_headless::file_dropzone::{
    self as h, FileDropzoneCommand, FileDropzoneEvent, FileDropzoneInput, FileDropzoneMachine, PickedFile, Rejection,
};
use hydroxy_headless::icons::Icon as IconName;
use hydroxy_statechart::Machine;
use leptos::html::Input;
use leptos::prelude::*;

use crate::icon::Icon;
use crate::local::{SendOption, browser_only};

type M = FileDropzoneMachine<web_sys::File>;
type State = <M as Machine>::State;

/// Read the state. Files are JS handles, so it is only held in the browser; a server render
/// reads the initial state.
fn read<R>(state: RwSignal<SendOption<State>>, input: Memo<FileDropzoneInput>, f: impl FnOnce(&State) -> R) -> R {
    state.with(|s| match s.as_ref() {
        Some(s) => f(s),
        None => f(&input.with_untracked(M::initial)),
    })
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "File.size is a whole, non-negative byte count"
)]
fn picked(list: Option<web_sys::FileList>) -> Vec<PickedFile<web_sys::File>> {
    let Some(list) = list else { return Vec::new() };
    (0..list.length())
        .filter_map(|i| list.get(i))
        .map(|file| PickedFile { name: file.name(), size: file.size() as u64, mime: file.type_(), file })
        .collect()
}

/// A drop area and file list. Holds the files itself and reports every change.
#[component]
pub fn FileDropzone(
    /// Main text in the drop area.
    #[prop(default = "Drop files here or click to browse".into(), into)]
    label: String,
    /// Smaller text under the label, such as "PDF or images, up to 5 MB".
    #[prop(optional, into)]
    hint: Option<String>,
    /// Allowed types, like the native `accept`: `.pdf,image/*`. Empty allows all.
    #[prop(optional, into)]
    accept: String,
    /// Largest file in bytes.
    #[prop(optional)]
    max_size: Option<u64>,
    /// Most files held at once.
    #[prop(optional)]
    max_files: Option<usize>,
    /// Hold many files. Off: a new pick replaces the current file.
    #[prop(default = true)]
    multiple: bool,
    /// Ignore drops, picks and removals.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Called with the whole file list after every add or remove.
    #[prop(optional, into)]
    on_files_change: Option<Callback<Vec<web_sys::File>>>,
    /// Called with the files that were turned away and why.
    #[prop(optional, into)]
    on_reject: Option<Callback<Vec<Rejection<web_sys::File>>>>,
) -> impl IntoView {
    let input = {
        let accept = accept.clone();
        Memo::new(move |_| FileDropzoneInput {
            accept: accept.clone(),
            max_size,
            max_files,
            multiple,
            disabled: disabled.get().unwrap_or(false),
        })
    };
    let state = RwSignal::new(browser_only(input.with_untracked(M::initial)));
    let errors = RwSignal::new(Vec::<String>::new());
    let picker = NodeRef::<Input>::new();
    let is_disabled = move || input.with(|i| i.disabled);

    let send = move |event: FileDropzoneEvent<web_sys::File>| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| s.as_ref().map(|s| M::transition(s, event, i))))
            .flatten()
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            state.set(browser_only(next));
        }
        for command in step.commands {
            match command {
                FileDropzoneCommand::FilesChange(files) => {
                    errors.set(Vec::new());
                    if let Some(cb) = on_files_change {
                        cb.run(files.into_iter().map(|f| f.file).collect());
                    }
                }
                FileDropzoneCommand::Reject(rejections) => {
                    errors.set(h::rejection_text(&rejections));
                    if let Some(cb) = on_reject {
                        cb.run(rejections);
                    }
                }
            }
        }
    };

    let item = move |(index, file): (usize, PickedFile<web_sys::File>)| {
        let remove_label = format!("Remove {}", file.name);
        view! {
            <li data-scope=h::SCOPE data-part="item">
                <Icon name=IconName::FileText size=16 />
                <span data-scope=h::SCOPE data-part="name">{file.name}</span>
                <span data-scope=h::SCOPE data-part="size">{h::format_size(file.size)}</span>
                <button
                    type="button"
                    aria-label=remove_label
                    disabled=is_disabled
                    data-scope=h::SCOPE
                    data-part="remove"
                    on:click=move |_| {
                        send(FileDropzoneEvent::Remove(index));
                        // The removed button took focus with it.
                        if let Some(el) = picker.get_untracked() {
                            let _ = el.focus();
                        }
                    }
                >
                    <Icon name=IconName::X size=14 />
                </button>
            </li>
        }
    };

    view! {
        <div data-scope=h::SCOPE data-part="root" data-disabled=move || is_disabled().then_some("")>
            <label
                data-scope=h::SCOPE
                data-part="dropzone"
                data-state=move || read(state, input, h::data_state)
                on:dragenter=move |ev| {
                    ev.prevent_default();
                    send(FileDropzoneEvent::DragEnter);
                }
                on:dragover=move |ev| ev.prevent_default()
                on:dragleave=move |_| send(FileDropzoneEvent::DragLeave)
                on:drop=move |ev| {
                    ev.prevent_default();
                    send(FileDropzoneEvent::Add(picked(ev.data_transfer().and_then(|dt| dt.files()))));
                }
            >
                <input
                    node_ref=picker
                    type="file"
                    accept=(!accept.is_empty()).then_some(accept)
                    multiple=multiple
                    disabled=is_disabled
                    data-scope=h::SCOPE
                    data-part="input"
                    on:change=move |_| {
                        let Some(el) = picker.get_untracked() else { return };
                        send(FileDropzoneEvent::Add(picked(el.files())));
                        // Let the same file be picked again after a remove.
                        el.set_value("");
                    }
                />
                <span data-scope=h::SCOPE data-part="icon" aria-hidden="true"><Icon name=IconName::ArrowUp size=18 /></span>
                <span data-scope=h::SCOPE data-part="label">{label}</span>
                {hint.map(|text| view! { <span data-scope=h::SCOPE data-part="hint">{text}</span> })}
            </label>
            <div role="alert" data-scope=h::SCOPE data-part="error">
                {move || errors.get().into_iter().map(|line| view! { <p>{line}</p> }).collect_view()}
            </div>
            <Show when=move || read(state, input, |s| !s.files.is_empty())>
                <ul data-scope=h::SCOPE data-part="list" aria-label="Selected files">
                    {move || read(state, input, |s| s.files.clone()).into_iter().enumerate().map(item).collect_view()}
                </ul>
            </Show>
        </div>
    }
}
