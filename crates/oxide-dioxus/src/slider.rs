//! `Slider`: a native `<input type="range">` with a label row and a value
//! readout. The native element gives keyboard support, `aria-valuenow` and form
//! participation; oxide only styles it.

use dioxus::prelude::*;

/// The `data-scope` value on every slider part.
pub const SCOPE: &str = "slider";

/// How far `value` is from `min` to `max`, as a percentage for the filled track.
fn fill_percent(value: f64, min: f64, max: f64) -> f64 {
    if max > min { ((value - min) / (max - min) * 100.0).clamp(0.0, 100.0) } else { 0.0 }
}

/// A labelled range control.
#[component]
pub fn Slider(
    /// Visible label. Also the accessible name.
    label: String,
    /// Lowest value.
    #[props(default = 0.0)]
    min: f64,
    /// Highest value.
    #[props(default = 100.0)]
    max: f64,
    /// Step between values.
    #[props(default = 1.0)]
    step: f64,
    /// Controlled value. Pair with `on_input`.
    #[props(default)]
    value: ReadSignal<Option<f64>>,
    /// Initial value when uncontrolled. Defaults to `min`.
    default_value: Option<f64>,
    /// Called with the new value as the thumb moves.
    on_input: Option<EventHandler<f64>>,
    /// Turns the value into the readout text, such as `"£1,200"`. Defaults to the number.
    format: Option<Callback<f64, String>>,
    /// Block changes.
    #[props(default)]
    disabled: bool,
    /// Form field name.
    name: Option<String>,
) -> Element {
    let id = use_hook(|| oxide_headless::next_id(SCOPE));
    // Uncontrolled value lives here so the readout and fill can follow the thumb.
    let mut local = use_signal(|| default_value.unwrap_or(min));
    let current = value().unwrap_or(local());
    let text = format.map_or_else(|| current.to_string(), |f| f.call(current));
    let valuetext = format.is_some().then(|| text.clone());

    rsx! {
        div { "data-scope": SCOPE, "data-part": "root", "data-disabled": disabled.then_some(""),
            div { "data-scope": SCOPE, "data-part": "header",
                label { "data-scope": SCOPE, "data-part": "label", r#for: id.clone(), {label} }
                span { "data-scope": SCOPE, "data-part": "value", "aria-hidden": "true", {text} }
            }
            input {
                "data-scope": SCOPE,
                "data-part": "control",
                r#type: "range",
                id,
                min,
                max,
                step,
                name,
                disabled,
                "aria-valuetext": valuetext,
                value: "{current}",
                style: "--ox-slider-fill: {fill_percent(current, min, max)}%;",
                oninput: move |e| {
                    let v = e.value().parse().unwrap_or(min);
                    local.set(v);
                    if let Some(handler) = on_input {
                        handler.call(v);
                    }
                },
            }
        }
    }
}
