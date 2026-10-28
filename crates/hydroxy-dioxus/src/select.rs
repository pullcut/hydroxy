//! `Select`: a native `<select>` styled to match `Input`, with a label and a
//! hint or error. The native element is the behaviour: keyboard, typeahead and
//! the platform picker all come free.

use dioxus::prelude::*;
use hydroxy_headless::input as h;

use crate::icon::{Icon, IconName};

pub use hydroxy_headless::select::{SCOPE, SelectOption};

/// A labelled dropdown of options.
#[component]
pub fn Select(
    /// Visible label.
    label: String,
    /// The choices, in display order.
    options: Vec<SelectOption>,
    /// Controlled value. Pair with `on_change`.
    value: Option<String>,
    /// Initial value when uncontrolled. Defaults to the first option.
    default_value: Option<String>,
    /// Called with the chosen value.
    on_change: Option<EventHandler<String>>,
    /// Help text under the field. Hidden while there is an error.
    hint: Option<String>,
    /// Error text. Replaces the hint and marks the field invalid.
    error: Option<String>,
    /// Block changes.
    #[props(default)]
    disabled: bool,
    /// Form field name.
    name: Option<String>,
    /// Native `required`.
    #[props(default)]
    required: bool,
) -> Element {
    let ids = use_hook(|| h::ids(&hydroxy_headless::next_id(SCOPE)));
    let attrs = h::control_attrs(&ids, hint.is_some(), error.is_some());
    // Uncontrolled: the selection never changes between renders, so the user's pick is left alone.
    let selected = value.or(default_value);
    let invalid = error.is_some();

    rsx! {
        div {
            "data-scope": SCOPE,
            "data-part": "root",
            "data-invalid": invalid.then_some(""),
            "data-disabled": disabled.then_some(""),
            label { "data-scope": SCOPE, "data-part": "label", r#for: ids.control.clone(), {label} }
            div { "data-scope": SCOPE, "data-part": "field",
                select {
                    "data-scope": SCOPE,
                    "data-part": "control",
                    id: ids.control.clone(),
                    name,
                    required,
                    disabled,
                    "aria-describedby": attrs.aria_describedby,
                    "aria-invalid": attrs.aria_invalid,
                    onchange: move |e| {
                        if let Some(handler) = on_change {
                            handler.call(e.value());
                        }
                    },
                    for o in options {
                        option {
                            key: "{o.value}",
                            value: o.value.clone(),
                            selected: selected.as_deref() == Some(o.value.as_str()),
                            {o.label}
                        }
                    }
                }
                span { "data-scope": SCOPE, "data-part": "icon", "aria-hidden": "true",
                    Icon { name: IconName::ChevronDown, size: 18 }
                }
            }
            if let Some(e) = error {
                p { "data-scope": SCOPE, "data-part": "error", id: ids.error.clone(), {e} }
            } else if let Some(t) = hint {
                p { "data-scope": SCOPE, "data-part": "hint", id: ids.hint.clone(), {t} }
            }
        }
    }
}
