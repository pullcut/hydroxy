//! `<InlineEdit>`: text that reads as plain text and becomes a field on click
//! or Enter. Enter or blur commits, Esc cancels. `multiline` swaps in a
//! textarea, where Ctrl or ⌘ + Enter commits.

use leptos::either::Either;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use oxide_headless::inline_edit::{
    self as h, InlineEditCommand, InlineEditEvent, InlineEditInput, InlineEditMachine, InlineEditState,
};
use oxide_statechart::Machine;

use crate::focus::focus_id;

#[derive(Clone, Copy)]
struct Ctx {
    state: RwSignal<InlineEditState>,
    input: Memo<InlineEditInput>,
    ids: StoredValue<h::Ids>,
    on_change: Option<Callback<String>>,
}

impl Ctx {
    fn send(self, event: InlineEditEvent) {
        let Some(step) = self
            .input
            .try_with_untracked(|i| self.state.try_with_untracked(|s| InlineEditMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                InlineEditCommand::Change(text) => {
                    if let Some(cb) = self.on_change {
                        cb.run(text);
                    }
                }
                // The element is hidden until the state change reaches the DOM.
                InlineEditCommand::FocusField => {
                    let id = self.ids.get_value().field;
                    set_timeout(move || focus_and_select(&id), std::time::Duration::ZERO);
                }
                InlineEditCommand::FocusPreview => {
                    let id = self.ids.get_value().preview;
                    set_timeout(move || focus_id(&id), std::time::Duration::ZERO);
                }
            }
        }
    }
}

fn focus_and_select(id: &str) {
    if let Some(el) = document().get_element_by_id(id) {
        // `select()` exists on both inputs and textareas.
        let el = el.unchecked_into::<web_sys::HtmlInputElement>();
        let _ = web_sys::HtmlElement::focus(&el);
        el.select();
    }
}

/// Click-to-edit text.
#[component]
pub fn InlineEdit(
    /// What the text is, such as "Project name". Names the button and the field.
    #[prop(into)]
    label: String,
    /// Controlled value. Pair with `on_change`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial value when uncontrolled.
    #[prop(optional, into)]
    default_value: String,
    /// Called with the new text when an edit is committed and changed something.
    #[prop(optional, into)]
    on_change: Option<Callback<String>>,
    /// Shown, dimmed, while the value is empty.
    #[prop(optional, into)]
    placeholder: Option<String>,
    /// Edit in a textarea. Enter adds a line; Ctrl or ⌘ + Enter commits.
    #[prop(optional)]
    multiline: bool,
    /// Block editing.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
) -> impl IntoView {
    let input = Memo::new(move |_| InlineEditInput {
        value: value.get(),
        default_value: default_value.clone(),
        disabled: disabled.get().unwrap_or(false),
    });
    let state = RwSignal::new(input.with_untracked(InlineEditMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| InlineEditMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ids = h::ids(&crate::id::next_id(h::SCOPE));
    let ctx = Ctx { state, input, ids: StoredValue::new(ids.clone()), on_change };
    let editing = move || state.with(|s| s.editing);
    let off = move || input.with(|i| i.disabled);
    let placeholder = StoredValue::new(placeholder.unwrap_or_default());
    let text = move || state.with(|s| s.value.clone());

    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        let modifier = ev.ctrl_key() || ev.meta_key();
        if let Some(event) = h::field_keydown(&ev.key(), multiline, modifier) {
            ev.prevent_default();
            ctx.send(event);
        }
    };
    let on_input = move |text: String| ctx.send(InlineEditEvent::Input(text));
    let draft = move || state.with(|s| s.draft.clone());

    let preview_name = h::preview_label(&label);
    let field = if multiline {
        Either::Left(view! {
            <textarea
                data-scope=h::SCOPE
                data-part="field"
                id=ids.field.clone()
                aria-label=label
                rows="3"
                hidden=move || !editing()
                prop:value=draft
                on:input=move |ev| on_input(event_target_value(&ev))
                on:keydown=on_keydown
                on:blur=move |_| ctx.send(InlineEditEvent::Blur)
            ></textarea>
        })
    } else {
        Either::Right(view! {
            <input
                data-scope=h::SCOPE
                data-part="field"
                id=ids.field.clone()
                type="text"
                aria-label=label
                hidden=move || !editing()
                prop:value=draft
                on:input=move |ev| on_input(event_target_value(&ev))
                on:keydown=on_keydown
                on:blur=move |_| ctx.send(InlineEditEvent::Blur)
            />
        })
    };

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-multiline=multiline.then_some("")
            data-editing=move || editing().then_some("")
            data-disabled=move || off().then_some("")
        >
            <button
                type="button"
                data-scope=h::SCOPE
                data-part="preview"
                id=ids.preview.clone()
                aria-label=preview_name
                hidden=editing
                disabled=off
                data-empty=move || text().is_empty().then_some("")
                on:click=move |_| ctx.send(InlineEditEvent::Edit)
                on:keydown=move |ev| {
                    if let Some(event) = h::preview_keydown(&ev.key()) {
                        ev.prevent_default();
                        ctx.send(event);
                    }
                }
            >
                {move || {
                    let t = text();
                    if t.is_empty() { placeholder.get_value() } else { t }
                }}
            </button>
            {field}
        </div>
    }
}
