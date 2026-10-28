//! `<MoneyField>`: an amount with a currency picker, held as integer minor units.

use leptos::prelude::*;
use hydroxy_headless::money_field::{self as h, MoneyFieldCommand, MoneyFieldEvent, MoneyFieldInput, MoneyFieldMachine};
use hydroxy_statechart::Machine;

use crate::icon::{Icon, IconName};

/// An amount in a currency. Typing groups thousands as you go and stops at the
/// currency's precision; pasting reads `$1,234.50` or `1.234,50 €`; Enter or blur
/// clamps to `min`..`max`. Arrows step one major unit (Shift or PageUp/PageDown: ten).
/// Values are minor units: cents, yen, fils.
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value, reason = "one view! tree; props are owned by the macro")]
#[component]
pub fn MoneyField(
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Controlled amount in minor units. Pair with `on_value_change`.
    #[prop(optional, into)]
    value: MaybeProp<i64>,
    /// Initial amount in minor units when uncontrolled.
    #[prop(optional, into)]
    default_value: Option<i64>,
    /// Controlled currency code. Pair with `on_currency_change`.
    #[prop(optional, into)]
    currency: MaybeProp<String>,
    /// Currency to start in when uncontrolled. Default USD.
    #[prop(default = "USD".into(), into)]
    default_currency: String,
    /// Codes offered in the picker. Empty offers every known currency; one code hides the picker.
    #[prop(optional, into)]
    currencies: Vec<String>,
    /// Lowest amount in minor units. Zero or more also blocks the minus sign.
    #[prop(optional, into)]
    min: Option<i64>,
    /// Highest amount in minor units.
    #[prop(optional, into)]
    max: Option<i64>,
    /// Arrow step in minor units. Default one major unit.
    #[prop(optional, into)]
    step: Option<i64>,
    /// Thousands separator. Default `,`.
    #[prop(default = ',')]
    group: char,
    /// Decimal mark. Default `.`.
    #[prop(default = '.')]
    decimal: char,
    /// Block changes and drop out of the tab order.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Focusable but not editable.
    #[prop(optional)]
    read_only: bool,
    /// Help text under the field.
    #[prop(optional, into)]
    hint: Option<String>,
    /// Form field name. Submits minor units; the currency goes as `{name}_currency`.
    #[prop(optional, into)]
    name: Option<String>,
    /// Placeholder while empty.
    #[prop(optional, into)]
    placeholder: Option<String>,
    /// Called with the amount in minor units whenever it changes. `None` is empty.
    #[prop(optional, into)]
    on_value_change: Option<Callback<Option<i64>>>,
    /// Called with the new currency code.
    #[prop(optional, into)]
    on_currency_change: Option<Callback<String>>,
) -> impl IntoView {
    let root = crate::id::next_id(h::SCOPE);
    let ids = h::ids(&root);
    let hint_id = format!("{root}-hint");
    let input = Memo::new(move |_| MoneyFieldInput {
        value: value.get(),
        default_value,
        currency: currency.get(),
        default_currency: default_currency.clone(),
        min,
        max,
        step,
        group,
        decimal,
        disabled: disabled.get().unwrap_or(false),
        read_only,
    });
    let state = RwSignal::new(input.with_untracked(MoneyFieldMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| MoneyFieldMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });

    let control = NodeRef::<leptos::html::Input>::new();
    let send = move |event: MoneyFieldEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| MoneyFieldMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                MoneyFieldCommand::ValueChange(v) => {
                    if let Some(cb) = on_value_change {
                        cb.run(v);
                    }
                }
                MoneyFieldCommand::CurrencyChange(code) => {
                    if let Some(cb) = on_currency_change {
                        cb.run(code);
                    }
                }
            }
        }
    };
    // Write the reformatted text back and keep the caret on the same digit. The text is
    // all BMP characters, so char indices are UTF-16 offsets.
    let place_caret = move || {
        let Some(el) = control.get_untracked() else { return };
        let (draft, caret) = state.with_untracked(|s| (s.draft.clone(), s.caret));
        el.set_value(&draft);
        let at = u32::try_from(caret).unwrap_or(u32::MAX);
        let _ = el.set_selection_range(at, at);
    };
    let pasting = StoredValue::new(false);

    let attrs = Memo::new(move |_| input.with(|i| state.with(|s| h::control_attrs(s, i))));
    let is_disabled = move || disabled.get().unwrap_or(false);
    let options = h::currency_options(&currencies);
    let pickable = options.len() > 1;
    let symbol = move || state.with(|s| h::symbol(&s.currency).to_string());
    let currency_name = name.as_ref().map(|n| format!("{n}_currency"));
    let described = hint.is_some().then(|| hint_id.clone());

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-disabled=move || is_disabled().then_some("")
            data-readonly=read_only.then_some("")
        >
            <label id=ids.label.clone() data-scope=h::SCOPE data-part="label" for=ids.control.clone()>
                {label}
            </label>
            <div data-scope=h::SCOPE data-part="field">
                {if pickable {
                    leptos::either::Either::Left(view! {
                        <span data-scope=h::SCOPE data-part="currency-wrap">
                            <select
                                id=ids.currency.clone()
                                data-scope=h::SCOPE
                                data-part="currency"
                                aria-label="Currency"
                                name=currency_name
                                disabled=move || is_disabled() || read_only
                                prop:value=move || state.with(|s| s.currency.clone())
                                on:change=move |ev| {
                                    send(MoneyFieldEvent::SetCurrency(event_target_value(&ev)));
                                }
                            >
                                {options
                                    .into_iter()
                                    .map(|(code, text)| {
                                        view! {
                                            <option value=code selected=move || state.with(|s| s.currency == code)>
                                                {text}
                                            </option>
                                        }
                                    })
                                    .collect_view()}
                            </select>
                            <span data-scope=h::SCOPE data-part="currency-code" aria-hidden="true">
                                {move || state.with(|s| s.currency.clone())}
                            </span>
                            <Icon name=IconName::ChevronDown size=14 />
                        </span>
                    })
                } else {
                    leptos::either::Either::Right(view! {
                        <span data-scope=h::SCOPE data-part="currency-code" data-static="">
                            {move || state.with(|s| s.currency.clone())}
                            {currency_name.map(|n| view! { <input type="hidden" name=n prop:value=move || state.with(|s| s.currency.clone()) /> })}
                        </span>
                    })
                }}
                <span data-scope=h::SCOPE data-part="symbol" aria-hidden="true">{symbol}</span>
                <input
                    node_ref=control
                    type="text"
                    id=ids.control.clone()
                    data-scope=h::SCOPE
                    data-part="control"
                    role="spinbutton"
                    inputmode=move || state.with(h::inputmode)
                    autocomplete="off"
                    spellcheck="false"
                    aria-valuenow=move || attrs.get().valuenow
                    aria-valuemin=move || attrs.get().valuemin
                    aria-valuemax=move || attrs.get().valuemax
                    aria-valuetext=move || attrs.get().valuetext
                    aria-describedby=described
                    placeholder=placeholder
                    disabled=is_disabled
                    readonly=read_only
                    prop:value=move || state.with(|s| s.draft.clone())
                    on:paste=move |_| pasting.set_value(true)
                    on:input=move |_| {
                        let Some(el) = control.get_untracked() else { return };
                        let text = el.value();
                        if pasting.get_value() {
                            pasting.set_value(false);
                            send(MoneyFieldEvent::Paste(text));
                        } else {
                            let caret = el.selection_start().ok().flatten().map_or(usize::MAX, |n| n as usize);
                            send(MoneyFieldEvent::SetText { text, caret });
                        }
                        place_caret();
                    }
                    on:keydown=move |ev| {
                        let event = input.try_with_untracked(|i| h::keydown(&ev.key(), ev.shift_key(), i)).flatten();
                        if let Some(event) = event {
                            ev.prevent_default();
                            send(event);
                            place_caret();
                        }
                    }
                    on:blur=move |_| send(MoneyFieldEvent::Commit)
                />
            </div>
            {hint.map(|t| view! { <p id=hint_id data-scope=h::SCOPE data-part="hint">{t}</p> })}
            {name.map(|name| view! { <input type="hidden" name=name disabled=is_disabled prop:value=move || state.with(h::form_value) /> })}
        </div>
    }
}
