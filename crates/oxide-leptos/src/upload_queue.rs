//! `<UploadQueue>`: a list of uploads with progress, pause, resume, cancel and
//! retry. The queue never sends a byte. Make an [`UploadQueueHandle`], hand
//! it to the component, start and abort transfers when it says so, and report
//! progress back through the handle.

use leptos::either::Either;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use oxide_headless::upload_queue::{self as h, UploadItem, UploadQueueEvent, UploadQueueInput, UploadQueueMachine, UploadQueueState};
use oxide_statechart::Machine;

use crate::icon::{Icon, IconName};
use crate::progress_bar::{ProgressBar, ProgressSize};

pub use oxide_headless::upload_queue::{UploadFile, UploadItem as Item, UploadQueueCommand, UploadStatus};

/// The queue's state plus the owner's side of it. Copy it freely.
#[derive(Clone, Copy)]
pub struct UploadQueueHandle {
    state: RwSignal<UploadQueueState>,
    input: StoredValue<UploadQueueInput>,
    on_command: Callback<UploadQueueCommand>,
    live: RwSignal<String>,
}

impl UploadQueueHandle {
    /// A queue that runs at most `concurrency` transfers at once. `on_command`
    /// hears when to start or abort a transfer and when an item is removed.
    pub fn new(concurrency: usize, on_command: impl Fn(UploadQueueCommand) + Send + Sync + 'static) -> Self {
        let input = UploadQueueInput { concurrency };
        Self {
            state: RwSignal::new(UploadQueueMachine::initial(&input)),
            input: StoredValue::new(input),
            on_command: Callback::new(on_command),
            live: RwSignal::new(String::new()),
        }
    }

    /// Feed an event in and run the commands it produces.
    pub fn send(self, event: UploadQueueEvent) {
        let Some(step) = self
            .input
            .try_with_value(|i| self.state.try_with_untracked(|s| UploadQueueMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        let items = step.state.as_ref().map_or_else(|| self.state.with_untracked(|s| s.items.clone()), |s| s.items.clone());
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            if let Some(text) = h::announcement(&command, &items) {
                self.live.set(text);
            }
            self.on_command.run(command);
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

    /// A snapshot of the items. Reactive when read in a tracking context.
    #[must_use]
    pub fn items(self) -> Vec<UploadItem> {
        self.state.with(|s| s.items.clone())
    }
}

/// The list. Each row shows the file, its progress and the buttons that fit its state.
#[component]
pub fn UploadQueue(
    /// The queue to show.
    queue: UploadQueueHandle,
    /// Accessible name for the list.
    #[prop(default = "Uploads".into(), into)]
    label: String,
) -> impl IntoView {
    let ids = Memo::new(move |_| queue.state.with(|s| s.items.iter().map(|i| i.id.clone()).collect::<Vec<_>>()));
    let items = Memo::new(move |_| queue.items());
    let finished = move || items.with(|is| is.iter().any(|i| matches!(i.status, UploadStatus::Done | UploadStatus::Cancelled)));

    view! {
        <div data-scope=h::SCOPE data-part="root">
            <div data-scope=h::SCOPE data-part="header">
                <span data-scope=h::SCOPE data-part="summary">{move || items.with(|is| h::summary_text(is))}</span>
                <Show when=finished>
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="clear"
                        on:click=move |ev| {
                            // This button leaves with the finished rows; keep the keyboard in the list.
                            focus_action(&ev, |list| list.query_selector("[data-part=item]:not([data-state=done]):not([data-state=cancelled]) [data-part=action]").ok().flatten());
                            queue.send(UploadQueueEvent::ClearFinished);
                        }
                    >
                        "Clear finished"
                    </button>
                </Show>
            </div>
            <ul data-scope=h::SCOPE data-part="list" aria-label=label>
                <For each=move || ids.get() key=String::clone children=move |id| item_view(queue, items, id) />
            </ul>
            <div data-scope=h::SCOPE data-part="live" role="status" aria-live="polite" aria-atomic="true">
                {move || queue.live.get()}
            </div>
        </div>
    }
}

fn item_view(queue: UploadQueueHandle, items: Memo<Vec<UploadItem>>, id: String) -> impl IntoView {
    let item = {
        let id = id.clone();
        Memo::new(move |_| items.with(|is| is.iter().find(|i| i.id == id).cloned()))
    };
    let name = move || item.with(|i| i.as_ref().map(|i| i.name.clone()).unwrap_or_default());
    let buttons = move || {
        item.with(|i| i.as_ref().map(h::actions).unwrap_or_default())
            .into_iter()
            .map(|action| {
                let id = id.clone();
                let label = action.label(&name());
                view! {
                    <button
                        type="button"
                        title=label.clone()
                        aria-label=label
                        data-scope=h::SCOPE
                        data-part="action"
                        data-action=action.as_str()
                        on:click=move |ev| {
                            // The row is about to go, so hand focus to a neighbour.
                            if action == h::Action::Remove {
                                focus_neighbour(&ev);
                            }
                            queue.send(action.event(&id));
                        }
                    >
                        {action_icon(action)}
                    </button>
                }
            })
            .collect_view()
    };

    view! {
        <li data-scope=h::SCOPE data-part="item" data-state=move || item.with(|i| i.as_ref().map(|i| i.status.as_str()))>
            <ProgressBar
                label=name()
                size=ProgressSize::Sm
                value=Signal::derive(move || item.with(|i| i.as_ref().map(|i| f64::from(i.percent()))))
                value_text=Signal::derive(move || item.with(|i| i.as_ref().map(h::status_text)))
            />
            <span data-scope=h::SCOPE data-part="actions">{buttons}</span>
        </li>
    }
}

/// Remove is an icon; the rest are short words, since the icon set has no glyph for them.
fn action_icon(action: h::Action) -> impl IntoView {
    match action {
        h::Action::Remove => Either::Left(view! { <Icon name=IconName::X size=14 /> }),
        other => Either::Right(view! { <span aria-hidden="true">{other.word()}</span> }),
    }
}

/// Focus the first button of the row after (or else before) the one `ev` came from.
fn focus_neighbour(ev: &leptos::ev::MouseEvent) {
    let row = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()).and_then(|e| e.closest("li").ok().flatten());
    let near = row.and_then(|r| r.next_element_sibling().or_else(|| r.previous_element_sibling()));
    if let Some(button) = near.and_then(|n| n.query_selector("[data-part=action]").ok().flatten()) {
        let _ = button.unchecked_into::<web_sys::HtmlElement>().focus();
    }
}

/// Focus the element `find` picks inside the queue that `ev`'s button belongs to.
fn focus_action(ev: &leptos::ev::MouseEvent, find: impl Fn(&web_sys::Element) -> Option<web_sys::Element>) {
    let root = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()).and_then(|e| e.closest("[data-part=root]").ok().flatten());
    if let Some(target) = root.and_then(|r| find(&r)) {
        let _ = target.unchecked_into::<web_sys::HtmlElement>().focus();
    }
}
