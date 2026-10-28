use dioxus::prelude::*;
use hydroxy_dioxus::combobox::{Combobox, ComboboxOption};

#[component]
pub fn ComboboxDemo() -> Element {
    let mut owner = use_signal(|| Some("jm".to_string()));
    let people = vec![
        ComboboxOption::new("jm", "Jo Marsh"),
        ComboboxOption::new("ak", "Amara Kaur"),
        ComboboxOption::new("rt", "Rui Tavares"),
        ComboboxOption::new("lb", "Lena Brandt").disabled(),
        ComboboxOption::new("sn", "Sam Novak"),
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-7); max-width: 360px;",
            // Controlled. Type to filter, arrows to move, Enter to choose, Esc to close or clear.
            Combobox {
                label: "Project owner",
                options: people,
                value: owner(),
                placeholder: "Search people",
                on_value_change: move |v| owner.set(v),
            }
            span { "Owner: {owner().unwrap_or_else(|| \"nobody\".into())}" }

            // Uncontrolled, submitted with a form as `region`.
            Combobox {
                label: "Data region",
                name: "region",
                default_value: "eu",
                options: vec![
                    ComboboxOption::new("eu", "Europe (Frankfurt)"),
                    ComboboxOption::new("uk", "United Kingdom (London)"),
                    ComboboxOption::new("us", "United States (Virginia)"),
                    ComboboxOption::new("ap", "Asia Pacific (Sydney)"),
                ],
            }
        }
    }
}
