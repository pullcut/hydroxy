//! `RadioGroup` / `RadioGroupItem`.
//! Items read the machine from context and render the connector's attributes.
//! Focus goes through `MountedData`, so it works off the web too.

use dioxus::prelude::*;
use oxide_headless::radio_group::{
    self as h, ChangeReason, RadioGroupCommand, RadioGroupEvent, RadioGroupInput, RadioGroupMachine, RadioGroupState,
};
use oxide_statechart::Machine;

pub use oxide_headless::radio_group::{Orientation, RadioItem};

use crate::focus::FocusMap;

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<RadioGroupState>,
    input: Memo<RadioGroupInput>,
    items: FocusMap,
    on_value_change: Option<EventHandler<String>>,
}

impl Ctx {
    fn send(mut self, event: RadioGroupEvent) {
        let step = RadioGroupMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                RadioGroupCommand::FocusItem(value) => self.items.focus(&value),
                RadioGroupCommand::ValueChange { value, .. } => {
                    if let Some(handler) = self.on_value_change {
                        handler.call(value);
                    }
                }
            }
        }
    }
}

fn use_ctx() -> Ctx {
    try_use_context::<Ctx>().expect("RadioGroupItem must be rendered inside RadioGroup")
}

/// Root. Owns the machine and provides it to the items below.
#[component]
pub fn RadioGroup(
    /// The options, in display order.
    items: ReadSignal<Vec<RadioItem>>,
    /// Controlled value. Pair with `on_value_change`.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: Option<String>,
    /// Disable the whole group.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Layout axis, vertical by default. Arrow keys work on both axes either way.
    #[props(default = Orientation::Vertical)]
    orientation: Orientation,
    /// Wrap focus from the last option to the first and back.
    #[props(default = true)]
    loop_focus: bool,
    /// Form field name. When set, the selected value is submitted with the form.
    #[props(default)]
    name: Option<String>,
    /// Accessible name for the group (`aria-label`).
    #[props(default)]
    label: Option<String>,
    /// Called with the new value whenever the selection changes or is requested.
    on_value_change: Option<EventHandler<String>>,
    children: Element,
) -> Element {
    let input = use_memo(move || RadioGroupInput {
        items: items(),
        value: value(),
        default_value: default_value.clone(),
        disabled: disabled(),
        loop_focus,
    });
    let mut state = use_signal(|| RadioGroupMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = RadioGroupMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    use_context_provider(|| Ctx {
        id: CopyValue::new(oxide_headless::next_id(h::SCOPE)),
        state,
        input,
        items: FocusMap::new(),
        on_value_change,
    });

    let selected = state.read().value.clone();
    rsx! {
        div {
            role: "radiogroup",
            "aria-label": label,
            "aria-orientation": orientation.as_str(),
            "aria-disabled": input.read().disabled.then_some("true"),
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-orientation": orientation.as_str(),
            {children}
            if let (Some(name), Some(value)) = (name, selected) {
                input { r#type: "hidden", name, value }
            }
        }
    }
}

/// One option: the radio circle, its label and an optional description.
#[component]
pub fn RadioGroupItem(
    /// The option this item selects.
    value: String,
    /// Visible label.
    label: String,
    /// Optional hint under the label.
    description: Option<String>,
) -> Element {
    let ctx = use_ctx();
    let root = ctx.id.read().clone();
    let a = h::item_attrs(&root, &ctx.state.read(), &ctx.input.read(), &value);
    let describedby = description.is_some().then_some(a.aria_describedby);
    let (mounted, clicked, focused) = (value.clone(), value.clone(), value.clone());
    rsx! {
        button {
            r#type: "button",
            id: a.id,
            role: a.role,
            "aria-checked": a.aria_checked.to_string(),
            "aria-labelledby": a.aria_labelledby,
            "aria-describedby": describedby,
            tabindex: a.tabindex,
            disabled: a.disabled,
            "data-scope": h::SCOPE,
            "data-part": "item",
            "data-state": a.data_state,
            onmounted: move |e| ctx.items.register(mounted.clone(), e.data()),
            onclick: move |_| ctx.send(RadioGroupEvent::Select(clicked.clone(), ChangeReason::Pointer)),
            onfocus: move |_| ctx.send(RadioGroupEvent::Focus(focused.clone())),
            onkeydown: move |e| {
                if let Some(event) = h::keydown(&e.key().to_string()) {
                    e.prevent_default();
                    ctx.send(event);
                }
            },
            span { "data-scope": h::SCOPE, "data-part": "control", "aria-hidden": "true" }
            span { "data-scope": h::SCOPE, "data-part": "text",
                span { id: h::label_id(&root, &value), "data-scope": h::SCOPE, "data-part": "label", {label} }
                if let Some(text) = description {
                    span {
                        id: h::description_id(&root, &value),
                        "data-scope": h::SCOPE,
                        "data-part": "description",
                        {text}
                    }
                }
            }
        }
    }
}
