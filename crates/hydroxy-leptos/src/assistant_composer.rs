//! The assistant composer family: [`AssistantComposer`] (text, queue, history, `@` and `/`
//! popover, attachments), [`AssistantQueueItem`], [`AssistantAttachment`],
//! [`AssistantSuggestion`] and the [`AssistantSuggestions`] grid.

use hydroxy_headless::assistant_composer::{
    self as h, AssistantComposerCommand, AssistantComposerEvent, AssistantComposerInput, AssistantComposerMachine,
    KeyAction, KeyPress, Popover,
};
use hydroxy_statechart::Machine;
use leptos::either::Either;
use leptos::html::{Input, Textarea};
use leptos::prelude::*;

use crate::drive::drive;
use crate::icon::{Icon, IconName};

pub use hydroxy_headless::assistant_composer::{AttachmentFile, AttachmentItem, AttachmentStatus};
pub use hydroxy_headless::assistant_thread::Suggestion;

mod attachment;
mod queue;
mod suggestions;

pub use attachment::*;
pub use queue::*;
pub use suggestions::*;

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "File.size is a whole, non-negative byte count"
)]
fn picked(list: Option<web_sys::FileList>) -> Vec<AttachmentFile> {
    let Some(list) = list else { return Vec::new() };
    (0..list.length())
        .filter_map(|i| list.get(i))
        .map(|f| AttachmentFile { name: f.name(), size: f.size() as u64, mime: f.type_() })
        .collect()
}

