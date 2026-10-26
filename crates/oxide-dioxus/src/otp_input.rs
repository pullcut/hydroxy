//! `OtpInput`: a one-time code, one box per character.

use dioxus::prelude::*;
use oxide_headless::otp_input::{self as h, OtpInputCommand, OtpInputEvent, OtpInputInput, OtpInputMachine, OtpInputState};
use oxide_statechart::Machine;

pub use oxide_headless::otp_input::OtpMode;

/// Write every cell's text from `state` and focus the focus cell. Cells keep
/// stray native text after a rejected keystroke, and an unchanged vdom value
/// isn't re-applied, so this runs after every edit. Through `document::eval`,
/// like the overlay helpers, so it works on web and in the desktop webview.
fn sync_dom(ids: &h::Ids, state: &OtpInputState, length: usize) {
    let cells: Vec<String> = (0..length).map(|i| ids.cell(i)).collect();
    let values: Vec<String> = (0..length).map(|i| state.char_at(i).map(String::from).unwrap_or_default()).collect();
    let _ = document::eval(&format!(
        "const ids = {cells:?}, values = {values:?};
         ids.forEach((id, i) => {{ const el = document.getElementById(id); if (el && el.value !== values[i]) el.value = values[i]; }});
         const f = document.getElementById(ids[{focus}]); if (f) {{ f.focus(); f.select(); }}",
        focus = state.focus,
    ));
}

/// A one-time code entered one character per box. Typing advances, Backspace
/// goes back, and pasting or autofilling a code fills every box.
#[component]
pub fn OtpInput(
    /// Visible label. Names the group.
    label: String,
    /// Number of boxes.
    #[props(default = 6)]
    length: usize,
    /// Controlled value. Pair with `on_value_change`. Must be `Some` at start to control.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: String,
    /// Digits only (default) or letters and digits.
    #[props(default)]
    mode: OtpMode,
    /// Block edits.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Marks the code wrong, such as after a failed check.
    #[props(default)]
    invalid: ReadSignal<bool>,
    /// Form field name. Submits the whole code.
    name: Option<String>,
    /// Called with the code whenever it changes or a change is requested.
    on_value_change: Option<EventHandler<String>>,
    /// Called with the code once every box is filled.
    on_complete: Option<EventHandler<String>>,
) -> Element {
    let length = length.max(1);
    let ids = use_hook(|| h::ids(&oxide_headless::next_id(h::SCOPE)));
    let input = use_memo(move || OtpInputInput {
        length,
        value: value(),
        default_value: default_value.clone(),
        mode,
        disabled: disabled(),
    });
    let mut state = use_signal(|| OtpInputMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = OtpInputMachine::sync(&state.peek(), &input.read());
        state.set(synced);
    });

    let send = {
        let ids = ids.clone();
        move |event: OtpInputEvent| {
            let step = OtpInputMachine::transition(&state.peek(), event, &input.peek());
            sync_dom(&ids, &step.state, length);
            state.set(step.state);
            for command in step.commands {
                match (command, on_value_change, on_complete) {
                    (OtpInputCommand::ValueChange(v), Some(handler), _) | (OtpInputCommand::Complete(v), _, Some(handler)) => {
                        handler.call(v);
                    }
                    _ => {}
                }
            }
        }
    };

    let s = state.read();
    let i = input.read();
    let is_disabled = i.disabled;
    let is_invalid = invalid();

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-state": h::root_state(&s, &i),
            "data-disabled": is_disabled.then_some(""),
            "data-invalid": is_invalid.then_some(""),
            span { "data-scope": h::SCOPE, "data-part": "label", id: "{ids.label}", "{label}" }
            div { "data-scope": h::SCOPE, "data-part": "cells", role: "group", "aria-labelledby": "{ids.label}",
                for index in 0..length {
                    {
                        let a = h::cell_attrs(&s, &i, index);
                        let (mut s1, mut s2, mut s3) = (send.clone(), send.clone(), send.clone());
                        rsx! {
                            input {
                                key: "{index}",
                                r#type: "text",
                                id: ids.cell(index),
                                "data-scope": h::SCOPE,
                                "data-part": "cell",
                                "data-index": index,
                                "data-state": a.data_state,
                                autocomplete: a.autocomplete,
                                inputmode: a.inputmode,
                                "aria-label": a.aria_label,
                                "aria-invalid": is_invalid.then_some("true"),
                                tabindex: a.tabindex,
                                spellcheck: "false",
                                autocapitalize: "off",
                                disabled: is_disabled,
                                value: a.value,
                                // Paste lands as a multi-character input event, which fills from here.
                                oninput: move |e| s1(OtpInputEvent::Input { index, text: e.value() }),
                                onkeydown: move |e| {
                                    if let Some(event) = h::keydown(&e.key().to_string(), index) {
                                        e.prevent_default();
                                        s2(event);
                                    }
                                },
                                onfocus: move |_| s3(OtpInputEvent::Focus(index)),
                            }
                        }
                    }
                }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name: "{name}", disabled: is_disabled, value: "{s.value}" }
            }
        }
    }
}
