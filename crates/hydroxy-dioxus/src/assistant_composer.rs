//! The assistant composer family: [`AssistantComposer`] (text, queue, history, `@` and `/`
//! popover, attachments), [`AssistantQueueItem`], [`AssistantAttachment`],
//! [`AssistantSuggestion`] and the [`AssistantSuggestions`] grid.

mod attachment;
mod queue;
mod suggestions;

pub use attachment::*;
pub use queue::*;
pub use suggestions::*;

use dioxus::html::FileData;
use dioxus::prelude::*;
use hydroxy_headless::assistant_composer::{
    self as h, AssistantComposerCommand, AssistantComposerEvent, AssistantComposerInput, AssistantComposerMachine,
    KeyAction, KeyPress, Popover,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::icon::{Icon, IconName};

pub use hydroxy_headless::assistant_composer::{AttachmentFile, AttachmentItem, AttachmentStatus};
pub use hydroxy_headless::assistant_thread::Suggestion;

fn picked(files: Vec<FileData>) -> Vec<AttachmentFile> {
    files
        .into_iter()
        .map(|f| AttachmentFile { name: f.name(), size: f.size(), mime: f.content_type().unwrap_or_default() })
        .collect()
}

/// A chat input that queues messages while a reply runs. Enter sends (or queues), Shift+Enter
/// adds a line, Ctrl or Cmd+Enter steers, Escape stops, and Up recalls earlier messages.
/// `@` lists `mentions`, and `/` at the start lists `commands`.
///
/// The owner runs the model: start a reply in `on_send`, set `running`, and clear it when the
/// reply ends. Queued messages are then sent one at a time through `on_send`.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn AssistantComposer(
    /// A reply is running: sends are queued and the stop button shows.
    #[props(default)]
    running: ReadSignal<bool>,
    /// Called with the trimmed message to send now.
    on_send: Option<EventHandler<String>>,
    /// Called with a message that should interrupt the running reply.
    on_steer: Option<EventHandler<String>>,
    /// Called when the person stops the running reply.
    on_cancel: Option<EventHandler<()>>,
    /// Called with the name, size and type of files picked or dropped.
    on_add_files: Option<EventHandler<Vec<AttachmentFile>>>,
    /// Files attached to the next message.
    #[props(default)]
    attachments: ReadSignal<Vec<AttachmentItem>>,
    /// Called with the attachment id when its remove button is pressed.
    on_remove_attachment: Option<EventHandler<String>>,
    /// Quoted text shown above the field.
    #[props(default)]
    quote: ReadSignal<Option<String>>,
    /// Called when the quote is dismissed.
    on_quote_dismiss: Option<EventHandler<()>>,
    /// Names offered after `@`.
    #[props(default)]
    mentions: ReadSignal<Vec<String>>,
    /// Commands offered after a leading `/`.
    #[props(default)]
    commands: ReadSignal<Vec<String>>,
    /// The `accept` filter of the file picker, such as `image/*,.pdf`.
    accept: Option<String>,
    /// Allow picking several files at once.
    #[props(default = true)]
    multiple: bool,
    /// Placeholder text.
    #[props(default = "Message the assistant".to_string())]
    placeholder: String,
    /// Accessible name for the text field.
    #[props(default = "Message".to_string())]
    label: String,
    /// Tighter padding and a single row.
    #[props(default)]
    compact: bool,
) -> Element {
    let root_id = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let field_id = format!("{root_id}-input");
    let list_id = format!("{root_id}-popover");
    let input = use_memo(move || AssistantComposerInput {
        running: running(),
        has_attachments: !attachments.read().is_empty(),
    });
    let state = use_signal(|| AssistantComposerMachine::initial(&AssistantComposerInput::default()));
    let mut dragging = use_signal(|| false);

    let send = move |event: AssistantComposerEvent| {
        let commands = drive::<AssistantComposerMachine>(state, input, event);
        for command in commands {
            match command {
                AssistantComposerCommand::Send(text) => {
                    if let Some(handler) = on_send {
                        handler.call(text);
                    }
                }
                AssistantComposerCommand::Steer(text) => {
                    if let Some(handler) = on_steer.or(on_send) {
                        handler.call(text);
                    }
                }
                AssistantComposerCommand::Cancel => {
                    if let Some(handler) = on_cancel {
                        handler.call(());
                    }
                }
                AssistantComposerCommand::Queued(_) => {}
            }
        }
    };

    // Push the machine's text and caret into the textarea after a programmatic change.
    let place_caret = {
        let field_id = field_id.clone();
        move || {
            let (text, caret) = {
                let s = state.peek();
                (s.text.clone(), s.caret)
            };
            let at = h::char_to_utf16(&text, caret);
            // `{id:?}` quotes the id as a JS string. Ids from `next_id` are plain ASCII.
            let _ = document::eval(&format!(
                "const el = document.getElementById({field_id:?});
                if (el) {{ el.value = {text:?}; el.focus(); el.setSelectionRange({at}, {at}); }}"
            ));
        }
    };

    // The reply ended: send the next queued message.
    let mut was_running = use_signal(|| *running.peek());
    use_effect(move || {
        let now = running();
        let was = *was_running.peek();
        was_running.set(now);
        if was && !now {
            send(AssistantComposerEvent::RunFinished);
        }
    });

    let s = state.read();
    let rows: Vec<String> = match &s.trigger {
        Some(t) => {
            let list = if t.char == '/' { commands.read() } else { mentions.read() };
            h::trigger_matches(&list, &t.query).into_iter().cloned().collect()
        }
        None => Vec::new(),
    };
    let open = !rows.is_empty();
    let active = s.active;
    let trigger_char = s.trigger.as_ref().map_or('@', |t| t.char);
    let text = s.text.clone();
    let queue = s.queue.clone();
    let has_attachments = !attachments.read().is_empty();
    let can_send = !text.trim().is_empty() || has_attachments;
    let is_running = running();
    drop(s);

    let on_keydown = {
        let rows = rows.clone();
        let place_caret = place_caret.clone();
        move |e: KeyboardEvent| {
            let m = e.modifiers();
            let press = KeyPress {
                key: e.key().to_string(),
                shift: m.shift(),
                ctrl: m.ctrl(),
                meta: m.meta(),
                ..KeyPress::default()
            };
            let accept = rows.get(state.peek().active).map(String::as_str);
            let popover = Popover { rows: rows.len(), accept, running: *running.peek() };
            let action = h::input_key(&press, e.is_composing(), &state.peek(), &popover);
            if let KeyAction::Event(ev) = action {
                e.prevent_default();
                let moves_text = matches!(
                    ev,
                    AssistantComposerEvent::AcceptTrigger(_)
                        | AssistantComposerEvent::HistoryPrev
                        | AssistantComposerEvent::HistoryNext
                );
                send(ev);
                if moves_text {
                    place_caret();
                }
            }
        }
    };
    // Read the caret from the DOM, since the input event carries only the value.
    let read_caret = {
        let field_id = field_id.clone();
        move || {
            let field_id = field_id.clone();
            spawn(async move {
                let mut js = document::eval(&format!(
                    "const el = document.getElementById({field_id:?});
                    dioxus.send(el ? [el.selectionStart ?? 0, el.value] : [0, '']);"
                ));
                let Ok((units, value)) = js.recv::<(usize, String)>().await else { return };
                if state.peek().text == value {
                    let caret = h::utf16_to_char(&value, units);
                    send(AssistantComposerEvent::SetText { text: value, caret });
                }
            });
        }
    };
    let add_files = move |files: Vec<AttachmentFile>| {
        if let (false, Some(handler)) = (files.is_empty(), on_add_files) {
            handler.call(files);
        }
    };

    let (act_root, ctl_id) = (root_id.clone(), list_id.clone());
    let active_id = open.then(|| h::option_id(&root_id, active));

    rsx! {
        div {
            id: "{root_id}",
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-running": is_running.then_some(""),
            "data-compact": compact.then_some(""),
            "data-dragging": dragging().then_some(""),
            ondragover: move |e| {
                if on_add_files.is_some() {
                    e.prevent_default();
                    dragging.set(true);
                }
            },
            ondragleave: move |_| dragging.set(false),
            ondrop: move |e| {
                if on_add_files.is_some() {
                    e.prevent_default();
                    dragging.set(false);
                    add_files(picked(e.data_transfer().files()));
                }
            },
            if let Some(q) = quote() {
                div { "data-scope": h::SCOPE, "data-part": "quote",
                    span { "data-scope": h::SCOPE, "data-part": "quote-text", "{q}" }
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "quote-dismiss",
                        "aria-label": "Remove quote",
                        title: "Remove quote",
                        onclick: move |_| {
                            if let Some(handler) = on_quote_dismiss {
                                handler.call(());
                            }
                        },
                        Icon { name: IconName::X, size: 14 }
                    }
                }
            }
            if has_attachments {
                div { "data-scope": h::SCOPE, "data-part": "attachments", role: "list", "aria-label": "Attachments",
                    for item in attachments.read().iter().cloned() {
                        AssistantAttachment {
                            key: "{item.id}",
                            item: item.clone(),
                            on_remove: on_remove_attachment,
                        }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "field",
                textarea {
                    id: "{field_id}",
                    "data-scope": h::SCOPE,
                    "data-part": "input",
                    rows: "1",
                    role: "combobox",
                    "aria-label": "{label}",
                    "aria-autocomplete": "list",
                    "aria-haspopup": "listbox",
                    "aria-expanded": "{open}",
                    "aria-controls": "{ctl_id}",
                    "aria-activedescendant": active_id,
                    placeholder: "{placeholder}",
                    value: "{text}",
                    oninput: {
                        let read_caret = read_caret.clone();
                        move |e| {
                            let value = e.value();
                            let caret = value.chars().count();
                            send(AssistantComposerEvent::SetText { text: value, caret });
                            read_caret();
                        }
                    },
                    onkeydown: on_keydown,
                    onkeyup: {
                        let read_caret = read_caret.clone();
                        move |e| {
                            // Moving the caret with the arrow keys can enter or leave a trigger.
                            if matches!(e.key().to_string().as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End") {
                                read_caret();
                            }
                        }
                    },
                    onclick: {
                        let read_caret = read_caret.clone();
                        move |_| read_caret()
                    },
                }
            }
            if open {
                ul { id: "{list_id}", role: "listbox", "aria-label": "Suggestions", "data-scope": h::SCOPE, "data-part": "popover",
                    for (i, name) in rows.iter().cloned().enumerate() {
                        li {
                            key: "{i}-{name}",
                            id: h::option_id(&act_root, i),
                            role: "option",
                            "data-scope": h::SCOPE,
                            "data-part": "option",
                            "aria-selected": "{active == i}",
                            "data-highlighted": (active == i).then_some(""),
                            onmousedown: |e| e.prevent_default(),
                            onclick: {
                                let pick = name.clone();
                                let place_caret = place_caret.clone();
                                move |_| {
                                    send(AssistantComposerEvent::AcceptTrigger(pick.clone()));
                                    place_caret();
                                }
                            },
                            span { "data-part": "option-mark", "aria-hidden": "true", "{trigger_char}" }
                            "{name}"
                        }
                    }
                }
            }
            if !queue.is_empty() {
                ul { "data-scope": h::SCOPE, "data-part": "queue", "aria-label": "Queued messages",
                    for q in queue {
                        AssistantQueueItem {
                            key: "{q.id}",
                            text: q.text.clone(),
                            on_steer: move |()| send(AssistantComposerEvent::SteerQueued(q.id)),
                            on_remove: move |()| send(AssistantComposerEvent::RemoveQueued(q.id)),
                        }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "actions",
                if on_add_files.is_some() {
                    label { "data-scope": h::SCOPE, "data-part": "add-attachment", title: "Add files",
                        input {
                            r#type: "file",
                            accept,
                            multiple,
                            "aria-label": "Add files",
                            "data-scope": h::SCOPE,
                            "data-part": "file-input",
                            onchange: move |e| add_files(picked(e.files())),
                        }
                        Icon { name: IconName::Plus, size: 16 }
                    }
                }
                span { "data-scope": h::SCOPE, "data-part": "spacer" }
                if is_running {
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "cancel",
                        "aria-label": "Stop",
                        title: "Stop (Escape)",
                        onclick: move |_| send(AssistantComposerEvent::Cancel),
                        span { "data-part": "stop-mark", "aria-hidden": "true" }
                    }
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "steer",
                        title: "Send now and interrupt (Ctrl+Enter)",
                        disabled: !can_send,
                        onclick: move |_| send(AssistantComposerEvent::Steer),
                        "Steer"
                    }
                }
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "send",
                    "aria-label": h::send_label(is_running),
                    title: h::send_label(is_running),
                    disabled: !can_send,
                    onclick: move |_| send(AssistantComposerEvent::Send),
                    if is_running {
                        "Queue"
                    } else {
                        Icon { name: IconName::ArrowUp, size: 16 }
                    }
                }
            }
        }
    }
}
