use hydroxy_leptos::select::{Select, SelectOption};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (theme, set_theme) = signal(String::from("system"));
    let themes = vec![
        SelectOption::new("system", "Match the system"),
        SelectOption::new("light", "Light"),
        SelectOption::new("dark", "Dark"),
    ];
    let (region, set_region) = signal(String::new());

    view! {
        <div style="display: grid; gap: var(--space-7); max-width: 360px;">
            // Controlled.
            <Select
                label="Default theme"
                options=themes
                value=theme
                on_change=move |v| set_theme.set(v)
                hint="New projects start in this theme."
            />
            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">"Theme: " {theme}</p>

            // Uncontrolled, with validation.
            <Select
                label="Data region"
                options=vec![
                    SelectOption::new("", "Choose a region"),
                    SelectOption::new("eu", "Europe (Frankfurt)"),
                    SelectOption::new("us", "United States (Virginia)"),
                ]
                on_change=move |v| set_region.set(v)
                error=Signal::derive(move || region.get().is_empty().then(|| "Choose where files are stored.".to_string()))
                required=true
                name="region"
            />

            <Select
                label="Billing currency"
                options=vec![SelectOption::new("gbp", "GBP (£)")]
                disabled=true
                hint="Set by your workspace owner."
            />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "select",
    name: "Select",
    group: "Forms",
    summary: "A native dropdown styled to match Input. The platform handles the picker, typeahead and keyboard.",
    demo,
    leptos: include_str!("select.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/select.rs"),
    parts: &[
        &["Select", "root", "Wrapper. data-invalid, data-disabled"],
        &["Select", "label", "<label for> the control"],
        &["Select", "field", "Positions the chevron over the control"],
        &["Select", "control", "Native <select>. aria-describedby the hint or error, aria-invalid on error"],
        &["Select", "icon", "Chevron. Ignores the pointer, so clicks reach the select"],
        &["Select", "hint", "Help text. Hidden while there is an error"],
        &["Select", "error", "Error text"],
    ],
    keys: &[
        &["Space / Enter / Alt+ArrowDown", "Opens the platform picker"],
        &["ArrowUp / ArrowDown", "Moves through the options"],
        &["Letters", "Jumps to the next option starting with them"],
    ],
    props: &[
        &["label", "String", "Required. Visible label"],
        &["options", "Vec<SelectOption>", "Required. SelectOption::new(value, label)"],
        &["value", "Option<String>", "Controlled value. Pair with on_change"],
        &["default_value", "String", "Initial value when uncontrolled. Defaults to the first option"],
        &["on_change", "Callback<String>", "Called with the chosen value"],
        &["hint", "Option<String>", "Help text under the field"],
        &["error", "Option<String>", "Replaces the hint and marks the field invalid"],
        &["disabled", "bool", "Block changes"],
        &["name", "String", "Form field name"],
        &["required", "bool", "Native required"],
    ],
};
