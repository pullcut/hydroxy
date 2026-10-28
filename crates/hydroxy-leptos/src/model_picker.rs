//! `<ModelPicker>`: a compact trigger showing the current model and effort,
//! opening a panel with the host's models (a radio group: arrows move and
//! select), a temperature slider, a token budget and a reasoning-effort control.
//! Changing model shrinks the budget to what the new model can produce.

use leptos::prelude::*;
use hydroxy_headless::model_picker::{self as h, RadioGroupCommand, RadioGroupEvent, RadioGroupMachine, RadioItem};
use hydroxy_headless::radio_group as rg;
use hydroxy_statechart::Machine;

use crate::button::ButtonVariant;
use crate::focus::focus_id;
use crate::icon::{Icon, IconName};
use crate::number_field::NumberField;
use crate::popover::{Align, Placement, Popover, PopoverContent, PopoverTrigger, Side};
use crate::segmented::{Segmented, SegmentedItem, SegmentedSize};
use crate::slider::Slider;

pub use hydroxy_headless::model_picker::{Effort, ModelOption, ModelSettings};

/// Pick a model and how it runs.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn ModelPicker(
    /// The models to offer.
    #[prop(into)]
    models: Signal<Vec<ModelOption>>,
    /// Controlled settings. Pair with `on_change`.
    #[prop(optional, into)]
    value: MaybeProp<ModelSettings>,
    /// Initial settings when uncontrolled. An empty model id picks the first model.
    #[prop(optional)]
    default_value: ModelSettings,
    /// Called with the whole new settings after any change.
    #[prop(optional, into)]
    on_change: Option<Callback<ModelSettings>>,
) -> impl IntoView {
    let local = RwSignal::new(default_value);
    let current = Memo::new(move |_| {
        let mut s = value.get().unwrap_or_else(|| local.get());
        if s.model.is_empty() {
            s.model = models.with(|m| m.first().map(|m| m.id.clone()).unwrap_or_default());
        }
        s
    });
    let commit = move |next: ModelSettings| {
        local.set(next.clone());
        if let Some(cb) = on_change {
            cb.run(next);
        }
    };
    let root = crate::id::next_id(h::SCOPE);
    let root_s = StoredValue::new(root.clone());

    // The model list is a radio group over the model ids.
    let radio_input = Memo::new(move |_| models.with(|m| current.with(|s| h::radio_input(m, &s.model))));
    let radio = RwSignal::new(radio_input.with_untracked(RadioGroupMachine::initial));
    Effect::new(move |_| {
        let synced = radio_input.with(|i| radio.with_untracked(|s| RadioGroupMachine::sync(s, i)));
        if let Some(synced) = synced {
            radio.set(synced);
        }
    });
    let send = move |event: RadioGroupEvent| {
        let Some(step) =
            radio_input.try_with_untracked(|i| radio.with_untracked(|s| RadioGroupMachine::transition(s, event, i)))
        else {
            return;
        };
        if let Some(next) = step.state {
            radio.set(next);
        }
        for command in step.commands {
            match command {
                RadioGroupCommand::FocusItem(id) => focus_id(&h::model_id(&root_s.get_value(), &id)),
                RadioGroupCommand::ValueChange { value, .. } => {
                    let next = models.with_untracked(|m| current.with_untracked(|s| h::choose_model(s, m, &value)));
                    commit(next);
                }
            }
        }
    };

    let trigger_label = move || models.with(|m| current.with(|s| h::trigger_label(s, m)));
    let model_name = move || models.with(|m| current.with(|s| h::trigger_text(s, m)));
    let effort = move || current.with(|s| s.effort.label());
    let selected = Memo::new(move |_| models.with(|m| current.with(|s| m.iter().find(|x| x.id == s.model).cloned())));
    let effort_items = Signal::derive(|| Effort::ALL.iter().map(|e| RadioItem::new(e.as_str())).collect::<Vec<_>>());

    let model_rows = move || {
        let root = root_s.get_value();
        models
            .get()
            .into_iter()
            .map(|m| {
                let id = m.id.clone();
                let attrs = Memo::new({
                    let id = id.clone();
                    let root = root.clone();
                    move |_| radio_input.with(|i| radio.with(|s| rg::item_attrs(&root, s, i, &id)))
                });
                let (click, focus) = (id.clone(), id.clone());
                view! {
                    <button
                        type="button"
                        role="radio"
                        id=h::model_id(&root, &id)
                        aria-checked=move || attrs.get().aria_checked.to_string()
                        aria-labelledby=rg::label_id(&root, &id)
                        aria-describedby=rg::description_id(&root, &id)
                        tabindex=move || attrs.get().tabindex
                        data-scope=h::SCOPE
                        data-part="model"
                        data-state=move || attrs.get().data_state
                        on:click=move |_| send(RadioGroupEvent::Select(click.clone(), rg::ChangeReason::Pointer))
                        on:focus=move |_| send(RadioGroupEvent::Focus(focus.clone()))
                        on:keydown=move |ev| {
                            if let Some(event) = rg::keydown(&ev.key()) {
                                ev.prevent_default();
                                send(event);
                            }
                        }
                    >
                        <span data-scope=h::SCOPE data-part="model-head">
                            <span data-scope=h::SCOPE data-part="name" id=rg::label_id(&root, &id)>{m.name.clone()}</span>
                            {m.tags.iter().map(|t| view! { <span data-scope=h::SCOPE data-part="tag">{t.clone()}</span> }).collect_view()}
                            <span data-scope=h::SCOPE data-part="check" aria-hidden="true"><Icon name=IconName::Check size=14 /></span>
                        </span>
                        <span data-scope=h::SCOPE data-part="details" id=rg::description_id(&root, &id)>
                            <span data-scope=h::SCOPE data-part="description">{m.description.clone()}</span>
                            <span data-scope=h::SCOPE data-part="meta">{h::meta(&m)}</span>
                        </span>
                    </button>
                }
            })
            .collect_view()
    };

    view! {
        <div data-scope=h::SCOPE data-part="root">
            <Popover placement=Placement::new(Side::Bottom, Align::Start)>
                <PopoverTrigger variant=ButtonVariant::Ghost>
                    <span data-scope=h::SCOPE data-part="trigger" title=trigger_label>
                        <span data-scope=h::SCOPE data-part="trigger-model">{model_name}</span>
                        <span data-scope=h::SCOPE data-part="trigger-effort">{effort}</span>
                        <Icon name=IconName::ChevronDown size=14 />
                    </span>
                </PopoverTrigger>
                <PopoverContent>
                    <div data-scope=h::SCOPE data-part="panel">
                        <div data-scope=h::SCOPE data-part="models" role="radiogroup" aria-label="Model">
                            {model_rows}
                        </div>
                        <div data-scope=h::SCOPE data-part="settings">
                            <Slider
                                label="Temperature"
                                min=0.0
                                max=1.0
                                step=0.05
                                value=Signal::derive(move || Some(current.with(|s| s.temperature)))
                                format=Callback::new(h::temperature_text)
                                on_input=move |t: f64| {
                                    let next = current.with_untracked(|s| ModelSettings { temperature: h::clamp_temperature(t), ..s.clone() });
                                    commit(next);
                                }
                            />
                            // Re-made per model, since the limit is fixed at mount.
                            {move || {
                                let model = selected.get();
                                let max = model.as_ref().map_or(f64::MAX, |m| f64::from(m.max_output.max(h::MIN_TOKENS)));
                                let on_tokens = move |v: Option<f64>| {
                                    let Some(v) = v else { return };
                                    // Clamped to u32 range first, so the cast cannot truncate.
                                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                                    let tokens = v.clamp(0.0, f64::from(u32::MAX)).round() as u32;
                                    let next = current.with_untracked(|s| ModelSettings {
                                        max_tokens: h::clamp_tokens(tokens, model.as_ref()),
                                        ..s.clone()
                                    });
                                    commit(next);
                                };
                                view! {
                                    <NumberField
                                        label="Max tokens"
                                        min=f64::from(h::MIN_TOKENS)
                                        max=max
                                        step=256.0
                                        large_step=4_096.0
                                        value=Signal::derive(move || Some(f64::from(current.with(|s| s.max_tokens))))
                                        on_value_change=on_tokens
                                    />
                                }
                            }}
                            <div data-scope=h::SCOPE data-part="effort">
                                <span data-scope=h::SCOPE data-part="effort-label" aria-hidden="true">"Reasoning effort"</span>
                                <Segmented
                                    label="Reasoning effort"
                                    items=effort_items
                                    size=SegmentedSize::Sm
                                    value=Signal::derive(move || Some(current.with(|s| s.effort.as_str().to_string())))
                                    on_value_change=move |v: String| {
                                        if let Some(effort) = Effort::parse(&v) {
                                            commit(current.with_untracked(|s| ModelSettings { effort, ..s.clone() }));
                                        }
                                    }
                                >
                                    {Effort::ALL.into_iter().map(|e| view! { <SegmentedItem value=e.as_str() label=e.label() /> }).collect_view()}
                                </Segmented>
                            </div>
                        </div>
                    </div>
                </PopoverContent>
            </Popover>
        </div>
    }
}