/// A chat input that queues messages while a reply runs. Enter sends (or queues), Shift+Enter
/// adds a line, Ctrl or Cmd+Enter steers, Escape stops, and Up recalls earlier messages.
/// `@` lists `mentions`, and `/` at the start lists `commands`.
///
/// The owner runs the model: start a reply in `on_send`, set `running`, and clear it when the
/// reply ends. Queued messages are then sent one at a time through `on_send`.
#[component]
pub fn AssistantComposer(
    /// A reply is running: sends are queued and the stop button shows.
    #[prop(optional, into)]
    running: Signal<bool>,
    /// Called with the trimmed message to send now.
    #[prop(optional, into)]
    on_send: Option<Callback<String>>,
    /// Called with a message that should interrupt the running reply.
    #[prop(optional, into)]
    on_steer: Option<Callback<String>>,
    /// Called when the person stops the running reply.
    #[prop(optional, into)]
    on_cancel: Option<Callback<()>>,
    /// Called with the name, size and type of files picked or dropped.
    #[prop(optional, into)]
    on_add_files: Option<Callback<Vec<AttachmentFile>>>,
    /// Files attached to the next message.
    #[prop(optional, into)]
    attachments: Signal<Vec<AttachmentItem>>,
    /// Called with the attachment id when its remove button is pressed.
    #[prop(optional, into)]
    on_remove_attachment: Option<Callback<String>>,
    /// Quoted text shown above the field.
    #[prop(optional, into)]
    quote: Signal<Option<String>>,
    /// Called when the quote is dismissed.
    #[prop(optional, into)]
    on_quote_dismiss: Option<Callback<()>>,
    /// Names offered after `@`.
    #[prop(optional, into)]
    mentions: Signal<Vec<String>>,
    /// Commands offered after a leading `/`.
    #[prop(optional, into)]
    commands: Signal<Vec<String>>,
    /// The `accept` filter of the file picker, such as `image/*,.pdf`.
    #[prop(optional, into)]
    accept: Option<String>,
    /// Allow picking several files at once.
    #[prop(default = true)]
    multiple: bool,
    /// Placeholder text.
    #[prop(default = "Message the assistant".into(), into)]
    placeholder: String,
    /// Accessible name for the text field.
    #[prop(default = "Message".into(), into)]
    label: String,
    /// Tighter padding and a single row.
    #[prop(optional)]
    compact: bool,
) -> impl IntoView {
    let root_id = crate::id::next_id(h::SCOPE);
    let list_id = format!("{root_id}-popover");
    let input = Memo::new(move |_| AssistantComposerInput {
        running: running.get(),
        has_attachments: attachments.with(|a| !a.is_empty()),
    });
    let state = RwSignal::new(AssistantComposerMachine::initial(&input.get_untracked()));
    let area = NodeRef::<Textarea>::new();
    let picker = NodeRef::<Input>::new();
    let dragging = RwSignal::new(false);

    let send = move |event: AssistantComposerEvent| {
        for command in drive::<AssistantComposerMachine, _>(state, input, event) {
            match command {
                AssistantComposerCommand::Send(text) => {
                    if let Some(cb) = on_send {
                        cb.run(text);
                    }
                }
                AssistantComposerCommand::Steer(text) => match (on_steer, on_send) {
                    (Some(cb), _) | (None, Some(cb)) => cb.run(text),
                    (None, None) => {}
                },
                AssistantComposerCommand::Cancel => {
                    if let Some(cb) = on_cancel {
                        cb.run(());
                    }
                }
                AssistantComposerCommand::Queued(_) => {}
            }
        }
    };

    // Push the machine's text and caret into the textarea after a programmatic change.
    let place_caret = move || {
        let Some(el) = area.get_untracked() else { return };
        let (text, caret) = state.with_untracked(|s| (s.text.clone(), s.caret));
        el.set_value(&text);
        let at = u32::try_from(h::char_to_utf16(&text, caret)).unwrap_or(u32::MAX);
        let _ = el.set_selection_range(at, at);
    };

    // The reply ended: send the next queued message.
    let was_running = StoredValue::new(running.get_untracked());
    Effect::new(move |_| {
        let now = running.get();
        let was = was_running.get_value();
        was_running.set_value(now);
        if was && !now {
            send(AssistantComposerEvent::RunFinished);
        }
    });

    let rows = Memo::new(move |_| {
        state.with(|s| match &s.trigger {
            Some(t) => {
                let list = if t.char == '/' { commands } else { mentions };
                list.with(|all| h::trigger_matches(all, &t.query).into_iter().cloned().collect::<Vec<_>>())
            }
            None => Vec::new(),
        })
    });
    let numbered = move || -> Vec<(usize, String)> { rows.get().into_iter().enumerate().collect() };
    let open = move || !rows.with(Vec::is_empty);
    let trigger_char = move || state.with(|s| s.trigger.as_ref().map_or('@', |t| t.char));
    let can_send = move || state.with(|s| !s.text.trim().is_empty()) || input.with(|i| i.has_attachments);

    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        let accept = rows.with(|r| state.with(|s| r.get(s.active).cloned()));
        let press = KeyPress {
            key: ev.key(),
            shift: ev.shift_key(),
            ctrl: ev.ctrl_key(),
            meta: ev.meta_key(),
            ..KeyPress::default()
        };
        let popover =
            Popover { rows: rows.with(Vec::len), accept: accept.as_deref(), running: running.get_untracked() };
        let action = state.with_untracked(|s| h::input_key(&press, ev.is_composing(), s, &popover));
        if let KeyAction::Event(e) = action {
            ev.prevent_default();
            let moves_text = matches!(
                e,
                AssistantComposerEvent::AcceptTrigger(_)
                    | AssistantComposerEvent::HistoryPrev
                    | AssistantComposerEvent::HistoryNext
            );
            send(e);
            if moves_text {
                place_caret();
            }
        }
    };
    let read_input = move || {
        let Some(el) = area.get_untracked() else { return };
        let text = el.value();
        let units = el.selection_start().ok().flatten().map_or(usize::MAX, |n| n as usize);
        let caret = h::utf16_to_char(&text, units);
        send(AssistantComposerEvent::SetText { text, caret });
    };
    let add_files = move |files: Vec<AttachmentFile>| {
        if let (false, Some(cb)) = (files.is_empty(), on_add_files) {
            cb.run(files);
        }
    };

    let pop_id = list_id.clone();
    let ctl_id = list_id;
    let opt_root = root_id.clone();
    let act_root = root_id.clone();

    view! {
        <div
            id=root_id
            data-scope=h::SCOPE
            data-part="root"
            data-running=move || running.get().then_some("")
            data-compact=compact.then_some("")
            data-dragging=move || dragging.get().then_some("")
            on:dragover=move |ev| {
                if on_add_files.is_some() {
                    ev.prevent_default();
                    dragging.set(true);
                }
            }
            on:dragleave=move |_| dragging.set(false)
            on:drop=move |ev| {
                if on_add_files.is_some() {
                    ev.prevent_default();
                    dragging.set(false);
                    add_files(picked(ev.data_transfer().and_then(|dt| dt.files())));
                }
            }
        >
            {move || {
                quote
                    .get()
                    .map(|text| {
                        view! {
                            <div data-scope=h::SCOPE data-part="quote">
                                <span data-scope=h::SCOPE data-part="quote-text">{text}</span>
                                <button
                                    type="button"
                                    data-scope=h::SCOPE
                                    data-part="quote-dismiss"
                                    aria-label="Remove quote"
                                    title="Remove quote"
                                    on:click=move |_| {
                                        if let Some(cb) = on_quote_dismiss {
                                            cb.run(());
                                        }
                                    }
                                >
                                    <Icon name=IconName::X size=14 />
                                </button>
                            </div>
                        }
                    })
            }}
            <Show when=move || attachments.with(|a| !a.is_empty())>
                <div data-scope=h::SCOPE data-part="attachments" role="list" aria-label="Attachments">
                    <For
                        each=move || attachments.get()
                        key=|a| (a.id.clone(), h::attachment_meta(a))
                        children=move |item| {
                            match on_remove_attachment {
                                Some(cb) => Either::Left(view! { <AssistantAttachment item=item on_remove=cb /> }),
                                None => Either::Right(view! { <AssistantAttachment item=item /> }),
                            }
                        }
                    />
                </div>
            </Show>
            <div data-scope=h::SCOPE data-part="field">
                <textarea
                    node_ref=area
                    data-scope=h::SCOPE
                    data-part="input"
                    rows="1"
                    role="combobox"
                    aria-label=label
                    aria-autocomplete="list"
                    aria-haspopup="listbox"
                    aria-expanded=move || open().to_string()
                    aria-controls=ctl_id
                    aria-activedescendant=move || {
                        open().then(|| h::option_id(&opt_root, state.with(|s| s.active)))
                    }
                    placeholder=placeholder
                    prop:value=move || state.with(|s| s.text.clone())
                    on:input=move |_| read_input()
                    on:keydown=on_keydown
                    on:keyup=move |ev| {
                        // Moving the caret with the arrow keys can enter or leave a trigger.
                        if matches!(ev.key().as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End") {
                            read_input();
                        }
                    }
                    on:click=move |_| read_input()
                ></textarea>
            </div>
            <Show when=open>
                <ul id=pop_id.clone() role="listbox" aria-label="Suggestions" data-scope=h::SCOPE data-part="popover">
                    <For
                        each=numbered
                        key=|(i, label)| (*i, label.clone())
                        children={
                            let root = act_root.clone();
                            move |(i, name): (usize, String)| {
                                let pick = name.clone();
                                view! {
                                    <li
                                        id=h::option_id(&root, i)
                                        role="option"
                                        data-scope=h::SCOPE
                                        data-part="option"
                                        aria-selected=move || (state.with(|s| s.active) == i).to_string()
                                        data-highlighted=move || (state.with(|s| s.active) == i).then_some("")
                                        on:mousedown=|ev| ev.prevent_default()
                                        on:click=move |_| {
                                            send(AssistantComposerEvent::AcceptTrigger(pick.clone()));
                                            place_caret();
                                            if let Some(el) = area.get_untracked() {
                                                let _ = el.focus();
                                            }
                                        }
                                    >
                                        <span data-part="option-mark" aria-hidden="true">{trigger_char}</span>
                                        {name}
                                    </li>
                                }
                            }
                        }
                    />
                </ul>
            </Show>
            <Show when=move || state.with(|s| !s.queue.is_empty())>
                <ul data-scope=h::SCOPE data-part="queue" aria-label="Queued messages">
                    <For
                        each=move || state.with(|s| s.queue.clone())
                        key=|q| q.id
                        children=move |q| {
                            let id = q.id;
                            view! {
                                <AssistantQueueItem
                                    text=q.text
                                    on_steer=move |()| send(AssistantComposerEvent::SteerQueued(id))
                                    on_remove=move |()| send(AssistantComposerEvent::RemoveQueued(id))
                                />
                            }
                        }
                    />
                </ul>
            </Show>
            <div data-scope=h::SCOPE data-part="actions">
                {on_add_files
                    .map(|_| {
                        view! {
                            <label data-scope=h::SCOPE data-part="add-attachment" title="Add files">
                                <input
                                    node_ref=picker
                                    type="file"
                                    accept=accept
                                    multiple=multiple
                                    aria-label="Add files"
                                    data-scope=h::SCOPE
                                    data-part="file-input"
                                    on:change=move |_| {
                                        let Some(el) = picker.get_untracked() else { return };
                                        add_files(picked(el.files()));
                                        // Let the same file be picked again after a remove.
                                        el.set_value("");
                                    }
                                />
                                <Icon name=IconName::Plus size=16 />
                            </label>
                        }
                    })}
                <span data-scope=h::SCOPE data-part="spacer"></span>
                <Show when=move || running.get()>
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="cancel"
                        aria-label="Stop"
                        title="Stop (Escape)"
                        on:click=move |_| send(AssistantComposerEvent::Cancel)
                    >
                        <span data-part="stop-mark" aria-hidden="true"></span>
                    </button>
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="steer"
                        title="Send now and interrupt (Ctrl+Enter)"
                        disabled=move || !can_send()
                        on:click=move |_| send(AssistantComposerEvent::Steer)
                    >
                        "Steer"
                    </button>
                </Show>
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="send"
                    aria-label=move || h::send_label(running.get())
                    title=move || h::send_label(running.get())
                    disabled=move || !can_send()
                    on:click=move |_| send(AssistantComposerEvent::Send)
                >
                    {move || {
                        if running.get() {
                            Either::Left("Queue")
                        } else {
                            Either::Right(view! { <Icon name=IconName::ArrowUp size=16 /> })
                        }
                    }}
                </button>
            </div>
        </div>
    }
}
