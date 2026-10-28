//! `UploadQueue`: a list of uploads with progress, pause, resume, cancel and
//! retry. The queue never sends a byte. Make a handle with [`use_upload_queue`],
//! pass it to the component, start and abort transfers when it says so, and
//! report progress back through the handle.

use dioxus::prelude::*;
use hydroxy_headless::upload_queue::{self as h, UploadItem, UploadQueueEvent, UploadQueueInput, UploadQueueMachine, UploadQueueState};
use hydroxy_statechart::Machine;

use crate::icon::{Icon, IconName};
use crate::progress_bar::{ProgressBar, ProgressSize};

pub use hydroxy_headless::upload_queue::{UploadFile, UploadItem as Item, UploadQueueCommand, UploadStatus};

/// The queue's state plus the owner's side of it. Copy it freely.
#[derive(Clone, Copy, PartialEq)]
pub struct UploadQueueHandle {
    state: Signal<UploadQueueState>,
    concurrency: usize,
    on_command: Callback<UploadQueueCommand>,
    live: Signal<String>,
}

/// Make a queue that runs at most `concurrency` transfers at once. `on_command`
/// hears when to start or abort a transfer and when an item is removed.
pub fn use_upload_queue(concurrency: usize, on_command: impl FnMut(UploadQueueCommand) + 'static) -> UploadQueueHandle {
    use_hook(|| UploadQueueHandle {
        state: Signal::new(UploadQueueMachine::initial(&UploadQueueInput { concurrency })),
        concurrency,
        on_command: Callback::new(on_command),
        live: Signal::new(String::new()),
    })
}

impl UploadQueueHandle {
    /// Feed an event in and run the commands it produces.
    pub fn send(mut self, event: UploadQueueEvent) {
        let step = UploadQueueMachine::transition(&self.state.peek(), event, &UploadQueueInput { concurrency: self.concurrency });
        let items = step.state.as_ref().map_or_else(|| self.state.peek().items.clone(), |s| s.items.clone());
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            if let Some(text) = h::announcement(&command, &items) {
                self.live.set(text);
            }
            self.on_command.call(command);
        }
    }

    /// Queue files. They start as slots free up.
    pub fn add(self, files: Vec<UploadFile>) {
        self.send(UploadQueueEvent::Add(files));
    }

    /// `id` has sent `sent` bytes so far.
    pub fn progress(self, id: impl Into<String>, sent: u64) {
        self.send(UploadQueueEvent::Progress { id: id.into(), sent });
    }

    /// `id` is fully sent.
    pub fn complete(self, id: impl Into<String>) {
        self.send(UploadQueueEvent::Complete(id.into()));
    }

    /// `id` failed with `message`.
    pub fn fail(self, id: impl Into<String>, message: impl Into<String>) {
        self.send(UploadQueueEvent::Fail { id: id.into(), message: message.into() });
    }

    /// A snapshot of the items.
    #[must_use]
    pub fn items(self) -> Vec<UploadItem> {
        self.state.read().items.clone()
    }
}

/// The list. Each row shows the file, its progress and the buttons that fit its state.
#[component]
pub fn UploadQueue(
    /// The queue to show.
    queue: UploadQueueHandle,
    /// Accessible name for the list.
    #[props(default = "Uploads".to_string())]
    label: String,
) -> Element {
    let items = queue.items();
    let finished = items.iter().any(|i| matches!(i.status, UploadStatus::Done | UploadStatus::Cancelled));
    let live = queue.live;

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root",
            div { "data-scope": h::SCOPE, "data-part": "header",
                span { "data-scope": h::SCOPE, "data-part": "summary", {h::summary_text(&items)} }
                if finished {
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "clear",
                        onclick: move |_| queue.send(UploadQueueEvent::ClearFinished),
                        "Clear finished"
                    }
                }
            }
            ul { "data-scope": h::SCOPE, "data-part": "list", "aria-label": label,
                for item in items.iter() {
                    li {
                        key: "{item.id}",
                        "data-scope": h::SCOPE,
                        "data-part": "item",
                        "data-state": item.status.as_str(),
                        ProgressBar {
                            label: item.name.clone(),
                            size: ProgressSize::Sm,
                            value: Some(f64::from(item.percent())),
                            value_text: Some(h::status_text(item)),
                        }
                        span { "data-scope": h::SCOPE, "data-part": "actions",
                            for action in h::actions(item) {
                                {
                                    let id = item.id.clone();
                                    let text = action.label(&item.name);
                                    rsx! {
                                        button {
                                            key: "{action.as_str()}",
                                            r#type: "button",
                                            title: text.clone(),
                                            "aria-label": text,
                                            "data-scope": h::SCOPE,
                                            "data-part": "action",
                                            "data-action": action.as_str(),
                                            onclick: move |_| queue.send(action.event(&id)),
                                            if action == h::Action::Remove {
                                                Icon { name: IconName::X, size: 14 }
                                            } else {
                                                span { "aria-hidden": "true", {action.word()} }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "live", role: "status", "aria-live": "polite", "aria-atomic": "true", "{live}" }
        }
    }
}
