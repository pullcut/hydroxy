//! `<RadioGroup>` / `<RadioGroupItem>`.
//! Items read the machine from context and render the connector's attributes.

use leptos::prelude::*;
use oxide_headless::radio_group::{
    self as h, ChangeReason, RadioGroupCommand, RadioGroupEvent, RadioGroupInput, RadioGroupMachine, RadioGroupState,
};
use oxide_statechart::Machine;

pub use oxide_headless::radio_group::{Orientation, RadioItem};

use crate::focus::focus_id;

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<RadioGroupState>,
    input: Memo<RadioGroupInput>,
    on_value_change: Option<Callback<String>>,
}

impl Ctx {
    fn send(self, event: RadioGroupEvent) {
        // Events can still arrive after unmount (blur or toggle on a removed element).
        let Some(step) = self
            .input
            .try_with_untracked(|input| {
                self.state.try_with_untracked(|state| RadioGroupMachine::transition(state, event, input))
            })
            .flatten()
        else {
            return;
        };
        self.state.set(step.state);
        for command in step.commands {
            match command {
                RadioGroupCommand::FocusItem(value) => focus_id(&h::item_id(&self.id.get_value(), &value)),
                RadioGroupCommand::ValueChange { value, .. } => {
                    if let Some(cb) = self.on_value_change {
                        cb.run(value);
                    }
                }
            }
        }
    }
}

fn use_ctx() -> Ctx {
    use_context::<Ctx>().expect("RadioGroupItem must be rendered inside <RadioGroup>")
}

/// Root. Owns the machine and provides it to the items below.
#[component]
pub fn RadioGroup(
    /// The options, in display order.
    #[prop(into)]
    items: Signal<Vec<RadioItem>>,
    /// Controlled value. Pair with `on_value_change`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial value when uncontrolled.
    #[prop(optional, into)]
    default_value: Option<String>,
    /// Disable the whole group.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Layout axis, vertical by default. Arrow keys work on both axes either way.
    #[prop(default = Orientation::Vertical)]
    orientation: Orientation,
    /// Wrap focus from the last option to the first and back.
    #[prop(optional, default = true)]
    loop_focus: bool,
    /// Form field name. When set, the selected value is submitted with the form.
    #[prop(optional, into)]
    name: Option<String>,
    /// Accessible name for the group (`aria-label`).
    #[prop(optional, into)]
    label: Option<String>,
    /// Called with the new value whenever the selection changes or is requested.
    #[prop(optional, into)]
    on_value_change: Option<Callback<String>>,
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| RadioGroupInput {
        items: items.get(),
        value: value.get(),
        default_value: default_value.clone(),
        disabled: disabled.get().unwrap_or(false),
        loop_focus,
    });
    let state = RwSignal::new(input.with_untracked(RadioGroupMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| RadioGroupMachine::sync(s, i)));
        state.set(synced);
    });

    provide_context(Ctx { id: StoredValue::new(crate::id::next_id(h::SCOPE)), state, input, on_value_change });

    let hidden = move || {
        let name = name.clone()?;
        let value = state.get().value?;
        Some(view! { <input type="hidden" name=name value=value /> })
    };

    view! {
        <div
            role="radiogroup"
            aria-label=label
            aria-orientation=orientation.as_str()
            aria-disabled=move || input.with(|i| i.disabled).then_some("true")
            data-scope=h::SCOPE
            data-part="root"
            data-orientation=orientation.as_str()
        >
            {children()}
            {hidden}
        </div>
    }
}

/// One option: the radio circle, its label and an optional description.
#[component]
pub fn RadioGroupItem(
    /// The option this item selects.
    #[prop(into)]
    value: String,
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Optional hint under the label.
    #[prop(optional, into)]
    description: Option<String>,
) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = {
        let value = value.clone();
        Memo::new(move |_| ctx.input.with(|i| ctx.state.with(|s| h::item_attrs(&ctx.id.get_value(), s, i, &value))))
    };
    let root = ctx.id.get_value();
    let label_id = h::label_id(&root, &value);
    let description = description.map(|text| {
        view! {
            <span id=h::description_id(&root, &value) data-scope=h::SCOPE data-part="description">
                {text}
            </span>
        }
    });
    let has_description = description.is_some();
    let (on_click, on_focus) = (value.clone(), value);
    view! {
        <button
            type="button"
            id=move || attrs.get().id
            role="radio"
            aria-checked=move || attrs.get().aria_checked.to_string()
            aria-labelledby=move || attrs.get().aria_labelledby
            aria-describedby=move || has_description.then(|| attrs.get().aria_describedby)
            tabindex=move || attrs.get().tabindex
            disabled=move || attrs.get().disabled
            data-scope=h::SCOPE
            data-part="item"
            data-state=move || attrs.get().data_state
            on:click=move |_| ctx.send(RadioGroupEvent::Select(on_click.clone(), ChangeReason::Pointer))
            on:focus=move |_| ctx.send(RadioGroupEvent::Focus(on_focus.clone()))
            on:keydown=move |ev| {
                if let Some(event) = h::keydown(&ev.key()) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            <span data-scope=h::SCOPE data-part="control" aria-hidden="true"></span>
            <span data-scope=h::SCOPE data-part="text">
                <span id=label_id data-scope=h::SCOPE data-part="label">
                    {label}
                </span>
                {description}
            </span>
        </button>
    }
}
