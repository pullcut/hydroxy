use leptos::prelude::*;
use hydroxy_leptos::otp_input::{OtpInput, OtpMode};

pub fn demo() -> AnyView {
    let (status, set_status) = signal("Enter the 6-digit code we sent you".to_string());
    let (wrong, set_wrong) = signal(false);

    view! {
        <div style="display: grid; gap: var(--space-8);">
            <div style="display: grid; gap: var(--space-3);">
                <OtpInput
                    label="Verification code"
                    invalid=wrong
                    on_value_change=move |_| set_wrong.set(false)
                    on_complete=move |code: String| {
                        let ok = code == "123456";
                        set_wrong.set(!ok);
                        set_status.set(if ok { "Verified".into() } else { "That code didn't match. Try 123456".into() });
                    }
                />
                <p style="margin: 0; font: var(--text-sm); color: var(--text-3);">{status}</p>
            </div>

            <OtpInput label="Recovery key" length=4 mode=OtpMode::Alphanumeric name="key" />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "otp-input",
    name: "OTP input",
    group: "Forms",
    summary: "A one-time code, one box per character. Typing advances, Backspace goes back, and paste or SMS autofill fills every box.",
    demo,
    leptos: include_str!("otp_input.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/otp_input.rs"),
    parts: &[
        &["OtpInput", "root", "Wrapper. data-state complete or incomplete, data-invalid, data-disabled"],
        &["OtpInput", "label", "Names the group"],
        &["OtpInput", "cells", "role=group around the boxes"],
        &["OtpInput", "cell", "One text input per character. data-state filled or empty; the first has autocomplete=one-time-code"],
    ],
    keys: &[
        &["0-9 (A-Z)", "Fill the box and move to the next"],
        &["Backspace", "Clear the box, or the previous one when empty"],
        &["Delete", "Clear the box and shift the rest left"],
        &["ArrowLeft / ArrowRight", "Previous / next box"],
        &["Home / End", "First / last filled box"],
        &["Tab", "Leave the group. Only one box is in the tab order"],
    ],
    props: &[
        &["label", "String", "Required. Visible label"],
        &["length", "usize", "Number of boxes. Default 6"],
        &["value", "Option<String>", "Controlled code. Pair with on_value_change"],
        &["default_value", "String", "Initial code when uncontrolled"],
        &["mode", "OtpMode", "Numeric (default) or Alphanumeric"],
        &["on_value_change", "Callback<String>", "Called on every change"],
        &["on_complete", "Callback<String>", "Called once every box is filled"],
        &["invalid", "bool", "Mark the code wrong, such as after a failed check"],
        &["disabled", "bool", "Block edits"],
        &["name", "String", "Form field name. Submits the whole code"],
    ],
};
