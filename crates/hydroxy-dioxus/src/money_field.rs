//! `MoneyField`: an amount with a currency picker, held as integer minor units.

use dioxus::prelude::*;
use hydroxy_headless::money_field::{
    self as h, MoneyFieldCommand, MoneyFieldEvent, MoneyFieldInput, MoneyFieldMachine,
};
use hydroxy_statechart::Machine;

use crate::icon::{Icon, IconName};

/// Read a text input's caret (a UTF-16 offset) through `document::eval`.
pub(crate) async fn read_caret(id: &str) -> usize {
    let js = format!("return document.getElementById({id:?})?.selectionStart ?? null;");
    document::eval(&js).join::<Option<usize>>().await.ok().flatten().unwrap_or(usize::MAX)
}

/// Write reformatted text into an input and put the caret at `caret` once the
/// render has landed. The text is BMP only, so char indices are UTF-16 offsets.
pub(crate) fn write_back(id: &str, text: &str, caret: usize) {
    let _ = document::eval(&format!(
        "const el = document.getElementById({id:?});
         if (el) {{
           if (el.value !== {text:?}) el.value = {text:?};
           requestAnimationFrame(() => {{ if (document.activeElement === el) el.setSelectionRange({caret}, {caret}); }});
         }}"
    ));
}

