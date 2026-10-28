use leptos::prelude::*;
use hydroxy_leptos::phone_field::{PhoneField, PhoneValue};

pub fn demo() -> AnyView {
    let (phone, set_phone) = signal(PhoneValue { e164: None, valid: false });
    let readout = move || {
        let p = phone.get();
        match p.e164 {
            Some(n) if p.valid => format!("{n} looks valid"),
            Some(n) => format!("{n} is incomplete"),
            None => "Nothing entered".to_string(),
        }
    };

    view! {
        <div style="display: grid; gap: var(--space-8); max-width: 360px;">
            <div style="display: grid; gap: var(--space-2);">
                <PhoneField
                    label="Mobile number"
                    default_country="GB"
                    hint="We text a code to this number. Type +33… to switch country."
                    name="mobile"
                    on_value_change=move |v| set_phone.set(v)
                />
                <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">{readout}</p>
            </div>

            <PhoneField label="Support line" default_value="+12015550123" read_only=true />

            <PhoneField label="Fax" default_country="JP" disabled=true />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "phone-field",
    name: "Phone field",
    group: "Forms",
    summary: "An international phone number: searchable country picker, grouping as you type, E.164 output and a validity check.",
    demo,
    leptos: include_str!("phone_field.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/phone_field.rs"),
    parts: &[
        &["PhoneField", "root", "Wrapper. data-disabled, data-readonly, data-invalid"],
        &["PhoneField", "label", "<label for> the number"],
        &["PhoneField", "field", "Bordered row and the picker's anchor"],
        &["PhoneField", "trigger", "Country button: flag, dial code. aria-haspopup=dialog"],
        &["PhoneField", "control", "type=tel input, autocomplete=tel-national"],
        &["PhoneField", "picker", "popover=auto, role=dialog"],
        &["PhoneField", "search", "role=combobox with aria-activedescendant"],
        &["PhoneField", "listbox / option", "Countries: flag, name, dial code"],
        &["PhoneField", "status", "Polite count of matching countries"],
        &["PhoneField", "hint / error", "Help text, or the error once the field is left invalid"],
    ],
    keys: &[
        &["ArrowDown / ArrowUp on the button", "Open the country list"],
        &["Type in the search", "Filter by name, ISO code or +dial code"],
        &["ArrowDown / ArrowUp", "Move through countries"],
        &["Enter", "Pick the highlighted country and return to the number"],
        &["Escape", "Close the list and return to the button"],
        &["+ in the number", "Type or paste an international number to switch country"],
    ],
    props: &[
        &["label", "String", "Required. Visible label"],
        &["value / default_value", "Option<String>", "E.164, such as +447400123456"],
        &["default_country", "String", "ISO code when there is no value. Default US"],
        &["on_value_change", "Callback<PhoneValue>", "{ e164, valid } on every change"],
        &["on_country_change", "Callback<String>", "ISO code"],
        &["hint", "String", "Help text, replaced by the error while invalid"],
        &["disabled / read_only / required", "bool", "Native states"],
        &["name", "String", "Submits E.164"],
    ],
};
