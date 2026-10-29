//! `QuantityStepper`: a compact − / value / + control for counts such as items
//! in a basket. It is the number field's machine in a tighter layout.

use dioxus::prelude::*;
use hydroxy_headless::quantity_stepper::{
    self as h, NumberFieldCommand, NumberFieldEvent, NumberFieldInput, NumberFieldMachine,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use hydroxy_headless::Direction;

/// A whole-number stepper. Type a number (commits on Enter or blur) or use the
/// buttons and arrow keys; the value stays within `min`..`max`.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn QuantityStepper(
    /// Accessible name, such as "Quantity of Oat milk". Not shown.
    label: String,
    /// Controlled value. Pair with `on_change`. Must be `Some` at start to control.
    #[props(default)]
    value: ReadSignal<Option<f64>>,
    /// Initial value when uncontrolled.
    #[props(default = 1.0)]
    default_value: f64,
    /// Lowest value.
    #[props(default = 0.0)]
    min: f64,
    /// Highest value.
    #[props(default)]
    max: Option<f64>,
    /// Amount per press.
    #[props(default = 1.0)]
    step: f64,
    /// Block changes.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Form field name. Submits the number.
    #[props(default)]
    name: Option<String>,
    /// Called with the new value whenever it commits or a change is requested.
    on_change: Option<EventHandler<f64>>,
) -> Element {
    let ids = use_hook(|| h::ids(&hydroxy_headless::next_id(h::SCOPE)));
    let input = use_memo(move || NumberFieldInput {
        value: value(),
        default_value: Some(default_value),
        min: Some(min),
        max,
        step,
        disabled: disabled(),
        ..NumberFieldInput::default()
    });
    let mut state = use_signal(|| NumberFieldMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = NumberFieldMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let send = move |event: NumberFieldEvent| {
        for NumberFieldCommand::ValueChange(v) in drive::<NumberFieldMachine>(state, input, event) {
            if let (Some(handler), Some(v)) = (on_change, v) {
                handler.call(v);
            }
        }
    };

    let s = state.read();
    let i = input.read();
    let a = h::control_attrs(&s, &i);
    let (can_down, can_up) = (h::can_step(&s, &i, Direction::Prev), h::can_step(&s, &i, Direction::Next));
    let form_value = h::form_value(&s);
    let is_disabled = i.disabled;
    let lower = label.to_lowercase();

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            role: "group",
            "aria-label": label.clone(),
            "data-disabled": is_disabled.then_some(""),
            button {
                r#type: "button",
                "data-scope": h::SCOPE,
                "data-part": "decrement",
                "aria-label": "Decrease {lower}",
                "aria-controls": "{ids.control}",
                disabled: !can_down,
                onclick: move |_| send(NumberFieldEvent::Step { direction: Direction::Prev, large: false }),
                svg { view_box: "0 0 16 16", width: "14", height: "14", "aria-hidden": "true",
                    path { d: "M3.5 8h9" }
                }
            }
            input {
                r#type: "text",
                inputmode: "numeric",
                autocomplete: "off",
                id: "{ids.control}",
                "data-scope": h::SCOPE,
                "data-part": "control",
                role: a.role,
                "aria-label": label,
                "aria-valuenow": a.aria_valuenow,
                "aria-valuemin": a.aria_valuemin,
                "aria-valuemax": a.aria_valuemax,
                disabled: is_disabled,
                value: "{s.draft}",
                oninput: move |e| send(NumberFieldEvent::SetText(e.value())),
                onkeydown: move |e| {
                    let event = h::keydown(&e.key().to_string(), &input.peek());
                    if let Some(event) = event {
                        e.prevent_default();
                        send(event);
                    }
                },
                onblur: move |_| send(NumberFieldEvent::Commit),
            }
            button {
                r#type: "button",
                "data-scope": h::SCOPE,
                "data-part": "increment",
                "aria-label": "Increase {lower}",
                "aria-controls": "{ids.control}",
                disabled: !can_up,
                onclick: move |_| send(NumberFieldEvent::Step { direction: Direction::Next, large: false }),
                svg { view_box: "0 0 16 16", width: "14", height: "14", "aria-hidden": "true",
                    path { d: "M3.5 8h9M8 3.5v9" }
                }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name, disabled: is_disabled, value: form_value }
            }
        }
    }
}
