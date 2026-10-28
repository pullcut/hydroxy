//! `<TagsInput>`: removable tags plus a draft input, in one field.

use hydroxy_headless::tags_input::{
    self as h, TagsInputCommand, TagsInputEvent, TagsInputInput, TagsInputMachine, TagsInputState,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::drive::drive;
use crate::focus::focus_id;
use crate::icon::{Icon, IconName};

/// A list of short values typed into one field. Enter or a comma adds the draft;
/// Backspace on an empty draft highlights, then removes, the last tag.
#[component]
pub fn TagsInput(
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Controlled tags. Pair with `on_value_change`.
    #[prop(optional, into)]
    value: MaybeProp<Vec<String>>,
    /// Initial tags when uncontrolled.
    #[prop(optional)]
    default_value: Vec<String>,
    /// Most tags allowed. Shows a "3 of 5" counter.
    #[prop(optional)]
    max: Option<usize>,
    /// Block edits.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Form field name. Each tag submits as its own value.
    #[prop(optional, into)]
    name: Option<String>,
    /// Placeholder for the draft input.
    #[prop(optional, into)]
    placeholder: Option<String>,
    /// Called with the new tags whenever they change or a change is requested.
    #[prop(optional, into)]
    on_value_change: Option<Callback<Vec<String>>>,
) -> impl IntoView {
    let ids = h::ids(&crate::id::next_id(h::SCOPE));
    let input = Memo::new(move |_| TagsInputInput {
        value: value.get(),
        default_value: default_value.clone(),
        max,
        disabled: disabled.get(),
    });
    let state = RwSignal::new(input.with_untracked(TagsInputMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| TagsInputMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let live = RwSignal::new(String::new());
    let pasting = StoredValue::new(false);

    let send = move |event: TagsInputEvent| {
        // Events can still arrive after unmount (blur on a removed element).
        let commands = drive::<TagsInputMachine, _>(state, input, event);
        if let Some(text) = h::last_announcement(&commands) {
            live.set(text);
        }
        for command in commands {
            if let (TagsInputCommand::ValueChange(tags), Some(cb)) = (command, on_value_change) {
                cb.run(tags);
            }
        }
    };

    let control = ids.control.clone();
    let hidden = name.map(|name| {
        move || {
            state.with(|s| {
                s.value
                    .iter()
                    .map(|tag| view! { <input type="hidden" name=name.clone() value=tag.clone() /> })
                    .collect_view()
            })
        }
    });
    let counter = max.map(|_| {
        let id = ids.counter.clone();
        view! {
            <span data-scope=h::SCOPE data-part="counter" id=id>
                {move || state.with(|s| h::counter(s.value.len(), max))}
            </span>
        }
    });

    view! {
        <div data-scope=h::SCOPE data-part="root" data-disabled=move || disabled.get().then_some("")>
            <label data-scope=h::SCOPE data-part="label" for=ids.control.clone()>
                {label}
            </label>
            <div data-scope=h::SCOPE data-part="field" on:click=move |_| focus_id(&control)>
                {move || tag_list(state, disabled, send)}
                <input
                    type="text"
                    id=ids.control.clone()
                    data-scope=h::SCOPE
                    data-part="control"
                    placeholder=placeholder
                    disabled=move || disabled.get()
                    aria-describedby=max.map(|_| ids.counter.clone())
                    prop:value=move || state.with(|s| s.draft.clone())
                    on:paste=move |_| pasting.set_value(true)
                    on:input=move |ev| {
                        let pasted = pasting.get_value();
                        pasting.set_value(false);
                        send(TagsInputEvent::SetDraft { text: event_target_value(&ev), pasted });
                    }
                    on:keydown=move |ev| {
                        if let Some(event) = state.try_with_untracked(|s| h::keydown(&ev.key(), s)).flatten() {
                            ev.prevent_default();
                            send(event);
                        }
                    }
                    on:blur=move |_| send(TagsInputEvent::Blur)
                />
            </div>
            {counter}
            {hidden}
            <span data-scope=h::SCOPE data-part="live" aria-live="polite">
                {move || live.get()}
            </span>
        </div>
    }
}

/// The tag chips, each with its remove button.
fn tag_list(
    state: RwSignal<TagsInputState>,
    disabled: Signal<bool>,
    send: impl Fn(TagsInputEvent) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    state.with(|s| {
        s.value
            .iter()
            .enumerate()
            .map(|(i, tag)| {
                view! {
                    <span data-scope=h::SCOPE data-part="tag" data-highlighted=(s.highlighted == Some(i)).then_some("")>
                        {tag.clone()}
                        <button
                            type="button"
                            tabindex="-1"
                            data-scope=h::SCOPE
                            data-part="remove"
                            aria-label=h::remove_label(tag)
                            disabled=disabled.get()
                            on:click=move |ev| {
                                ev.stop_propagation();
                                send(TagsInputEvent::Remove(i));
                            }
                        >
                            <Icon name=IconName::X size=11 />
                        </button>
                    </span>
                }
            })
            .collect_view()
    })
}
