//! `PhoneField`: a country picker and a phone number that groups as you type,
//! reported in E.164.

use dioxus::prelude::*;
use hydroxy_headless::phone_field::{
    self as h, PhoneFieldCommand, PhoneFieldEvent, PhoneFieldInput, PhoneFieldMachine,
};
use hydroxy_statechart::Machine;

use crate::combobox::scroll_into_list;
use crate::drive::drive;
use crate::focus::focus_soon;
use crate::icon::{Icon, IconName};
use crate::money_field::{read_caret, write_back};
use crate::overlay::{hide_popover, show_popover};

/// What the number reports on every change: E.164 and whether it looks valid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhoneValue {
    /// `+` dial code and number, such as `+447400123456`. `None` when empty.
    pub e164: Option<String>,
    /// Plausible length for the chosen country.
    pub valid: bool,
}

/// An international phone number. The country button opens a searchable list
/// (name, code or dial code); typing groups the number in the country's style; a
/// number typed or pasted with `+` picks its country. Validity shows after blur.
#[component]
pub fn PhoneField(
    /// Visible label.
    label: String,
    /// Controlled E.164 value. Pair with `on_value_change`.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Initial E.164 value when uncontrolled.
    default_value: Option<String>,
    /// ISO code of the starting country when there is no value.
    #[props(default = "US".to_string(), into)]
    default_country: String,
    /// Block changes.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Focusable but not editable.
    #[props(default)]
    read_only: bool,
    /// Native `required` on the number.
    #[props(default)]
    required: bool,
    /// Help text under the field. Replaced by the error while invalid.
    hint: Option<String>,
    /// Form field name. Submits E.164.
    name: Option<String>,
    /// Called with E.164 and validity whenever the number changes.
    on_value_change: Option<EventHandler<PhoneValue>>,
    /// Called with the ISO code when the country changes.
    on_country_change: Option<EventHandler<String>>,
) -> Element {
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let ids = h::ids(&root);
    let input = use_memo(move || PhoneFieldInput {
        value: value(),
        default_value: default_value.clone(),
        default_country: default_country.clone(),
        disabled: disabled(),
        read_only,
    });
    let mut state = use_signal(|| PhoneFieldMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = PhoneFieldMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });

    let send = {
        let ids = ids.clone();
        move |event: PhoneFieldEvent| {
            for command in drive::<PhoneFieldMachine>(state, input, event) {
                match command {
                    PhoneFieldCommand::ValueChange { e164, valid } => {
                        if let Some(handler) = on_value_change {
                            handler.call(PhoneValue { e164, valid });
                        }
                    }
                    PhoneFieldCommand::CountryChange(iso) => {
                        if let Some(handler) = on_country_change {
                            handler.call(iso);
                        }
                    }
                    PhoneFieldCommand::FocusNumber => focus_soon(&ids.control),
                    PhoneFieldCommand::FocusTrigger => focus_soon(&ids.trigger),
                }
            }
        }
    };

    // The native popover follows the state; opening focuses the search box and
    // the highlighted country stays in view.
    {
        let (root, ids) = (root.clone(), ids.clone());
        use_effect(move || {
            let (open, highlighted) = {
                let s = state.read();
                (s.picker_open, s.highlighted.clone())
            };
            if open {
                show_popover(&ids.picker);
                focus_soon(&ids.search);
                if let Some(iso) = highlighted {
                    scroll_into_list(&h::option_id(&root, &iso));
                }
            } else {
                hide_popover(&ids.picker);
            }
        });
    }

    let s = state.read();
    let country = *s.country();
    let open = s.picker_open;
    let data_state = h::data_state(open);
    let trigger_label = h::trigger_label(&s);
    let placeholder = h::placeholder(&s);
    let error = h::error_text(&s);
    let text = s.text.clone();
    let query = s.query.clone();
    let active = h::active_descendant(&root, &s);
    let selected = s.country.clone();
    let highlighted = s.highlighted.clone();
    let status = open.then(|| h::result_count(&s.query));
    let form = h::form_value(&s);
    drop(s);
    let shown = h::visible(&query);
    let is_disabled = disabled();
    let invalid = error.is_some();
    let described = match (&error, &hint) {
        (Some(_), _) => Some(ids.error.clone()),
        (None, Some(_)) => Some(ids.hint.clone()),
        (None, None) => None,
    };
    let picker_id = ids.picker.clone();
    let control_id = ids.control.clone();
    let (s1, s2, s3, s4, s5, s6) = (send.clone(), send.clone(), send.clone(), send.clone(), send.clone(), send.clone());

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-disabled": is_disabled.then_some(""),
            "data-readonly": read_only.then_some(""),
            "data-invalid": invalid.then_some(""),
            label { id: "{ids.label}", "data-scope": h::SCOPE, "data-part": "label", r#for: "{ids.control}", "{label}" }
            div { "data-scope": h::SCOPE, "data-part": "field", style: h::field_style(&root),
                button {
                    r#type: "button",
                    id: "{ids.trigger}",
                    "data-scope": h::SCOPE,
                    "data-part": "trigger",
                    "aria-label": trigger_label,
                    "aria-haspopup": "dialog",
                    "aria-expanded": open.to_string(),
                    "aria-controls": "{ids.picker}",
                    "popovertarget": "{ids.picker}",
                    "data-state": data_state,
                    disabled: is_disabled || read_only,
                    onclick: move |e| {
                        // Keep `popovertarget` for its light-dismiss exemption, but let the machine toggle.
                        e.prevent_default();
                        s1(PhoneFieldEvent::TogglePicker);
                    },
                    onkeydown: move |e| {
                        if let Some(event) = h::trigger_keydown(&e.key().to_string()) {
                            e.prevent_default();
                            s2(event);
                        }
                    },
                    span { "data-scope": h::SCOPE, "data-part": "flag", "aria-hidden": "true", {h::flag(country.iso)} }
                    span { "data-scope": h::SCOPE, "data-part": "dial", "aria-hidden": "true", "+{country.dial}" }
                    Icon { name: IconName::ChevronDown, size: 14 }
                }
                input {
                    r#type: "tel",
                    id: "{ids.control}",
                    "data-scope": h::SCOPE,
                    "data-part": "control",
                    inputmode: "tel",
                    autocomplete: "tel-national",
                    placeholder,
                    required,
                    disabled: is_disabled,
                    readonly: read_only,
                    "aria-invalid": invalid.then_some("true"),
                    "aria-describedby": described,
                    value: "{text}",
                    oninput: move |e| {
                        let text = e.value();
                        let id = control_id.clone();
                        let send = s3.clone();
                        spawn(async move {
                            let caret = read_caret(&id).await;
                            send(PhoneFieldEvent::SetText { text, caret });
                            let (text, caret) = {
                                let s = state.peek();
                                (s.text.clone(), s.caret)
                            };
                            write_back(&id, &text, caret);
                        });
                    },
                    onblur: move |_| s4(PhoneFieldEvent::Blur),
                }
            }
            div {
                id: "{ids.picker}",
                "popover": "auto",
                role: "dialog",
                "aria-label": "Choose country",
                style: h::picker_style(&root),
                "data-overlay": "",
                "data-scope": h::SCOPE,
                "data-part": "picker",
                "data-state": data_state,
                ontoggle: move |_| {
                    // The browser may have closed it (outside click): ask, then catch the machine up.
                    let js = format!("return document.getElementById({picker_id})?.matches(':popover-open') ?? false", picker_id = crate::js::js_string(&picker_id));
                    let send = s5.clone();
                    spawn(async move {
                        if let Ok(false) = document::eval(&js).join::<bool>().await {
                            send(PhoneFieldEvent::ClosePicker { restore_focus: false });
                        }
                    });
                },
                div { "data-scope": h::SCOPE, "data-part": "search-field",
                    Icon { name: IconName::Search, size: 14 }
                    input {
                        r#type: "text",
                        id: "{ids.search}",
                        "data-scope": h::SCOPE,
                        "data-part": "search",
                        role: "combobox",
                        "aria-label": "Search countries",
                        "aria-autocomplete": "list",
                        "aria-expanded": open.to_string(),
                        "aria-controls": "{ids.listbox}",
                        "aria-activedescendant": active,
                        autocomplete: "off",
                        spellcheck: "false",
                        placeholder: "Country or code",
                        value: "{query}",
                        oninput: move |e: FormEvent| send(PhoneFieldEvent::Query(e.value())),
                        onkeydown: move |e| {
                            if let Some(event) = h::search_keydown(&e.key().to_string()) {
                                e.prevent_default();
                                if matches!(event, PhoneFieldEvent::ClosePicker { .. }) {
                                    e.stop_propagation();
                                }
                                s6(event);
                            }
                        },
                    }
                }
                div { id: "{ids.listbox}", role: "listbox", "aria-label": "Countries", "data-scope": h::SCOPE, "data-part": "listbox",
                    if shown.is_empty() {
                        div { "data-scope": h::SCOPE, "data-part": "empty", "No countries match" }
                    }
                    for c in shown {
                        div {
                            key: "{c.iso}",
                            id: h::option_id(&root, c.iso),
                            role: "option",
                            "aria-selected": (selected == c.iso).to_string(),
                            "data-scope": h::SCOPE,
                            "data-part": "option",
                            "data-highlighted": (highlighted.as_deref() == Some(c.iso)).then_some(""),
                            // Keep focus in the search box.
                            onpointerdown: move |e| e.prevent_default(),
                            onpointermove: {
                                let send = send.clone();
                                move |_| {
                                    if state.peek().highlighted.as_deref() != Some(c.iso) {
                                        send(PhoneFieldEvent::Highlight(c.iso.to_string()));
                                    }
                                }
                            },
                            onclick: {
                                let send = send.clone();
                                move |_| send(PhoneFieldEvent::Select(c.iso.to_string()))
                            },
                            span { "data-scope": h::SCOPE, "data-part": "flag", "aria-hidden": "true", {h::flag(c.iso)} }
                            span { "data-scope": h::SCOPE, "data-part": "option-name", "{c.name}" }
                            span { "data-scope": h::SCOPE, "data-part": "option-dial", "+{c.dial}" }
                            if selected == c.iso {
                                Icon { name: IconName::Check, size: 14 }
                            }
                        }
                    }
                }
                span { "data-scope": h::SCOPE, "data-part": "status", role: "status", "aria-live": "polite", {status} }
            }
            if let Some(e) = error {
                p { id: "{ids.error}", "data-scope": h::SCOPE, "data-part": "error", "{e}" }
            } else if let Some(t) = hint {
                p { id: "{ids.hint}", "data-scope": h::SCOPE, "data-part": "hint", "{t}" }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name: "{name}", disabled: is_disabled, value: form }
            }
        }
    }
}
