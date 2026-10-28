//! `<Input>`: a labelled native text field with an optional hint or error.

use leptos::either::Either;
use leptos::prelude::*;
use hydroxy_headless::input as h;

pub use hydroxy_headless::input::Size;

/// Text field with a label, and a hint or an error below it.
#[component]
pub fn Input(
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Help text under the field. Hidden while there is an error.
    #[prop(optional, into)]
    hint: MaybeProp<String>,
    /// Error text. Replaces the hint and marks the field invalid.
    #[prop(optional, into)]
    error: MaybeProp<String>,
    /// Controlled value. Pair with `on_input`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial value when uncontrolled.
    #[prop(optional, into)]
    default_value: String,
    /// Called with the new text on every keystroke.
    #[prop(optional, into)]
    on_input: Option<Callback<String>>,
    /// The native `type`. Default `text`.
    #[prop(optional, into, default = "text".into())]
    input_type: String,
    /// Placeholder text.
    #[prop(optional, into)]
    placeholder: Option<String>,
    /// Form field name.
    #[prop(optional, into)]
    name: Option<String>,
    /// Block editing.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Native `required`.
    #[prop(optional)]
    required: bool,
    /// Monospaced text, for money and rates.
    #[prop(optional)]
    mono: bool,
    /// Field height.
    #[prop(optional)]
    size: Size,
    /// Text before the value inside the field, such as `£`.
    #[prop(optional, into)]
    prefix: Option<String>,
    /// Text after the value inside the field, such as `%`.
    #[prop(optional, into)]
    suffix: Option<String>,
) -> impl IntoView {
    let ids = h::ids(&crate::id::next_id(h::SCOPE));
    let attrs = {
        let ids = ids.clone();
        Memo::new(move |_| h::control_attrs(&ids, hint.with(Option::is_some), error.with(Option::is_some)))
    };
    let below = {
        let ids = ids.clone();
        move || match (error.get(), hint.get()) {
            (Some(e), _) => {
                Some(Either::Left(view! { <p data-scope=h::SCOPE data-part="error" id=ids.error.clone()>{e}</p> }))
            }
            (None, Some(t)) => {
                Some(Either::Right(view! { <p data-scope=h::SCOPE data-part="hint" id=ids.hint.clone()>{t}</p> }))
            }
            (None, None) => None,
        }
    };
    let affix = |text: Option<String>| text.map(|t| view! { <span data-scope=h::SCOPE data-part="affix">{t}</span> });

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-size=size.as_str()
            data-mono=mono.then_some("")
            data-invalid=move || error.with(Option::is_some).then_some("")
            data-disabled=move || disabled.get().unwrap_or(false).then_some("")
        >
            <label data-scope=h::SCOPE data-part="label" for=ids.control.clone()>{label}</label>
            <div data-scope=h::SCOPE data-part="field">
                {affix(prefix)}
                <input
                    data-scope=h::SCOPE
                    data-part="control"
                    id=ids.control.clone()
                    type=input_type
                    placeholder=placeholder
                    name=name
                    required=required
                    disabled=move || disabled.get().unwrap_or(false)
                    aria-describedby=move || attrs.get().aria_describedby
                    aria-invalid=move || attrs.get().aria_invalid
                    prop:value=move || value.get().unwrap_or_else(|| default_value.clone())
                    on:input=move |ev| {
                        if let Some(cb) = on_input {
                            cb.run(event_target_value(&ev));
                        }
                    }
                />
                {affix(suffix)}
            </div>
            {below}
        </div>
    }
}
