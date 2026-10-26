//! `<TimeField>`: a styled native `<input type="time">`.
//!
//! Native over a custom segmented field: the browser already gives segmented
//! hour/minute/AM-PM entry with arrow-key spinning, the locale's 12- or 24-hour
//! clock, a wheel picker on phones, `min`/`max`/`step` validation and form
//! submission, all announced correctly by screen readers. oxide styles the
//! field and its segments. The cost: the hour cycle follows the user's locale,
//! not a prop.

use leptos::either::Either;
use leptos::prelude::*;
use oxide_headless::input as h;

/// The `data-scope` value on every time field part.
pub const SCOPE: &str = "time-field";

/// A labelled time of day, with an optional hint or error. Values are `"HH:MM"`
/// (or `"HH:MM:SS"` with a sub-minute `step`), 24-hour, whatever the display.
#[component]
pub fn TimeField(
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Controlled value, such as `"09:30"`. Pair with `on_input`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial value when uncontrolled.
    #[prop(optional, into)]
    default_value: String,
    /// Earliest allowed time.
    #[prop(optional, into)]
    min: Option<String>,
    /// Latest allowed time.
    #[prop(optional, into)]
    max: Option<String>,
    /// Step in seconds. Default 60; below 60 shows a seconds segment.
    #[prop(optional)]
    step: Option<u32>,
    /// Called with the new value as it is edited. Empty while incomplete.
    #[prop(optional, into)]
    on_input: Option<Callback<String>>,
    /// Help text under the field. Hidden while there is an error.
    #[prop(optional, into)]
    hint: MaybeProp<String>,
    /// Error text. Replaces the hint and marks the field invalid.
    #[prop(optional, into)]
    error: MaybeProp<String>,
    /// Form field name.
    #[prop(optional, into)]
    name: Option<String>,
    /// Block editing.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
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
            (Some(e), _) => Some(Either::Left(view! { <p data-scope=SCOPE data-part="error" id=ids.error.clone()>{e}</p> })),
            (None, Some(t)) => Some(Either::Right(view! { <p data-scope=SCOPE data-part="hint" id=ids.hint.clone()>{t}</p> })),
            (None, None) => None,
        }
    };

    view! {
        <div
            data-scope=SCOPE
            data-part="root"
            data-invalid=move || error.with(Option::is_some).then_some("")
            data-disabled=move || disabled.get().unwrap_or(false).then_some("")
        >
            <label data-scope=SCOPE data-part="label" for=ids.control.clone()>{label}</label>
            <input
                data-scope=SCOPE
                data-part="control"
                type="time"
                id=ids.control.clone()
                min=min
                max=max
                step=step
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
            {below}
        </div>
    }
}
