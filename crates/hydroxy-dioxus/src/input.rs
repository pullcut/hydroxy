//! `Input`: a labelled native text field with an optional hint or error.

use dioxus::prelude::*;
use hydroxy_headless::input as h;

pub use hydroxy_headless::input::Size;

/// Text field with a label, and a hint or an error below it.
#[component]
pub fn Input(
    /// Visible label.
    label: String,
    /// Help text under the field. Hidden while there is an error.
    hint: Option<String>,
    /// Error text. Replaces the hint and marks the field invalid.
    error: Option<String>,
    /// Controlled value. Pair with `on_input`.
    value: Option<String>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: String,
    /// Called with the new text on every keystroke.
    on_input: Option<EventHandler<String>>,
    /// The native `type`. Default `text`.
    #[props(default = "text".to_string())]
    input_type: String,
    /// Placeholder text.
    placeholder: Option<String>,
    /// Form field name.
    name: Option<String>,
    /// Block editing.
    #[props(default)]
    disabled: bool,
    /// Native `required`.
    #[props(default)]
    required: bool,
    /// Monospaced text, for money and rates.
    #[props(default)]
    mono: bool,
    /// Field height.
    #[props(default)]
    size: Size,
    /// Text before the value inside the field, such as `£`.
    prefix: Option<String>,
    /// Text after the value inside the field, such as `%`.
    suffix: Option<String>,
) -> Element {
    let ids = use_hook(|| h::ids(&hydroxy_headless::next_id(h::SCOPE)));
    let attrs = h::control_attrs(&ids, hint.is_some(), error.is_some());
    // Uncontrolled: the value never changes between renders, so typing is left alone.
    let current = value.unwrap_or(default_value);
    let invalid = error.is_some();

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-size": size.as_str(),
            "data-mono": mono.then_some(""),
            "data-invalid": invalid.then_some(""),
            "data-disabled": disabled.then_some(""),
            label { "data-scope": h::SCOPE, "data-part": "label", r#for: ids.control.clone(), {label} }
            div { "data-scope": h::SCOPE, "data-part": "field",
                if let Some(p) = prefix {
                    span { "data-scope": h::SCOPE, "data-part": "affix", {p} }
                }
                input {
                    "data-scope": h::SCOPE,
                    "data-part": "control",
                    id: ids.control,
                    r#type: input_type,
                    placeholder,
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
                if let Some(s) = suffix {
                    span { "data-scope": h::SCOPE, "data-part": "affix", {s} }
                }
            }
            if let Some(e) = error {
                p { "data-scope": h::SCOPE, "data-part": "error", id: ids.error.clone(), {e} }
            } else if let Some(t) = hint {
                p { "data-scope": h::SCOPE, "data-part": "hint", id: ids.hint.clone(), {t} }
            }
        }
    }
}
