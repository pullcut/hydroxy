//! `TagsInput`: removable tags plus a draft input, in one field.

use dioxus::prelude::*;
use oxide_headless::tags_input::{
    self as h, TagsInputCommand, TagsInputEvent, TagsInputInput, TagsInputMachine, TagsInputState,
};
use oxide_statechart::Machine;

use crate::focus::FocusMap;
use crate::icon::{Icon, IconName};

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<TagsInputState>,
    input: Memo<TagsInputInput>,
    live: Signal<String>,
    on_value_change: Option<EventHandler<Vec<String>>>,
}

impl Ctx {
    fn send(mut self, event: TagsInputEvent) {
        let step = TagsInputMachine::transition(&self.state.peek(), event, &self.input.peek());
        self.state.set(step.state);
        if let Some(text) = h::last_announcement(&step.commands) {
            self.live.set(text);
        }
        for command in step.commands {
            if let (TagsInputCommand::ValueChange(tags), Some(handler)) = (command, self.on_value_change) {
                handler.call(tags);
            }
        }
    }
}

/// A list of short values typed into one field. Enter or a comma adds the draft;
/// Backspace on an empty draft highlights, then removes, the last tag.
#[component]
pub fn TagsInput(
    /// Visible label.
    label: String,
    /// Controlled tags. Pair with `on_value_change`.
    #[props(default)]
    value: ReadSignal<Option<Vec<String>>>,
    /// Initial tags when uncontrolled.
    #[props(default)]
    default_value: Vec<String>,
    /// Most tags allowed. Shows a "3 of 5" counter.
    max: Option<usize>,
    /// Block edits.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Form field name. Each tag submits as its own value.
    name: Option<String>,
    /// Placeholder for the draft input.
    placeholder: Option<String>,
    /// Called with the new tags whenever they change or a change is requested.
    on_value_change: Option<EventHandler<Vec<String>>>,
) -> Element {
    let ids = use_hook(|| h::ids(&oxide_headless::next_id(h::SCOPE)));
    let input = use_memo(move || TagsInputInput {
        value: value(),
        default_value: default_value.clone(),
        max,
        disabled: disabled(),
    });
    let mut state = use_signal(|| TagsInputMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = TagsInputMachine::sync(&state.peek(), &input.read());
        state.set(synced);
    });
    let live = use_signal(String::new);
    let mut pasting = use_signal(|| false);
    let focus = use_hook(FocusMap::new);
    let ctx = Ctx { state, input, live, on_value_change };

    let current = state.read().clone();
    let is_disabled = disabled();
    let described_by = max.map(|_| ids.counter.clone());

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-disabled": is_disabled.then_some(""),
            label { "data-scope": h::SCOPE, "data-part": "label", r#for: "{ids.control}", "{label}" }
            div {
                "data-scope": h::SCOPE,
                "data-part": "field",
                onclick: move |_| focus.focus("control"),
                for (i, tag) in current.value.iter().cloned().enumerate() {
                    span {
                        key: "{i}-{tag}",
                        "data-scope": h::SCOPE,
                        "data-part": "tag",
                        "data-highlighted": (current.highlighted == Some(i)).then_some(""),
                        "{tag}"
                        button {
                            r#type: "button",
                            tabindex: "-1",
                            "data-scope": h::SCOPE,
                            "data-part": "remove",
                            "aria-label": h::remove_label(&tag),
                            disabled: is_disabled,
                            onclick: move |e| {
                                e.stop_propagation();
                                ctx.send(TagsInputEvent::Remove(i));
                            },
                            Icon { name: IconName::X, size: 11 }
                        }
                    }
                }
                input {
                    r#type: "text",
                    id: "{ids.control}",
                    "data-scope": h::SCOPE,
                    "data-part": "control",
                    placeholder,
                    disabled: is_disabled,
                    "aria-describedby": described_by,
                    value: "{current.draft}",
                    onmounted: move |e| focus.register("control".to_string(), e.data()),
                    onpaste: move |_| pasting.set(true),
                    oninput: move |e| {
                        let pasted = std::mem::take(&mut *pasting.write());
                        ctx.send(TagsInputEvent::SetDraft { text: e.value(), pasted });
                    },
                    onkeydown: move |e| {
                        let event = h::keydown(&e.key().to_string(), &ctx.state.peek());
                        if let Some(event) = event {
                            e.prevent_default();
                            ctx.send(event);
                        }
                    },
                    onblur: move |_| ctx.send(TagsInputEvent::Blur),
                }
            }
            if let Some(text) = h::counter(current.value.len(), max) {
                span { "data-scope": h::SCOPE, "data-part": "counter", id: "{ids.counter}", "{text}" }
            }
            if let Some(name) = name {
                for tag in current.value.iter().cloned() {
                    input { r#type: "hidden", name: "{name}", value: "{tag}" }
                }
            }
            span { "data-scope": h::SCOPE, "data-part": "live", "aria-live": "polite", "{live}" }
        }
    }
}
