//! `<PasswordField>`: a password input with a show/hide toggle and a Caps Lock warning. In
//! [`PasswordMode::New`] it also shows a strength meter and a live checklist of rules.

use leptos::prelude::*;
use hydroxy_headless::password_field::{
    self as h, PasswordFieldCommand, PasswordFieldEvent, PasswordFieldInput, PasswordFieldMachine,
};
use hydroxy_statechart::Machine;

use crate::icon::{Icon, IconName};

pub use hydroxy_headless::password_field::{PasswordMode, Rule, RuleKind, default_rules};

/// A password input. New-password mode adds the meter and checklist; current-password mode
/// is the sign-in field.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn PasswordField(
    /// Visible label.
    #[prop(default = "Password".into(), into)]
    label: String,
    /// New (meter, rules, `new-password`) or current (`current-password`).
    #[prop(optional)]
    mode: PasswordMode,
    /// Controlled value. Pair with `on_change`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Called with the text on every edit.
    #[prop(optional, into)]
    on_change: Option<Callback<String>>,
    /// Called with whether every rule passes, on every edit.
    #[prop(optional, into)]
    on_valid_change: Option<Callback<bool>>,
    /// The checklist. Defaults to [`default_rules`]. Empty hides it.
    #[prop(optional, into)]
    rules: Option<Vec<Rule>>,
    /// Words the person used elsewhere (name, email) that make a password easy to guess.
    #[prop(optional, into)]
    user_inputs: Signal<Vec<String>>,
    /// Help text under the field.
    #[prop(optional, into)]
    hint: Option<String>,
    /// Form field name.
    #[prop(optional, into)]
    name: Option<String>,
    /// Block editing.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Read-only: the toggle still works.
    #[prop(optional)]
    readonly: bool,
    /// Mark the input required.
    #[prop(optional)]
    required: bool,
) -> impl IntoView {
    let input = Memo::new(move |_| PasswordFieldInput {
        value: value.get(),
        default_value: String::new(),
        disabled: disabled.get().unwrap_or(false),
    });
    let state = RwSignal::new(input.with_untracked(PasswordFieldMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| PasswordFieldMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let rules = StoredValue::new(if mode == PasswordMode::New { rules.unwrap_or_else(default_rules) } else { Vec::new() });
    let ids = h::ids(&crate::id::next_id(h::SCOPE));
    let off = move || input.with(|i| i.disabled);

    let send = move |event: PasswordFieldEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| PasswordFieldMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        for PasswordFieldCommand::Change(text) in step.commands {
            if let Some(cb) = on_change {
                cb.run(text.clone());
            }
            if let Some(cb) = on_valid_change {
                let valid = rules.with_value(|r| h::report(&text, r, &user_inputs.get_untracked()).valid);
                cb.run(valid);
            }
        }
    };

    let report = Memo::new(move |_| {
        let v = state.with(|s| s.value.clone());
        user_inputs.with(|u| rules.with_value(|r| h::report(&v, r, u)))
    });
    let text = move || state.with(|s| s.value.clone());
    let visible = move || state.with(|s| s.visible);
    let caps = move || state.with(|s| s.caps_lock);
    let has_rules = rules.with_value(|r| !r.is_empty());
    let has_hint = hint.is_some();
    let rules_attr = rules.with_value(|r| h::password_rules_attr(r));
    // `passwordrules` isn't in Leptos's attribute list, so set it once mounted.
    let control = NodeRef::<leptos::html::Input>::new();
    Effect::new(move |_| {
        if let (Some(el), false) = (control.get(), rules_attr.is_empty()) {
            let _ = el.set_attribute("passwordrules", &rules_attr);
        }
    });

    let caps_key = move |ev: &leptos::ev::KeyboardEvent| {
        if h::reports_caps_lock(&ev.key()) {
            send(PasswordFieldEvent::CapsLock(ev.get_modifier_state("CapsLock")));
        }
    };

    let described = {
        let ids = ids.clone();
        move || h::described_by(&ids, has_hint, caps(), mode, has_rules)
    };

    let meter = (mode == PasswordMode::New).then(|| {
        let score = move || report.with(|r| r.strength.score);
        let warning = move || {
            report.with(|r| r.strength.warning.filter(|_| !text().is_empty() && r.strength.score < 3).map(hydroxy_headless::password_field::Warning::message))
        };
        view! {
            <div data-scope=h::SCOPE data-part="meter-wrap">
                <div
                    data-scope=h::SCOPE
                    data-part="meter"
                    role="meter"
                    aria-label="Password strength"
                    aria-valuemin="0"
                    aria-valuemax="4"
                    aria-valuenow=move || if text().is_empty() { 0 } else { score() }
                    aria-valuetext=move || report.with(|r| h::meter_valuetext(&text(), &r.strength))
                    data-strength=move || report.with(|r| h::data_strength(&text(), &r.strength))
                >
                    {(0..4)
                        .map(|i| {
                            view! {
                                <span
                                    data-scope=h::SCOPE
                                    data-part="meter-bar"
                                    data-on=move || (!text().is_empty() && score() > i).then_some("")
                                ></span>
                            }
                        })
                        .collect_view()}
                </div>
                <p id=ids.strength.clone() data-scope=h::SCOPE data-part="strength" aria-live="polite">
                    {move || report.with(|r| h::strength_text(&text(), &r.strength))}
                    {move || warning().map(|m| view! { <span data-scope=h::SCOPE data-part="warning">" " {m}</span> })}
                </p>
            </div>
        }
    });

    let checklist = has_rules.then(|| {
        let items = rules.get_value();
        view! {
            <ul id=ids.rules.clone() data-scope=h::SCOPE data-part="rules" aria-label="Password requirements">
                {items
                    .into_iter()
                    .enumerate()
                    .map(|(i, rule)| {
                        let passed = move || report.with(|r| r.passed.get(i).copied().unwrap_or(false));
                        view! {
                            <li data-scope=h::SCOPE data-part="rule" data-met=move || passed().then_some("")>
                                <span data-scope=h::SCOPE data-part="rule-mark" aria-hidden="true">
                                    <Icon name=IconName::Check size=12 />
                                </span>
                                <span>{rule.label}</span>
                                <span data-scope=h::SCOPE data-part="sr-only">
                                    {move || format!(", {}", h::rule_status(passed()))}
                                </span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        }
    });

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-mode=if mode == PasswordMode::New { "new" } else { "current" }
            data-disabled=move || off().then_some("")
            data-valid=move || report.with(|r| (has_rules && r.valid).then_some(""))
        >
            <label data-scope=h::SCOPE data-part="label" for=ids.control.clone()>
                {label}
            </label>
            <div data-scope=h::SCOPE data-part="field">
                <input
                    id=ids.control.clone()
                    data-scope=h::SCOPE
                    data-part="control"
                    type=move || h::input_type(visible())
                    name=name
                    autocomplete=mode.autocomplete()
                    autocapitalize="off"
                    spellcheck="false"
                    node_ref=control
                    required=required
                    readonly=readonly
                    disabled=off
                    aria-describedby=described
                    prop:value=text
                    on:input=move |ev| send(PasswordFieldEvent::Input(event_target_value(&ev)))
                    on:keydown=move |ev| caps_key(&ev)
                    on:keyup=move |ev| caps_key(&ev)
                    on:blur=move |_| send(PasswordFieldEvent::Blur)
                />
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="toggle"
                    aria-label=h::TOGGLE_LABEL
                    title=move || if visible() { "Hide password" } else { "Show password" }
                    aria-pressed=move || visible().to_string()
                    aria-controls=ids.control.clone()
                    disabled=off
                    on:click=move |_| send(PasswordFieldEvent::ToggleVisible)
                >
                    {move || {
                        view! { <Icon name=if visible() { IconName::EyeOff } else { IconName::Eye } size=18 /> }
                    }}
                </button>
            </div>
            // The live region stays in the DOM so the warning is announced when it appears.
            <p id=ids.caps.clone() data-scope=h::SCOPE data-part="caps-lock" role="status" data-active=move || caps().then_some("")>
                {move || caps().then(|| view! { <Icon name=IconName::TriangleAlert size=14 /> "Caps Lock is on" })}
            </p>
            {hint.map(|t| view! { <p id=ids.hint.clone() data-scope=h::SCOPE data-part="hint">{t}</p> })}
            {meter}
            {checklist}
        </div>
    }
}
