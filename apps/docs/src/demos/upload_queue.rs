use std::time::Duration;

use hydroxy_leptos::button::{Button, ButtonVariant};
use hydroxy_leptos::file_dropzone::FileDropzone;
use hydroxy_leptos::upload_queue::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (last, set_last) = signal(String::from("none yet"));
    let queue = UploadQueueHandle::new(2, move |command| {
        set_last.set(match command {
            UploadQueueCommand::Start { id, attempt, .. } => format!("start {id}, attempt {attempt}"),
            UploadQueueCommand::Abort(id) => format!("abort {id}"),
            UploadQueueCommand::Removed(id) => format!("removed {id}"),
            UploadQueueCommand::Completed(id) => format!("completed {id}"),
            UploadQueueCommand::Failed { id, .. } => format!("failed {id}"),
        });
    });
    let next = StoredValue::new(0_u32);
    let seen = StoredValue::new(Vec::<(String, u64)>::new());
    let add = move |files: Vec<UploadFile>| {
        queue.add(
            files
                .into_iter()
                .map(|f| UploadFile::new(format!("f{}", next.get_value()), f.name, f.size))
                .inspect(|_| next.update_value(|n| *n += 1))
                .collect(),
        );
    };

    // The queue only tracks; this timer plays the part of a network. It reports
    // progress for whatever is uploading, and fails corrupt.zip once.
    Effect::new(move |_| {
        let handle = set_interval_with_handle(
            move || {
                for item in queue.items().into_iter().filter(|i| i.status == UploadStatus::Uploading) {
                    let sent = item.sent + item.size / 8 + 1;
                    if item.name == "corrupt.zip" && item.attempt == 1 && sent * 2 > item.size {
                        queue.fail(&item.id, "Connection reset");
                    } else if sent >= item.size {
                        queue.complete(&item.id);
                    } else {
                        queue.progress(&item.id, sent);
                    }
                }
            },
            Duration::from_millis(450),
        )
        .ok();
        on_cleanup(move || {
            if let Some(handle) = handle {
                handle.clear();
            }
        });
    });

    view! {
        <div style="display: grid; gap: var(--space-4); max-width: 520px;">
            <FileDropzone
                hint="Picked files join the queue below"
                on_files_change={move |files: Vec<web_sys::File>| {
                    // The dropzone reports its whole list; queue the files it hasn't shown us yet.
                    let fresh: Vec<UploadFile> = files
                        .iter()
                        .filter_map(|f| {
                            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "File.size is a whole byte count")]
                            let key = (f.name(), f.size() as u64);
                            let new = !seen.with_value(|s| s.contains(&key));
                            new.then(|| {
                                seen.update_value(|s| s.push(key.clone()));
                                UploadFile::new("", key.0, key.1)
                            })
                        })
                        .collect();
                    add(fresh);
                }}
            />
            <div>
                <Button
                    variant=ButtonVariant::Secondary
                    on_click=move |_| {
                        add(vec![
                            UploadFile::new("", "interviews-q3.pdf", 2_400_000),
                            UploadFile::new("", "corrupt.zip", 800_000),
                            UploadFile::new("", "board-update.md", 12_000),
                            UploadFile::new("", "tickets-sept.csv", 540_000),
                        ]);
                    }
                >
                    "Add sample files"
                </Button>
            </div>
            <UploadQueue queue=queue />
            <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">"Last command: " {last}</p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "upload-queue",
    name: "Upload queue",
    group: "Forms",
    summary: "A list of uploads with progress, pause, resume, cancel and retry. The queue only tracks state: it tells you when to start or abort a transfer, and you report progress back.",
    demo,
    leptos: include_str!("upload_queue.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/upload_queue.rs"),
    parts: &[
        &["UploadQueue", "root", "Wrapper"],
        &[
            "(inside)",
            "header, summary, clear",
            "Counts such as \"2 of 4 uploaded\", and a Clear finished button once something is done or cancelled",
        ],
        &["(inside)", "list", "<ul> of uploads"],
        &["(inside)", "item", "<li>. data-state=queued|uploading|paused|done|failed|cancelled"],
        &[
            "(inside)",
            "actions, action",
            "Native buttons that fit the state, each labelled with the file name. data-action=pause|resume|cancel|retry|remove",
        ],
        &["(inside)", "live", "role=status region that announces finished and failed uploads"],
        &[
            "ProgressBar",
            "root, track, indicator",
            "Each row's progress bar. Its label is the file name and its text the status",
        ],
    ],
    keys: &[&["Tab", "Moves between the buttons of each upload"], &["Space / Enter", "Presses the focused button"]],
    props: &[
        &[
            "UploadQueueHandle::new(concurrency, on_command)",
            "fn",
            "Makes the queue. At most concurrency items upload at once. on_command hears Start, Abort, Removed, Completed and Failed",
        ],
        &["handle.add(files)", "Vec<UploadFile>", "Queue files. Each has an id, a name and a size in bytes"],
        &["handle.progress(id, sent)", "u64", "Bytes sent so far. Ignored unless the item is uploading"],
        &["handle.complete(id) / handle.fail(id, message)", "", "The transfer ended"],
        &["queue", "UploadQueueHandle", "The queue the component shows"],
        &["label", "String", "Accessible name of the list. Default \"Uploads\""],
    ],
};
