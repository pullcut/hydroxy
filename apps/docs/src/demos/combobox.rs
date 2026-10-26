use leptos::prelude::*;
use oxide_leptos::combobox::*;

pub fn demo() -> AnyView {
    let (owner, set_owner) = signal(Some("jm".to_string()));
    let people = vec![
        ComboboxOption::new("jm", "Jo Marsh"),
        ComboboxOption::new("ak", "Amara Kaur"),
        ComboboxOption::new("rt", "Rui Tavares"),
        ComboboxOption::new("lb", "Lena Brandt").disabled(),
        ComboboxOption::new("sn", "Sam Novak"),
    ];

    view! {
        <div style="display: grid; gap: var(--space-7); max-width: 360px;">
            // Controlled. Type to filter, arrows to move, Enter to choose, Esc to close or clear.
            <Combobox
                label="Project owner"
                options=people
                value=owner
                placeholder="Search people"
                on_value_change=move |v| set_owner.set(v)
            />
            <span>"Owner: " {move || owner.get().unwrap_or_else(|| "nobody".into())}</span>

            // Uncontrolled, submitted with a form as `region`.
            <Combobox
                label="Data region"
                name="region"
                default_value="eu"
                options=vec![
                    ComboboxOption::new("eu", "Europe (Frankfurt)"),
                    ComboboxOption::new("uk", "United Kingdom (London)"),
                    ComboboxOption::new("us", "United States (Virginia)"),
                    ComboboxOption::new("ap", "Asia Pacific (Sydney)"),
                ]
            />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "combobox",
    name: "Combobox",
    group: "Forms",
    summary: "A text field that filters a list of options as you type. Use it when the list is too long to scan.",
    demo,
    leptos: include_str!("combobox.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/combobox.rs"),
    parts: &[
        &["Combobox", "root, label", "Wrapper and visible label"],
        &[
            "(field)",
            "field, control, chevron",
            "The bordered field and CSS anchor. input role=combobox, aria-expanded, aria-controls, aria-activedescendant",
        ],
        &[
            "(list)",
            "listbox, option, empty",
            "role=listbox, popover=manual, anchored below and at least as wide as the field. role=option, aria-selected, aria-disabled, data-highlighted",
        ],
    ],
    keys: &[
        &["Typing", "Filters the options and opens the list"],
        &["ArrowDown / ArrowUp", "Opens the list, or moves through enabled options. Wraps at the ends"],
        &["Enter", "Chooses the highlighted option and closes"],
        &["Escape", "Closes the list. When already closed, clears the value"],
        &["Tab", "Closes and moves focus on, keeping the chosen option"],
    ],
    props: &[
        &["label", "String", "Required. The visible label"],
        &["options", "Vec<ComboboxOption>", "Required. Value, label and disabled state, in order"],
        &["value", "Option<String>", "Controlled value. Pair with on_value_change"],
        &["default_value", "String", "Initial value when uncontrolled"],
        &["placeholder", "String", "Shown while the field is empty"],
        &["name", "String", "Submits the chosen value with a form"],
        &["disabled", "bool", "Greyed out and not editable"],
        &["on_value_change", "Callback<Option<String>>", "Called with the chosen value, or None when cleared"],
    ],
};
