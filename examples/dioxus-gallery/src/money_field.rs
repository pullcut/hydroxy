use dioxus::prelude::*;
use hydroxy_dioxus::money_field::MoneyField;

#[component]
pub fn MoneyFieldDemo() -> Element {
    // Minor units: 249_900 is 2,499.00.
    let mut price = use_signal(|| Some(249_900_i64));
    let mut code = use_signal(|| Some("USD".to_string()));
    let summary = match price() {
        Some(v) => format!("{v} minor units of {}", code().unwrap_or_default()),
        None => "No price yet".to_string(),
    };

    rsx! {
        div { style: "display: grid; gap: var(--space-8); max-width: 360px;",
            div { style: "display: grid; gap: var(--space-2);",
                MoneyField {
                    label: "Listing price",
                    value: price(),
                    currency: code(),
                    currencies: vec!["USD".into(), "EUR".into(), "GBP".into(), "JPY".into(), "BHD".into()],
                    min: 0,
                    max: 100_000_000,
                    hint: "Paste \"$1,234.50\" or \"1.234,50 €\" to try the parser",
                    on_value_change: move |v| price.set(v),
                    on_currency_change: move |c| code.set(Some(c)),
                }
                p { style: "margin: 0; font: var(--text-xs); color: var(--text-3);", "{summary}" }
            }
            MoneyField {
                label: "Monatsbudget",
                default_currency: "EUR",
                currencies: vec!["EUR".into()],
                default_value: 120_000,
                group: '.',
                decimal: ',',
                step: 5_000,
                name: "budget",
            }
            MoneyField { label: "Refund issued", default_currency: "GBP", default_value: -4_250, read_only: true }
        }
    }
}
