use dioxus::prelude::*;
use hydroxy_dioxus::button::{Button, ButtonVariant};
use hydroxy_dioxus::upload_queue::{UploadFile, UploadQueue, UploadQueueCommand, UploadStatus, use_upload_queue};

#[component]
pub fn UploadQueueDemo() -> Element {
    let mut last = use_signal(|| String::from("none yet"));
    let queue = use_upload_queue(2, move |command| {
        last.set(match command {
            UploadQueueCommand::Start { id, attempt, .. } => format!("start {id}, attempt {attempt}"),
            UploadQueueCommand::Abort(id) => format!("abort {id}"),
            UploadQueueCommand::Removed(id) => format!("removed {id}"),
            UploadQueueCommand::Completed(id) => format!("completed {id}"),
            UploadQueueCommand::Failed { id, .. } => format!("failed {id}"),
        });
    });
    let mut next = use_signal(|| 0_u32);

    // The queue only tracks; this button plays the part of a network. It
    // reports progress for whatever is uploading, and fails corrupt.zip once.
    let tick = move |_| {
        for item in queue.items().into_iter().filter(|i| i.status == UploadStatus::Uploading) {
            let sent = item.sent + item.size / 4 + 1;
            if item.name == "corrupt.zip" && item.attempt == 1 && sent * 2 > item.size {
                queue.fail(&item.id, "Connection reset");
            } else if sent >= item.size {
                queue.complete(&item.id);
            } else {
                queue.progress(&item.id, sent);
            }
        }
    };

    rsx! {
        div { style: "display: grid; gap: var(--space-4); max-width: 520px;",
            div { style: "display: flex; gap: var(--space-2);",
                Button {
                    variant: ButtonVariant::Secondary,
                    on_click: move |_| {
                        let files = ["interviews-q3.pdf", "corrupt.zip", "board-update.md", "tickets-sept.csv"]
                            .into_iter()
                            .zip([2_400_000, 800_000, 12_000, 540_000])
                            .map(|(name, size)| {
                                let id = format!("f{next}");
                                next += 1;
                                UploadFile::new(id, name, size)
                            })
                            .collect();
                        queue.add(files);
                    },
                    "Add sample files"
                }
                Button { variant: ButtonVariant::Ghost, on_click: tick, "Send more data" }
            }
            UploadQueue { queue }
            p { style: "margin: 0; font: var(--text-xs); color: var(--text-3);", "Last command: {last}" }
        }
    }
}
