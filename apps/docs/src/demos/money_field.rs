use hydroxy_leptos::money_field::MoneyField;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    // Minor units: 249_900 is 2,499.00.
    let (price, set_price) = signal(Some(249_900_i64));
    let (code, set_code) = signal("USD".to_string());
    let summary = move || match price.get() {
        Some(v) => format!("{v} minor units of {}", code.get()),
        None => "No price yet".to_string(),
    };

    view! {
        <div style="display: grid; gap: var(--space-8); max-width: 360px;">
            // Controlled amount and currency. Switching currency keeps the amount
            // and re-rounds it: JPY has no minor unit, BHD has three.
            <div style="display: grid; gap: var(--space-2);">
                <MoneyField
                    label="Listing price"
                    value=price
                    currency=code
                    currencies=vec!["USD".into(), "EUR".into(), "GBP".into(), "JPY".into(), "BHD".into()]
                    min=0
                    max=100_000_000
                    hint="Paste \"$1,234.50\" or \"1.234,50 €\" to try the parser"
                    on_value_change=move |v| set_price.set(v)
                    on_currency_change=move |c| set_code.set(c)
                />
                <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">{summary}</p>
            </div>

            // European separators, a single fixed currency, and a form name.
            <MoneyField
                label="Monatsbudget"
                default_currency="EUR"
                currencies=vec!["EUR".into()]
                default_value=120_000
                group='.'
                decimal=','
                step=5_000
                name="budget"
            />

            <MoneyField label="Refund issued" default_currency="GBP" default_value=-4_250 read_only=true />
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "money-field",
    name: "Money field",
    group: "Forms",
    summary: "An amount with a currency. Groups thousands as you type, stops at the currency's precision, reads pasted amounts, and reports integer minor units.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("money_field.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/money_field.rs"),
    parts: &[
        &["MoneyField", "root", "Wrapper. data-disabled, data-readonly"],
        &["MoneyField", "label", "<label for> the amount"],
        &["MoneyField", "field", "Bordered row: currency, symbol, amount"],
        &["MoneyField", "currency-wrap", "Visible currency code and chevron over the native select"],
        &["MoneyField", "currency", "Native <select> named \"Currency\""],
        &["MoneyField", "currency-code", "The code; data-static when there is only one currency"],
        &["MoneyField", "symbol", "Currency symbol, aria-hidden"],
        &["MoneyField", "control", "Text input, role=spinbutton, aria-valuetext reads the money"],
        &["MoneyField", "hint", "Help text, linked by aria-describedby"],
    ],
    keys: &[
        &["ArrowUp / ArrowDown", "Step one major unit, or by step"],
        &["Shift+Arrow, PageUp / PageDown", "Step ten at a time"],
        &["Home / End", "Minimum / maximum, when set"],
        &["Enter", "Clamp to the range and pad the fraction"],
    ],
    props: &[
        &["label", "String", "Required. Visible label"],
        &["value / default_value", "Option<i64>", "Amount in minor units (cents, yen, fils)"],
        &["currency / default_currency", "String", "ISO 4217 code. Default USD"],
        &["currencies", "Vec<String>", "Codes in the picker. Empty: all; one: no picker"],
        &["min / max / step", "Option<i64>", "Minor units. min >= 0 blocks the minus sign"],
        &["group / decimal", "char", "Separators. Default , and ."],
        &["on_value_change", "Callback<Option<i64>>", "Every change, in minor units"],
        &["on_currency_change", "Callback<String>", "The new code"],
        &["disabled / read_only", "bool", "Block changes"],
        &["hint / placeholder", "String", "Help text and empty text"],
        &["name", "String", "Submits minor units, plus {name}_currency"],
    ],
    events: &[],
};
