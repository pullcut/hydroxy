//! `NumberField`: a labelled number input with step buttons.

use dioxus::prelude::*;
use hydroxy_headless::number_field::{
    self as h, NumberFieldCommand, NumberFieldEvent, NumberFieldInput, NumberFieldMachine,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use hydroxy_headless::Direction;

/// A number you can type or step. Typing commits on Enter or blur, clamped to
/// `min`..`max`; arrows, PageUp/PageDown and the buttons step at once.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn NumberField(
    /// Visible label.
    label: String,
    /// Controlled value. Pair with `on_value_change`. Must be `Some` at start to control.
    #[props(default)]
    value: ReadSignal<Option<f64>>,
    /// Initial value when uncontrolled. Starts empty without one.
    default_value: Option<f64>,
    /// Lowest value.
    min: Option<f64>,
    /// Highest value.
    max: Option<f64>,
    /// Step for arrows and buttons.
    #[props(default = 1.0)]
    step: f64,
    /// Step for PageUp/PageDown. Default ten steps.
    large_step: Option<f64>,
    /// Fixed decimal places, such as 2 for money.
    fraction_digits: Option<u8>,
    /// Block changes and drop out of the tab order.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Focusable but not editable.
    #[props(default)]
    read_only: bool,
    /// Form field name. Submits the committed number.
    name: Option<String>,
    /// Placeholder while empty.
    placeholder: Option<String>,
    /// Text before the value inside the field, such as `£`.
    prefix: Option<String>,
    /// Text after the value inside the field, such as `%`.
    suffix: Option<String>,
    /// Called with the new value whenever it commits or a change is requested. `None` is empty.
    on_value_change: Option<EventHandler<Option<f64>>>,
) -> Element {
    let ids = use_hook(|| h::ids(&hydroxy_headless::next_id(h::SCOPE)));
    let input = use_memo(move || NumberFieldInput {
        value: value(),
        default_value,
        min,
        max,
        step,
        large_step,
        fraction_digits,
        disabled: disabled(),
        read_only,
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
            if let Some(handler) = on_value_change {
                handler.call(v);
            }
        }
    };

    let s = state.read();
    let i = input.read();
    let a = h::control_attrs(&s, &i);
    let (can_down, can_up) = (h::can_step(&s, &i, Direction::Prev), h::can_step(&s, &i, Direction::Next));
    let is_disabled = i.disabled;

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-disabled": is_disabled.then_some(""),
            "data-readonly": read_only.then_some(""),
            label { "data-scope": h::SCOPE, "data-part": "label", r#for: "{ids.control}", "{label}" }
            div { "data-scope": h::SCOPE, "data-part": "field",
                if let Some(text) = prefix {
                    span { "data-scope": h::SCOPE, "data-part": "affix", "{text}" }
                }
                input {
                    r#type: "text",
                    inputmode: "decimal",
                    autocomplete: "off",
                    id: "{ids.control}",
                    "data-scope": h::SCOPE,
                    "data-part": "control",
                    role: a.role,
                    "aria-valuenow": a.aria_valuenow,
                    "aria-valuemin": a.aria_valuemin,
                    "aria-valuemax": a.aria_valuemax,
                    placeholder,
                    disabled: is_disabled,
                    readonly: read_only,
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
                if let Some(text) = suffix {
                    span { "data-scope": h::SCOPE, "data-part": "affix", "{text}" }
                }
                div { "data-scope": h::SCOPE, "data-part": "steppers",
                    button {
                        r#type: "button",
                        tabindex: "-1",
                        "data-scope": h::SCOPE,
                        "data-part": "decrement",
                        "aria-label": "Decrease",
                        "aria-controls": "{ids.control}",
                        disabled: !can_down,
                        onclick: move |_| send(NumberFieldEvent::Step { direction: Direction::Prev, large: false }),
                        svg { view_box: "0 0 16 16", width: "14", height: "14", "aria-hidden": "true",
                            path { d: "M3.5 8h9" }
                        }
                    }
                    button {
                        r#type: "button",
                        tabindex: "-1",
                        "data-scope": h::SCOPE,
                        "data-part": "increment",
                        "aria-label": "Increase",
                        "aria-controls": "{ids.control}",
                        disabled: !can_up,
                        onclick: move |_| send(NumberFieldEvent::Step { direction: Direction::Next, large: false }),
                        svg { view_box: "0 0 16 16", width: "14", height: "14", "aria-hidden": "true",
                            path { d: "M3.5 8h9M8 3.5v9" }
                        }
                    }
                }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name: "{name}", disabled: is_disabled, value: h::form_value(&s) }
            }
        }
    }
}
