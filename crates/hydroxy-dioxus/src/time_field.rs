//! `TimeField`: a styled native `<input type="time">`.
//!
//! Native over a custom segmented field: the browser already gives segmented
//! hour/minute/AM-PM entry with arrow-key spinning, the locale's 12- or 24-hour
//! clock, a wheel picker on phones, `min`/`max`/`step` validation and form
//! submission, all announced correctly by screen readers. hydroxy styles the
//! field and its segments. The cost: the hour cycle follows the user's locale,
//! not a prop.

use dioxus::prelude::*;
use hydroxy_headless::input as h;

/// The `data-scope` value on every time field part.
pub const SCOPE: &str = "time-field";

/// A labelled time of day, with an optional hint or error. Values are `"HH:MM"`
/// (or `"HH:MM:SS"` with a sub-minute `step`), 24-hour, whatever the display.
#[component]
pub fn TimeField(
    /// Visible label.
    label: String,
    /// Controlled value, such as `"09:30"`. Pair with `on_input`.
    value: Option<String>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: String,
    /// Earliest allowed time.
    min: Option<String>,
    /// Latest allowed time.
    max: Option<String>,
    /// Step in seconds. Default 60; below 60 shows a seconds segment.
    step: Option<u32>,
    /// Called with the new value as it is edited. Empty while incomplete.
    on_input: Option<EventHandler<String>>,
    /// Help text under the field. Hidden while there is an error.
    hint: Option<String>,
    /// Error text. Replaces the hint and marks the field invalid.
    error: Option<String>,
    /// Form field name.
    name: Option<String>,
    /// Block editing.
    #[props(default)]
    disabled: bool,
    /// Native `required`.
    #[props(default)]
    required: bool,
) -> Element {
    let ids = use_hook(|| h::ids(&hydroxy_headless::next_id(SCOPE)));
    let attrs = h::control_attrs(&ids, hint.is_some(), error.is_some());
    // Uncontrolled: the value never changes between renders, so typing is left alone.
    let current = value.unwrap_or(default_value);
    let invalid = error.is_some();

    rsx! {
        div {
            "data-scope": SCOPE,
            "data-part": "root",
            "data-invalid": invalid.then_some(""),
            "data-disabled": disabled.then_some(""),
            label { "data-scope": SCOPE, "data-part": "label", r#for: "{ids.control}", "{label}" }
            input {
                "data-scope": SCOPE,
                "data-part": "control",
                r#type: "time",
                id: "{ids.control}",
                min,
                max,
                step: step.map(|s| s.to_string()),
                name,
                required,
                disabled,
                "aria-describedby": attrs.aria_describedby,
                "aria-invalid": attrs.aria_invalid,
                value: current,
                oninput: move |e| {
                    if let Some(handler) = on_input {
                        handler.call(e.value());
                    }
                },
            }
            if let Some(e) = error {
                p { "data-scope": SCOPE, "data-part": "error", id: "{ids.error}", "{e}" }
            } else if let Some(t) = hint {
                p { "data-scope": SCOPE, "data-part": "hint", id: "{ids.hint}", "{t}" }
            }
        }
    }
}
