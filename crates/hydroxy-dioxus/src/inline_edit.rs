//! `InlineEdit`: text that reads as plain text and becomes a field on click
//! or Enter. Enter or blur commits, Esc cancels. `multiline` swaps in a
//! textarea, where Ctrl or ⌘ + Enter commits.

use dioxus::prelude::*;
use hydroxy_headless::inline_edit::{
    self as h, InlineEditCommand, InlineEditEvent, InlineEditInput, InlineEditMachine, InlineEditState,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::focus::focus_and_select;

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<InlineEditState>,
    input: Memo<InlineEditInput>,
    /// Which element should take focus when it mounts: the field or the preview.
    pending: Signal<Option<Pending>>,
    on_change: Option<EventHandler<String>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pending {
    Field,
    Preview,
}

impl Ctx {
    fn send(mut self, event: InlineEditEvent) {
        for command in drive::<InlineEditMachine>(self.state, self.input, event) {
            match command {
                InlineEditCommand::Change(text) => {
                    if let Some(handler) = self.on_change {
                        handler.call(text);
                    }
                }
                InlineEditCommand::FocusField => self.pending.set(Some(Pending::Field)),
                InlineEditCommand::FocusPreview => self.pending.set(Some(Pending::Preview)),
            }
        }
    }

    /// Focus `el` if it is the element a command asked for.
    fn take_focus(mut self, which: Pending, el: std::rc::Rc<MountedData>, select_id: Option<String>) {
        if *self.pending.peek() != Some(which) {
            return;
        }
        self.pending.set(None);
        spawn(async move {
            let _ = el.set_focus(true).await;
            if let Some(id) = select_id {
                // Select the text so typing replaces it. Best-effort.
                focus_and_select(&id);
            }
        });
    }
}

/// Click-to-edit text.
#[component]
pub fn InlineEdit(
    /// What the text is, such as "Project name". Names the button and the field.
    label: String,
    /// Controlled value. Pair with `on_change`.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: String,
    /// Called with the new text when an edit is committed and changed something.
    on_change: Option<EventHandler<String>>,
    /// Shown, dimmed, while the value is empty.
    placeholder: Option<String>,
    /// Edit in a textarea. Enter adds a line; Ctrl or ⌘ + Enter commits.
    #[props(default)]
    multiline: bool,
    /// Block editing.
    #[props(default)]
    disabled: ReadSignal<bool>,
) -> Element {
    let input = use_memo(move || InlineEditInput {
        value: value(),
        default_value: default_value.clone(),
        disabled: disabled(),
    });
    let mut state = use_signal(|| InlineEditMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = InlineEditMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ids = use_hook(|| h::ids(&hydroxy_headless::next_id(h::SCOPE)));
    let ctx = use_hook(|| Ctx { state, input, pending: Signal::new(None), on_change });

    let s = state.read();
    let off = input.read().disabled;
    let field_id = ids.field.clone();
    let on_keydown = move |e: KeyboardEvent| {
        let m = e.modifiers();
        if let Some(event) = h::field_keydown(&e.key().to_string(), multiline, m.ctrl() || m.meta()) {
            e.prevent_default();
            ctx.send(event);
        }
    };

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-multiline": multiline.then_some(""),
            "data-editing": s.editing.then_some(""),
            "data-disabled": off.then_some(""),
            if s.editing {
                if multiline {
                    textarea {
                        "data-scope": h::SCOPE,
                        "data-part": "field",
                        id: ids.field.clone(),
                        "aria-label": label.clone(),
                        rows: "3",
                        value: s.draft.clone(),
                        onmounted: move |e| ctx.take_focus(Pending::Field, e.data(), Some(field_id.clone())),
                        oninput: move |e| ctx.send(InlineEditEvent::Input(e.value())),
                        onkeydown: on_keydown,
                        onblur: move |_| ctx.send(InlineEditEvent::Blur),
                    }
                } else {
                    input {
                        "data-scope": h::SCOPE,
                        "data-part": "field",
                        id: ids.field.clone(),
                        r#type: "text",
                        "aria-label": label.clone(),
                        value: s.draft.clone(),
                        onmounted: move |e| ctx.take_focus(Pending::Field, e.data(), Some(field_id.clone())),
                        oninput: move |e| ctx.send(InlineEditEvent::Input(e.value())),
                        onkeydown: on_keydown,
                        onblur: move |_| ctx.send(InlineEditEvent::Blur),
                    }
                }
            } else {
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "preview",
                    id: ids.preview.clone(),
                    "aria-label": h::preview_label(&label),
                    disabled: off,
                    "data-empty": s.value.value.is_empty().then_some(""),
                    onmounted: move |e| ctx.take_focus(Pending::Preview, e.data(), None),
                    onclick: move |_| ctx.send(InlineEditEvent::Edit),
                    onkeydown: move |e| {
                        if let Some(event) = h::preview_keydown(&e.key().to_string()) {
                            e.prevent_default();
                            ctx.send(event);
                        }
                    },
                    if s.value.value.is_empty() {
                        {placeholder.clone().unwrap_or_default()}
                    } else {
                        "{s.value.value}"
                    }
                }
            }
        }
    }
}
