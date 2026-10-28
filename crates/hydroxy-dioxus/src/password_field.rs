//! `PasswordField`: a password input with a show/hide toggle and a Caps Lock warning. In
//! [`PasswordMode::New`] it also shows a strength meter and a live checklist of rules.

use dioxus::prelude::*;
use hydroxy_headless::password_field::{
    self as h, PasswordFieldCommand, PasswordFieldEvent, PasswordFieldInput, PasswordFieldMachine,
};
use hydroxy_statechart::Machine;

use crate::icon::{Icon, IconName};

pub use hydroxy_headless::password_field::{PasswordMode, Rule, RuleKind, default_rules};

/// A password input. New-password mode adds the meter and checklist; current-password mode
/// is the sign-in field.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn PasswordField(
    /// Visible label.
    #[props(default = "Password".to_string())]
    label: String,
    /// New (meter, rules, `new-password`) or current (`current-password`).
    #[props(default)]
    mode: PasswordMode,
    /// Controlled value. Pair with `on_change`.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Called with the text on every edit.
    on_change: Option<EventHandler<String>>,
    /// Called with whether every rule passes, on every edit.
    on_valid_change: Option<EventHandler<bool>>,
    /// The checklist. Defaults to [`default_rules`]. Empty hides it.
    rules: Option<Vec<Rule>>,
    /// Words the person used elsewhere (name, email) that make a password easy to guess.
    #[props(default)]
    user_inputs: ReadSignal<Vec<String>>,
    /// Help text under the field.
    hint: Option<String>,
    /// Form field name.
    name: Option<String>,
    /// Block editing.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Read-only: the toggle still works.
    #[props(default)]
    readonly: bool,
    /// Mark the input required.
    #[props(default)]
    required: bool,
) -> Element {
    let input = use_memo(move || PasswordFieldInput { value: value(), default_value: String::new(), disabled: disabled() });
    let mut state = use_signal(|| PasswordFieldMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = PasswordFieldMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let rule_list = use_signal(|| if mode == PasswordMode::New { rules.unwrap_or_else(default_rules) } else { Vec::new() });
    let ids = use_hook(|| h::ids(&hydroxy_headless::next_id(h::SCOPE)));

    let mut send = move |event: PasswordFieldEvent| {
        let step = PasswordFieldMachine::transition(&state.peek(), event, &input.peek());
        if let Some(next) = step.state {
            state.set(next);
        }
        for PasswordFieldCommand::Change(text) in step.commands {
            if let Some(cb) = on_change {
                cb.call(text.clone());
            }
            if let Some(cb) = on_valid_change {
                cb.call(h::report(&text, &rule_list.peek(), &user_inputs.peek()).valid);
            }
        }
    };

    // Dioxus key events don't carry Caps Lock, so read it from the DOM event directly.
    let control_id = ids.control.clone();
    use_future(move || {
        let id = control_id.clone();
        async move {
            let mut js = document::eval(&format!(
                "const el = document.getElementById({id:?});
                if (el) {{
                  const f = (e) => {{
                    if (e.key === 'CapsLock' || (e.key && e.key.length === 1)) dioxus.send(e.getModifierState('CapsLock'));
                  }};
                  el.addEventListener('keydown', f);
                  el.addEventListener('keyup', f);
                }}
                await new Promise(() => {{}});"
            ));
            while let Ok(on) = js.recv::<bool>().await {
                send(PasswordFieldEvent::CapsLock(on));
            }
        }
    });

    let rules = rule_list.read();
    let s = state.read();
    let text = s.value.clone();
    let visible = s.visible;
    let caps = s.caps_lock;
    let off = input.read().disabled;
    let report = h::report(&text, &rules, &user_inputs.read());
    let has_rules = !rules.is_empty();
    let rules_attr = h::password_rules_attr(&rules);
    let described = h::described_by(&ids, hint.is_some(), caps, mode, has_rules);
    let score = report.strength.score;
    let warning = report.strength.warning.filter(|_| !text.is_empty() && score < 3).map(hydroxy_headless::password_field::Warning::message);
    let empty = text.is_empty();

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-mode": if mode == PasswordMode::New { "new" } else { "current" },
            "data-disabled": off.then_some(""),
            "data-valid": (has_rules && report.valid).then_some(""),
            label { "data-scope": h::SCOPE, "data-part": "label", r#for: ids.control.clone(), "{label}" }
            div { "data-scope": h::SCOPE, "data-part": "field",
                input {
                    id: ids.control.clone(),
                    "data-scope": h::SCOPE,
                    "data-part": "control",
                    r#type: h::input_type(visible),
                    name: name.clone(),
                    autocomplete: mode.autocomplete(),
                    autocapitalize: "off",
                    spellcheck: "false",
                    "passwordrules": (!rules_attr.is_empty()).then_some(rules_attr),
                    required,
                    readonly,
                    disabled: off,
                    "aria-describedby": described,
                    value: text.clone(),
                    oninput: move |e| send(PasswordFieldEvent::Input(e.value())),
                    onblur: move |_| send(PasswordFieldEvent::Blur),
                }
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "toggle",
                    "aria-label": h::TOGGLE_LABEL,
                    title: if visible { "Hide password" } else { "Show password" },
                    "aria-pressed": "{visible}",
                    "aria-controls": ids.control.clone(),
                    disabled: off,
                    onclick: move |_| send(PasswordFieldEvent::ToggleVisible),
                    Icon { name: if visible { IconName::EyeOff } else { IconName::Eye }, size: 18 }
                }
            }
            p {
                id: ids.caps.clone(),
                "data-scope": h::SCOPE,
                "data-part": "caps-lock",
                role: "status",
                "data-active": caps.then_some(""),
                if caps {
                    Icon { name: IconName::TriangleAlert, size: 14 }
                    "Caps Lock is on"
                }
            }
            if let Some(t) = hint.clone() {
                p { id: ids.hint.clone(), "data-scope": h::SCOPE, "data-part": "hint", "{t}" }
            }
            if mode == PasswordMode::New {
                div { "data-scope": h::SCOPE, "data-part": "meter-wrap",
                    div {
                        "data-scope": h::SCOPE,
                        "data-part": "meter",
                        role: "meter",
                        "aria-label": "Password strength",
                        "aria-valuemin": "0",
                        "aria-valuemax": "4",
                        "aria-valuenow": if empty { "0".to_string() } else { score.to_string() },
                        "aria-valuetext": h::meter_valuetext(&text, &report.strength),
                        "data-strength": h::data_strength(&text, &report.strength),
                        for i in 0..4u8 {
                            span {
                                key: "{i}",
                                "data-scope": h::SCOPE,
                                "data-part": "meter-bar",
                                "data-on": (!empty && score > i).then_some(""),
                            }
                        }
                    }
                    p { id: ids.strength.clone(), "data-scope": h::SCOPE, "data-part": "strength", "aria-live": "polite",
                        {h::strength_text(&text, &report.strength)}
                        if let Some(m) = warning {
                            span { "data-scope": h::SCOPE, "data-part": "warning", " {m}" }
                        }
                    }
                }
            }
            if has_rules {
                ul {
                    id: ids.rules.clone(),
                    "data-scope": h::SCOPE,
                    "data-part": "rules",
                    "aria-label": "Password requirements",
                    for (i, rule) in rules.iter().enumerate() {
                        li {
                            key: "{i}",
                            "data-scope": h::SCOPE,
                            "data-part": "rule",
                            "data-met": report.passed[i].then_some(""),
                            span { "data-scope": h::SCOPE, "data-part": "rule-mark", "aria-hidden": "true",
                                Icon { name: IconName::Check, size: 12 }
                            }
                            span { "{rule.label}" }
                            span { "data-scope": h::SCOPE, "data-part": "sr-only", ", {h::rule_status(report.passed[i])}" }
                        }
                    }
                }
            }
        }
    }
}
