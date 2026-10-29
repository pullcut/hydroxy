//! `<OtpInput>`: a one-time code, one box per character.

use crate::drive::drive;
use hydroxy_headless::otp_input::{
    self as h, OtpInputCommand, OtpInputEvent, OtpInputInput, OtpInputMachine, OtpInputState,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

pub use hydroxy_headless::otp_input::OtpMode;

/// Write every cell's text from `state` and focus the focus cell.
/// Cells keep stray native text after a rejected keystroke, so this runs after every edit.
fn sync_dom(ids: &h::Ids, state: &OtpInputState, length: usize) {
    let doc = document();
    for i in 0..length {
        if let Some(el) = doc.get_element_by_id(&ids.cell(i)) {
            let el = el.unchecked_into::<web_sys::HtmlInputElement>();
            let text = state.char_at(i).map(String::from).unwrap_or_default();
            if el.value() != text {
                el.set_value(&text);
            }
            if i == state.focus {
                let _ = el.focus();
                el.select();
            }
        }
    }
}

/// A one-time code entered one character per box. Typing advances, Backspace
/// goes back, and pasting or autofilling a code fills every box.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn OtpInput(
    /// Visible label. Names the group.
    #[prop(into)]
    label: String,
    /// Number of boxes. Default 6.
    #[prop(default = 6)]
    length: usize,
    /// Controlled value. Pair with `on_value_change`. Must be `Some` at start to control.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial value when uncontrolled.
    #[prop(optional, into)]
    default_value: String,
    /// Digits only (default) or letters and digits.
    #[prop(optional)]
    mode: OtpMode,
    /// Block edits.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Marks the code wrong, such as after a failed check.
    #[prop(optional, into)]
    invalid: MaybeProp<bool>,
    /// Form field name. Submits the whole code.
    #[prop(optional, into)]
    name: Option<String>,
    /// Called with the code whenever it changes or a change is requested.
    #[prop(optional, into)]
    on_value_change: Option<Callback<String>>,
    /// Called with the code once every box is filled.
    #[prop(optional, into)]
    on_complete: Option<Callback<String>>,
) -> impl IntoView {
    let length = length.max(1);
    let ids = h::ids(&crate::id::next_id(h::SCOPE));
    let input = Memo::new(move |_| OtpInputInput {
        length,
        value: value.get(),
        default_value: default_value.clone(),
        mode,
        disabled: disabled.get().unwrap_or(false),
    });
    let state = RwSignal::new(input.with_untracked(OtpInputMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| OtpInputMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let send = {
        let ids = ids.clone();
        move |event: OtpInputEvent| {
            // State first: focusing a cell re-enters `send` with a Focus event.
            let commands = drive::<OtpInputMachine, _>(state, input, event);
            if state.try_with_untracked(|s| sync_dom(&ids, s, length)).is_none() {
                return;
            }
            for command in commands {
                match command {
                    OtpInputCommand::ValueChange(v) => {
                        if let Some(cb) = on_value_change {
                            cb.run(v);
                        }
                    }
                    OtpInputCommand::Complete(v) => {
                        if let Some(cb) = on_complete {
                            cb.run(v);
                        }
                    }
                }
            }
        }
    };

    let is_disabled = move || disabled.get().unwrap_or(false);
    let cells = (0..length)
        .map(|i| {
            let send = send.clone();
            let (s1, s2, s3) = (send.clone(), send.clone(), send.clone());
            let attrs = Memo::new(move |_| input.with(|inp| state.with(|s| h::cell_attrs(s, inp, i))));
            view! {
                <input
                    type="text"
                    id=ids.cell(i)
                    data-scope=h::SCOPE
                    data-part="cell"
                    data-index=i
                    data-state=move || attrs.with(|x| x.data_state)
                    autocomplete=move || attrs.with(|x| x.autocomplete)
                    inputmode=move || attrs.with(|x| x.inputmode)
                    aria-label=move || attrs.with(|x| x.aria_label.clone())
                    aria-invalid=move || invalid.get().unwrap_or(false).then_some("true")
                    tabindex=move || attrs.with(|x| x.tabindex)
                    spellcheck="false"
                    autocapitalize="off"
                    disabled=is_disabled
                    prop:value=move || attrs.with(|x| x.value.clone())
                    on:input=move |ev| send(OtpInputEvent::Input { index: i, text: event_target_value(&ev) })
                    on:keydown=move |ev| {
                        if let Some(event) = h::keydown(&ev.key(), i) {
                            ev.prevent_default();
                            s1(event);
                        }
                    }
                    on:paste=move |ev| {
                        ev.prevent_default();
                        let text = ev.clipboard_data().and_then(|d| d.get_data("text").ok()).unwrap_or_default();
                        s2(OtpInputEvent::Paste { index: i, text });
                    }
                    on:focus=move |_| s3(OtpInputEvent::Focus(i))
                />
            }
        })
        .collect_view();

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-state=move || input.with(|i| state.with(|s| h::root_state(s, i)))
            data-disabled=move || is_disabled().then_some("")
            data-invalid=move || invalid.get().unwrap_or(false).then_some("")
        >
            <span data-scope=h::SCOPE data-part="label" id=ids.label.clone()>{label}</span>
            <div data-scope=h::SCOPE data-part="cells" role="group" aria-labelledby=ids.label.clone()>
                {cells}
            </div>
            {name.map(|name| {
                view! { <input type="hidden" name=name disabled=is_disabled prop:value=move || state.with(|s| s.value.value.clone()) /> }
            })}
        </div>
    }
}
