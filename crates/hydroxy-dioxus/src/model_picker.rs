//! `ModelPicker`: a compact trigger showing the current model and effort,
//! opening a panel with the host's models (a radio group: arrows move and
//! select), a temperature slider, a token budget and a reasoning-effort control.
//! Changing model shrinks the budget to what the new model can produce.

use dioxus::prelude::*;
use hydroxy_headless::model_picker::{self as h, RadioGroupCommand, RadioGroupEvent, RadioGroupMachine, RadioItem};
use hydroxy_headless::radio_group as rg;
use hydroxy_statechart::Machine;

use crate::button::ButtonVariant;
use crate::emoji_picker::focus_by_id;
use crate::icon::{Icon, IconName};
use crate::number_field::NumberField;
use crate::popover::{Align, Placement, Popover, PopoverContent, PopoverTrigger, Side};
use crate::segmented::{Segmented, SegmentedItem, SegmentedSize};
use crate::slider::Slider;

pub use hydroxy_headless::model_picker::{Effort, ModelOption, ModelSettings};

/// Pick a model and how it runs.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn ModelPicker(
    /// The models to offer.
    models: ReadSignal<Vec<ModelOption>>,
    /// Controlled settings. Pair with `on_change`.
    #[props(default)]
    value: ReadSignal<Option<ModelSettings>>,
    /// Initial settings when uncontrolled. An empty model id picks the first model.
    #[props(default)]
    default_value: ModelSettings,
    /// Called with the whole new settings after any change.
    on_change: Option<EventHandler<ModelSettings>>,
) -> Element {
    let mut local = use_signal(|| default_value.clone());
    let current = use_memo(move || {
        let mut s = value().unwrap_or_else(&*local);
        if s.model.is_empty() {
            s.model = models.read().first().map(|m| m.id.clone()).unwrap_or_default();
        }
        s
    });
    let mut commit = move |next: ModelSettings| {
        local.set(next.clone());
        if let Some(handler) = on_change {
            handler.call(next);
        }
    };
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));

    // The model list is a radio group over the model ids.
    let radio_input = use_memo(move || h::radio_input(&models.read(), &current.read().model));
    let mut radio = use_signal(|| RadioGroupMachine::initial(&radio_input.peek()));
    use_effect(move || {
        let synced = RadioGroupMachine::sync(&radio.peek(), &radio_input.read());
        if let Some(synced) = synced {
            radio.set(synced);
        }
    });
    // A CopyValue keeps `send` Copy, so every option's handlers can hold it.
    let send_root = use_hook(|| CopyValue::new(root.clone()));
    let mut send = move |event: RadioGroupEvent| {
        let step = RadioGroupMachine::transition(&radio.peek(), event, &radio_input.peek());
        if let Some(next) = step.state {
            radio.set(next);
        }
        for command in step.commands {
            match command {
                RadioGroupCommand::FocusItem(id) => focus_by_id(&h::model_id(&send_root.peek(), &id)),
                RadioGroupCommand::ValueChange { value, .. } => {
                    let next = h::choose_model(&current.peek(), &models.peek(), &value);
                    commit(next);
                }
            }
        }
    };

    let s = current();
    let list = models.read().clone();
    let selected = list.iter().find(|m| m.id == s.model).cloned();
    let max = selected.as_ref().map(|m| f64::from(m.max_output.max(h::MIN_TOKENS)));
    let effort_items: Vec<RadioItem> = Effort::ALL.iter().map(|e| RadioItem::new(e.as_str())).collect();
    let radio_state = radio.read().clone();
    let radio_in = radio_input.read().clone();

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root",
            Popover { placement: Placement::new(Side::Bottom, Align::Start),
                PopoverTrigger { variant: ButtonVariant::Ghost,
                    span { "data-scope": h::SCOPE, "data-part": "trigger", title: h::trigger_label(&s, &list),
                        span { "data-scope": h::SCOPE, "data-part": "trigger-model", {h::trigger_text(&s, &list)} }
                        span { "data-scope": h::SCOPE, "data-part": "trigger-effort", {s.effort.label()} }
                        Icon { name: IconName::ChevronDown, size: 14 }
                    }
                }
                PopoverContent {
                    div { "data-scope": h::SCOPE, "data-part": "panel",
                        div { "data-scope": h::SCOPE, "data-part": "models", role: "radiogroup", "aria-label": "Model",
                            for m in list.clone() {
                                {
                                    let a = rg::item_attrs(&root, &radio_state, &radio_in, &m.id);
                                    let (click, focus) = (m.id.clone(), m.id.clone());
                                    rsx! {
                                        button {
                                            key: "{m.id}",
                                            r#type: "button",
                                            role: "radio",
                                            id: h::model_id(&root, &m.id),
                                            "aria-checked": a.aria_checked.to_string(),
                                            "aria-labelledby": rg::label_id(&root, &m.id),
                                            "aria-describedby": rg::description_id(&root, &m.id),
                                            tabindex: a.tabindex,
                                            "data-scope": h::SCOPE,
                                            "data-part": "model",
                                            "data-state": a.data_state,
                                            onclick: move |_| send(RadioGroupEvent::Select(click.clone(), rg::ChangeReason::Pointer)),
                                            onfocus: move |_| send(RadioGroupEvent::Focus(focus.clone())),
                                            onkeydown: move |e| {
                                                if let Some(event) = rg::keydown(&e.key().to_string()) {
                                                    e.prevent_default();
                                                    send(event);
                                                }
                                            },
                                            span { "data-scope": h::SCOPE, "data-part": "model-head",
                                                span { "data-scope": h::SCOPE, "data-part": "name", id: rg::label_id(&root, &m.id), "{m.name}" }
                                                for t in m.tags.clone() {
                                                    span { "data-scope": h::SCOPE, "data-part": "tag", "{t}" }
                                                }
                                                span { "data-scope": h::SCOPE, "data-part": "check", "aria-hidden": "true",
                                                    Icon { name: IconName::Check, size: 14 }
                                                }
                                            }
                                            span { "data-scope": h::SCOPE, "data-part": "details", id: rg::description_id(&root, &m.id),
                                                span { "data-scope": h::SCOPE, "data-part": "description", "{m.description}" }
                                                span { "data-scope": h::SCOPE, "data-part": "meta", {h::meta(&m)} }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { "data-scope": h::SCOPE, "data-part": "settings",
                            Slider {
                                label: "Temperature",
                                min: 0.0,
                                max: 1.0,
                                step: 0.05,
                                value: Some(s.temperature),
                                format: Callback::new(h::temperature_text),
                                on_input: move |t: f64| {
                                    let next = ModelSettings { temperature: h::clamp_temperature(t), ..current.peek().clone() };
                                    commit(next);
                                },
                            }
                            // Keyed per model, since the limit is fixed at mount.
                            NumberField {
                                key: "{s.model}",
                                label: "Max tokens",
                                min: f64::from(h::MIN_TOKENS),
                                max,
                                step: 256.0,
                                large_step: 4_096.0,
                                value: Some(f64::from(s.max_tokens)),
                                on_value_change: move |v: Option<f64>| {
                                    let Some(v) = v else { return };
                                    // Clamped to u32 range first, so the cast cannot truncate.
                                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                                    let tokens = v.clamp(0.0, f64::from(u32::MAX)).round() as u32;
                                    let model = models.peek().iter().find(|m| m.id == current.peek().model).cloned();
                                    let next = ModelSettings {
                                        max_tokens: h::clamp_tokens(tokens, model.as_ref()),
                                        ..current.peek().clone()
                                    };
                                    commit(next);
                                },
                            }
                            div { "data-scope": h::SCOPE, "data-part": "effort",
                                span { "data-scope": h::SCOPE, "data-part": "effort-label", "aria-hidden": "true", "Reasoning effort" }
                                Segmented {
                                    label: "Reasoning effort",
                                    items: effort_items,
                                    size: SegmentedSize::Sm,
                                    value: Some(s.effort.as_str().to_string()),
                                    on_value_change: move |v: String| {
                                        if let Some(effort) = Effort::parse(&v) {
                                            commit(ModelSettings { effort, ..current.peek().clone() });
                                        }
                                    },
                                    for e in Effort::ALL {
                                        SegmentedItem { key: "{e.as_str()}", value: e.as_str(), label: e.label() }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