/// An amount in a currency. Typing groups thousands as you go and stops at the
/// currency's precision; pasting reads `$1,234.50` or `1.234,50 €`; Enter or blur
/// clamps to `min`..`max`. Arrows step one major unit (Shift or PageUp/PageDown: ten).
/// Values are minor units: cents, yen, fils.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn MoneyField(
    /// Visible label.
    label: String,
    /// Controlled amount in minor units. Pair with `on_value_change`.
    #[props(default)]
    value: ReadSignal<Option<i64>>,
    /// Initial amount in minor units when uncontrolled.
    default_value: Option<i64>,
    /// Controlled currency code. Pair with `on_currency_change`.
    #[props(default)]
    currency: ReadSignal<Option<String>>,
    /// Currency to start in when uncontrolled.
    #[props(default = "USD".to_string(), into)]
    default_currency: String,
    /// Codes offered in the picker. Empty offers every known currency; one code hides the picker.
    #[props(default)]
    currencies: Vec<String>,
    /// Lowest amount in minor units. Zero or more also blocks the minus sign.
    min: Option<i64>,
    /// Highest amount in minor units.
    max: Option<i64>,
    /// Arrow step in minor units. Default one major unit.
    step: Option<i64>,
    /// Thousands separator.
    #[props(default = ',')]
    group: char,
    /// Decimal mark.
    #[props(default = '.')]
    decimal: char,
    /// Block changes and drop out of the tab order.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Focusable but not editable.
    #[props(default)]
    read_only: bool,
    /// Help text under the field.
    hint: Option<String>,
    /// Form field name. Submits minor units; the currency goes as `{name}_currency`.
    name: Option<String>,
    /// Placeholder while empty.
    placeholder: Option<String>,
    /// Called with the amount in minor units whenever it changes. `None` is empty.
    on_value_change: Option<EventHandler<Option<i64>>>,
    /// Called with the new currency code.
    on_currency_change: Option<EventHandler<String>>,
) -> Element {
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let ids = h::ids(&root);
    let hint_id = format!("{root}-hint");
    let input = use_memo(move || MoneyFieldInput {
        value: value(),
        default_value,
        currency: currency(),
        default_currency: default_currency.clone(),
        min,
        max,
        step,
        group,
        decimal,
        disabled: disabled(),
        read_only,
    });
    let mut state = use_signal(|| MoneyFieldMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = MoneyFieldMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });

    let control_id = ids.control.clone();
    let send = move |event: MoneyFieldEvent| {
        let step = MoneyFieldMachine::transition(&state.peek(), event, &input.peek());
        let mut state = state;
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                MoneyFieldCommand::ValueChange(v) => {
                    if let Some(handler) = on_value_change {
                        handler.call(v);
                    }
                }
                MoneyFieldCommand::CurrencyChange(code) => {
                    if let Some(handler) = on_currency_change {
                        handler.call(code);
                    }
                }
            }
        }
    };
    let place = {
        let id = control_id.clone();
        move || {
            let (draft, caret) = {
                let s = state.peek();
                (s.draft.clone(), s.caret)
            };
            write_back(&id, &draft, caret);
        }
    };
    let mut pasting = use_signal(|| false);

    let s = state.read();
    let i = input.read();
    let a = h::control_attrs(&s, &i);
    let is_disabled = i.disabled;
    let options = h::currency_options(&currencies);
    let pickable = options.len() > 1;
    let current = s.currency.clone();
    let symbol = h::symbol(&s.currency).to_string();
    let draft = s.draft.clone();
    let mode = h::inputmode(&s);
    let form = h::form_value(&s);
    drop(i);
    drop(s);
    let currency_name = name.as_ref().map(|n| format!("{n}_currency"));
    let described = hint.as_ref().map(|_| hint_id.clone());
    let (place_input, place_key) = (place.clone(), place);

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-disabled": is_disabled.then_some(""),
            "data-readonly": read_only.then_some(""),
            label { id: "{ids.label}", "data-scope": h::SCOPE, "data-part": "label", r#for: "{ids.control}", "{label}" }
            div { "data-scope": h::SCOPE, "data-part": "field",
                if pickable {
                    span { "data-scope": h::SCOPE, "data-part": "currency-wrap",
                        select {
                            id: "{ids.currency}",
                            "data-scope": h::SCOPE,
                            "data-part": "currency",
                            "aria-label": "Currency",
                            name: currency_name.clone(),
                            disabled: is_disabled || read_only,
                            value: "{current}",
                            onchange: move |e| send(MoneyFieldEvent::SetCurrency(e.value())),
                            for (code, text) in options {
                                option { key: "{code}", value: code, selected: code == current, "{text}" }
                            }
                        }
                        span { "data-scope": h::SCOPE, "data-part": "currency-code", "aria-hidden": "true", "{current}" }
                        Icon { name: IconName::ChevronDown, size: 14 }
                    }
                } else {
                    span { "data-scope": h::SCOPE, "data-part": "currency-code", "data-static": "",
                        "{current}"
                        if let Some(n) = currency_name {
                            input { r#type: "hidden", name: n, value: "{current}" }
                        }
                    }
                }
                span { "data-scope": h::SCOPE, "data-part": "symbol", "aria-hidden": "true", "{symbol}" }
                input {
                    r#type: "text",
                    id: "{ids.control}",
                    "data-scope": h::SCOPE,
                    "data-part": "control",
                    role: "spinbutton",
                    inputmode: mode,
                    autocomplete: "off",
                    spellcheck: "false",
                    "aria-valuenow": a.valuenow,
                    "aria-valuemin": a.valuemin,
                    "aria-valuemax": a.valuemax,
                    "aria-valuetext": a.valuetext,
                    "aria-describedby": described,
                    placeholder,
                    disabled: is_disabled,
                    readonly: read_only,
                    value: "{draft}",
                    onpaste: move |_| pasting.set(true),
                    oninput: move |e| {
                        let text = e.value();
                        if pasting() {
                            pasting.set(false);
                            send(MoneyFieldEvent::Paste(text));
                            place_input();
                            return;
                        }
                        let id = control_id.clone();
                        let place = place_input.clone();
                        spawn(async move {
                            let caret = read_caret(&id).await;
                            send(MoneyFieldEvent::SetText { text, caret });
                            place();
                        });
                    },
                    onkeydown: move |e| {
                        let event = h::keydown(&e.key().to_string(), e.modifiers().shift(), &input.peek());
                        if let Some(event) = event {
                            e.prevent_default();
                            send(event);
                            place_key();
                        }
                    },
                    onblur: move |_| send(MoneyFieldEvent::Commit),
                }
            }
            if let Some(t) = hint {
                p { id: "{hint_id}", "data-scope": h::SCOPE, "data-part": "hint", "{t}" }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name: "{name}", disabled: is_disabled, value: form }
            }
        }
    }
}
