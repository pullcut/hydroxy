//! The spinbutton segment that `<DateField>` and `<DurationField>` both render.

use hydroxy_headless::{date_field, duration_field};
use leptos::prelude::*;

/// What one segment shows and announces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SegmentView {
    pub id: String,
    pub label: &'static str,
    pub text: String,
    pub valuenow: Option<u32>,
    pub valuemin: u32,
    pub valuemax: u32,
    pub valuetext: String,
}

impl From<date_field::SegmentAttrs> for SegmentView {
    fn from(a: date_field::SegmentAttrs) -> Self {
        Self {
            id: a.id,
            label: a.label,
            text: a.text,
            valuenow: a.valuenow,
            valuemin: a.valuemin,
            valuemax: a.valuemax,
            valuetext: a.valuetext,
        }
    }
}

impl From<duration_field::SegmentAttrs> for SegmentView {
    fn from(a: duration_field::SegmentAttrs) -> Self {
        Self {
            id: a.id,
            label: a.label,
            text: a.text,
            valuenow: a.valuenow,
            valuemin: a.valuemin,
            valuemax: a.valuemax,
            valuetext: a.valuetext,
        }
    }
}

/// One numeric segment: a text input with the spinbutton role. The owner decides what keys
/// do (`on_keydown`) and what a typed digit means (`on_digit`); the input then shows the
/// segment's text again, so it never holds anything the machine did not accept.
#[component]
pub(crate) fn SegmentInput(
    /// The `data-scope` value.
    scope: &'static str,
    /// What the segment shows.
    #[prop(into)]
    attrs: Signal<SegmentView>,
    /// Editing is blocked.
    #[prop(into)]
    disabled: Signal<bool>,
    /// Focusable but not editable.
    read_only: bool,
    /// Marks the segment `aria-invalid`.
    #[prop(optional, into)]
    invalid: Signal<bool>,
    /// Width in `ch`. Unset leaves the width to the styles.
    #[prop(optional)]
    width: Option<u8>,
    /// Called when the segment takes focus.
    #[prop(into)]
    on_focus: Callback<()>,
    /// Called for every key press.
    #[prop(into)]
    on_keydown: Callback<web_sys::KeyboardEvent>,
    /// Called with a digit typed through `input` events (soft keyboards report no key).
    #[prop(into)]
    on_digit: Callback<u32>,
) -> impl IntoView {
    view! {
        <input
            type="text"
            inputmode="numeric"
            autocomplete="off"
            spellcheck="false"
            role="spinbutton"
            data-scope=scope
            data-part="segment"
            id=move || attrs.with(|a| a.id.clone())
            aria-label=move || attrs.with(|a| a.label)
            aria-valuenow=move || attrs.with(|a| a.valuenow)
            aria-valuemin=move || attrs.with(|a| a.valuemin)
            aria-valuemax=move || attrs.with(|a| a.valuemax)
            aria-valuetext=move || attrs.with(|a| a.valuetext.clone())
            aria-invalid=move || invalid.get().then_some("true")
            style=width.map(|w| format!("width: {w}ch;"))
            disabled=move || disabled.get()
            readonly=read_only
            prop:value=move || attrs.with(|a| a.text.clone())
            on:focus=move |_| on_focus.run(())
            on:keydown=move |ev| on_keydown.run(ev)
            on:input=move |ev| {
                // Take the typed digit, then put the segment's text back.
                let el = event_target::<web_sys::HtmlInputElement>(&ev);
                if let Some(digit) = date_field::input_digit(&el.value()) {
                    on_digit.run(digit);
                }
                el.set_value(&attrs.with_untracked(|a| a.text.clone()));
            }
        />
    }
}
