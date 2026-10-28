use dioxus::prelude::*;
use hydroxy_dioxus::phone_field::{PhoneField, PhoneValue};

#[component]
pub fn PhoneFieldDemo() -> Element {
    let mut phone = use_signal(|| PhoneValue { e164: None, valid: false });
    let readout = match phone() {
        PhoneValue { e164: Some(n), valid: true } => format!("{n} looks valid"),
        PhoneValue { e164: Some(n), valid: false } => format!("{n} is incomplete"),
        PhoneValue { e164: None, .. } => "Nothing entered".to_string(),
    };

    rsx! {
        div { style: "display: grid; gap: var(--space-8); max-width: 360px;",
            div { style: "display: grid; gap: var(--space-2);",
                PhoneField {
                    label: "Mobile number",
                    default_country: "GB",
                    hint: "We text a code to this number. Type +33… to switch country.",
                    name: "mobile",
                    on_value_change: move |v| phone.set(v),
                }
                p { style: "margin: 0; font: var(--text-xs); color: var(--text-3);", "{readout}" }
            }
            PhoneField { label: "Support line", default_value: "+12015550123", read_only: true }
            PhoneField { label: "Fax", default_country: "JP", disabled: true }
        }
    }
}
