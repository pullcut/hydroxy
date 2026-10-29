//! `<Select>`: a native `<select>` styled to match `Input`, with a label and a
//! hint or error. The native element is the behaviour: keyboard, typeahead and
//! the platform picker all come free.

use hydroxy_headless::input as h;
use leptos::either::Either;
use leptos::prelude::*;

use crate::icon::{Icon, IconName};

pub use hydroxy_headless::select::{SCOPE, SelectOption};

/// A labelled dropdown of options.
#[component]
pub fn Select(
    /// Visible label.
    #[prop(into)]
    label: String,
    /// The choices, in display order.
    #[prop(into)]
    options: Signal<Vec<SelectOption>>,
    /// Controlled value. Pair with `on_change`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial value when uncontrolled. Defaults to the first option.
    #[prop(optional, into)]
    default_value: Option<String>,
    /// Called with the chosen value.
    #[prop(optional, into)]
    on_change: Option<Callback<String>>,
    /// Help text under the field. Hidden while there is an error.
    #[prop(optional, into)]
    hint: MaybeProp<String>,
    /// Error text. Replaces the hint and marks the field invalid.
    #[prop(optional, into)]
    error: MaybeProp<String>,
    /// Block changes.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Form field name.
    #[prop(optional, into)]
    name: Option<String>,
    /// Native `required`.
    #[prop(optional)]
    required: bool,
) -> impl IntoView {
    let ids = h::ids(&crate::id::next_id(SCOPE));
    let attrs = {
        let ids = ids.clone();
        Memo::new(move |_| h::control_attrs(&ids, hint.with(Option::is_some), error.with(Option::is_some)))
    };
    let below = {
        let ids = ids.clone();
        move || match (error.get(), hint.get()) {
            (Some(e), _) => {
                Some(Either::Left(view! { <p data-scope=SCOPE data-part="error" id=ids.error.clone()>{e}</p> }))
            }
            (None, Some(t)) => {
                Some(Either::Right(view! { <p data-scope=SCOPE data-part="hint" id=ids.hint.clone()>{t}</p> }))
            }
            (None, None) => None,
        }
    };
    // Controlled: follow `value`. Uncontrolled: only the first render reads `default_value`.
    let selected = move || value.get().or_else(|| default_value.clone());

    view! {
        <div
            data-scope=SCOPE
            data-part="root"
            data-invalid=move || error.with(Option::is_some).then_some("")
            data-disabled=move || disabled.get().unwrap_or(false).then_some("")
        >
            <label data-scope=SCOPE data-part="label" for=ids.control.clone()>{label}</label>
            <div data-scope=SCOPE data-part="field">
                <select
                    data-scope=SCOPE
                    data-part="control"
                    id=ids.control
                    name=name
                    required=required
                    disabled=move || disabled.get().unwrap_or(false)
                    aria-describedby=move || attrs.with(|x| x.aria_describedby.clone())
                    aria-invalid=move || attrs.with(|x| x.aria_invalid)
                    on:change=move |ev| {
                        if let Some(cb) = on_change {
                            cb.run(event_target_value(&ev));
                        }
                    }
                >
                    <For each=move || options.get() key=|o| o.value.clone() let:o>
                        {
                            let is_selected = {
                                let v = o.value.clone();
                                let selected = selected.clone();
                                move || selected().as_deref() == Some(v.as_str())
                            };
                            view! { <option value=o.value prop:selected=is_selected>{o.label}</option> }
                        }
                    </For>
                </select>
                <span data-scope=SCOPE data-part="icon" aria-hidden="true">
                    <Icon name=IconName::ChevronDown size=18 />
                </span>
            </div>
            {below}
        </div>
    }
}
