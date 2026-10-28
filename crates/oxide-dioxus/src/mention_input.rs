//! `MentionInput`: a text field where `@`, `/` or any configured character opens a
//! filtered list, and a picked row becomes an atomic mention. The field is a native
//! `<textarea>`; a mirror behind it draws the mention pills and anchors the list at the
//! trigger. Edits and caret moves arrive through the shared [`h::WATCH_JS`] watcher.

use dioxus::prelude::*;
use oxide_headless::mention_input::{
    self as h, KeyPress, MentionInputCommand, MentionInputEvent, MentionInputMachine, MentionInputState, Report,
    Segment,
};
use oxide_statechart::Machine;

use crate::js;

pub use oxide_headless::mention_input::{Mention, MentionItem, MentionSource, MentionValue, TriggerSpec};

/// Write `text` into the textarea and select `start..end` (char indices).
fn place(input_id: &str, text: &str, start: usize, end: usize) {
    let (a, b) = (h::char_to_utf16(text, start).to_string(), h::char_to_utf16(text, end).to_string());
    js::run(
        &[("id", input_id), ("text", text), ("a", a.as_str()), ("b", b.as_str())],
        "const el = document.getElementById(id); if (el) { el.value = text; el.setSelectionRange(+a, +b); }",
    );
}

/// A text field with `@` mentions and `/` commands (or any triggers in `sources`). Typing a
/// trigger at the start of a word lists that source's items, filtered as you type; Up and
/// Down move, Enter or Tab inserts, Escape closes. An inserted mention is one token: it is
/// deleted whole and the caret steps over it. `on_change` gets the text plus every
/// mention's id and range.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn MentionInput(
    /// Visible label.
    label: String,
    /// Triggers and the items each one lists.
    sources: ReadSignal<Vec<MentionSource>>,
    /// Initial text and mentions.
    default_value: Option<MentionValue>,
    /// Called with the new value after every change.
    on_change: Option<EventHandler<MentionValue>>,
    /// Placeholder text.
    placeholder: Option<String>,
    /// Visible text rows.
    #[props(default = 3)]
    rows: u32,
    /// Greyed out and not editable.
    #[props(default)]
    disabled: bool,
) -> Element {
    let root = use_hook(|| oxide_headless::next_id(h::SCOPE));
    let ids = h::ids(&root);
    let input_id = use_hook(|| CopyValue::new(ids.input.clone()));
    let mut state = use_signal(move || default_value.map(MentionInputState::from_value).unwrap_or_default());

    let mut send = move |event: MentionInputEvent| {
        let step = MentionInputMachine::transition(&state.peek(), event, &h::machine_input(&sources.peek()));
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                MentionInputCommand::Changed(value) => {
                    if let Some(handler) = on_change {
                        handler.call(value);
                    }
                }
                MentionInputCommand::PlaceCaret { start, end } => {
                    let text = state.peek().text.clone();
                    place(&input_id.read(), &text, start, end);
                }
            }
        }
    };

    let watch = js::use_watch(h::WATCH_JS, root.clone(), move |report| match h::parse_report(&report) {
        Some(Report::Input { text, caret }) => send(MentionInputEvent::SetText { text, caret }),
        // An edit still on its way arrives as `input`; only pure caret moves count here.
        Some(Report::Select { text, start, end }) if state.peek().text == text => {
            send(MentionInputEvent::Select { start, end });
        }
        _ => {}
    });

    // Keep the highlighted row in view.
    use_effect({
        let root = root.clone();
        move || {
            let s = state.read();
            if s.is_open() {
                crate::combobox::scroll_into_list(&h::option_id(&root, s.active));
            }
        }
    });

    let onkeydown = move |e: KeyboardEvent| {
        let key = e.key().to_string();
        let press = KeyPress { key: &key, shift: e.modifiers().shift(), composing: e.is_composing() };
        let event = {
            let s = state.peek();
            let rows = h::rows(&s, &sources.peek());
            h::input_key(&press, &s, rows.len(), rows.get(s.active))
        };
        if let Some(ev) = event {
            e.prevent_default();
            send(ev);
        }
    };

    let s = state.read();
    let list = h::rows(&s, &sources.read());
    let open = s.is_open();
    let active = s.active;
    let mark = s.trigger.as_ref().map(|t| t.char.to_string()).unwrap_or_default();
    let list_name = h::open_source(&s, &sources.read()).map(|o| o.name.clone());
    let segments = h::segments(&s);
    let status = h::status_text(&s, list.len());
    let text = s.text.clone();
    drop(s);
    let active_id = (open && !list.is_empty()).then(|| h::option_id(&root, active));
    let empty = list.is_empty();

    rsx! {
        div {
            id: "{root}",
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-open": open.then_some(""),
            "data-disabled": disabled.then_some(""),
            onmounted: move |_| {
                let _ = watch.send("start");
            },
            label { id: "{ids.label}", r#for: "{ids.input}", "data-scope": h::SCOPE, "data-part": "label", "{label}" }
            div { "data-scope": h::SCOPE, "data-part": "field",
                div { id: "{ids.mirror}", "data-scope": h::SCOPE, "data-part": "mirror", "aria-hidden": "true",
                    for seg in segments {
                        {
                            match seg {
                                Segment::Text(t) => rsx! { "{t}" },
                                Segment::Mention(t) => rsx! {
                                    span { "data-scope": h::SCOPE, "data-part": "mention", "{t}" }
                                },
                                Segment::Anchor => rsx! {
                                    span { id: "{ids.anchor}", "data-scope": h::SCOPE, "data-part": "anchor" }
                                },
                            }
                        }
                    }
                }
                textarea {
                    id: "{ids.input}",
                    "data-scope": h::SCOPE,
                    "data-part": "input",
                    rows: "{rows}",
                    role: "combobox",
                    "aria-autocomplete": "list",
                    "aria-haspopup": "listbox",
                    "aria-expanded": "{open}",
                    "aria-controls": "{ids.listbox}",
                    "aria-activedescendant": active_id,
                    placeholder,
                    disabled,
                    value: "{text}",
                    onkeydown,
                    onblur: move |_| {
                        if state.peek().is_open() {
                            send(MentionInputEvent::Dismiss);
                        }
                    },
                }
                if open {
                    div { "data-scope": h::SCOPE, "data-part": "popover",
                        ul {
                            id: "{ids.listbox}",
                            role: "listbox",
                            "aria-label": list_name,
                            "data-scope": h::SCOPE,
                            "data-part": "listbox",
                            for (i, item) in list.into_iter().enumerate() {
                                li {
                                    key: "{i}-{item.id}",
                                    id: h::option_id(&root, i),
                                    role: "option",
                                    "data-scope": h::SCOPE,
                                    "data-part": "option",
                                    "aria-selected": "{active == i}",
                                    "data-highlighted": (active == i).then_some(""),
                                    onmousedown: |e| e.prevent_default(),
                                    onclick: {
                                        let (id, label) = (item.id.clone(), item.label.clone());
                                        move |_| send(MentionInputEvent::Accept { id: id.clone(), label: label.clone() })
                                    },
                                    span { "data-scope": h::SCOPE, "data-part": "option-mark", "aria-hidden": "true", "{mark}" }
                                    span { "data-scope": h::SCOPE, "data-part": "option-label", "{item.label}" }
                                    if !item.description.is_empty() {
                                        span { "data-scope": h::SCOPE, "data-part": "option-description", "{item.description}" }
                                    }
                                }
                            }
                        }
                        if empty {
                            p { "data-scope": h::SCOPE, "data-part": "empty", {h::EMPTY_TEXT} }
                        }
                    }
                }
            }
            span { id: "{ids.status}", role: "status", "data-scope": h::SCOPE, "data-part": "status", "{status}" }
        }
    }
}
