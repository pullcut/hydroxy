use dioxus::prelude::*;
use oxide_dioxus::select::{Select, SelectOption};

#[component]
pub fn SelectDemo() -> Element {
    let mut theme = use_signal(|| String::from("system"));
    let mut region = use_signal(String::new);
    let region_error = region().is_empty().then(|| "Choose where files are stored.".to_string());

    rsx! {
        div { style: "display: grid; gap: var(--space-7); max-width: 360px;",
            Select {
                label: "Default theme",
                options: vec![
                    SelectOption::new("system", "Match the system"),
                    SelectOption::new("light", "Light"),
                    SelectOption::new("dark", "Dark"),
                ],
                value: theme(),
                on_change: move |v| theme.set(v),
                hint: "New projects start in this theme.",
            }
            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);", "Theme: {theme}" }
            Select {
                label: "Data region",
                options: vec![
                    SelectOption::new("", "Choose a region"),
                    SelectOption::new("eu", "Europe (Frankfurt)"),
                    SelectOption::new("us", "United States (Virginia)"),
                ],
                on_change: move |v| region.set(v),
                error: region_error,
                required: true,
                name: "region",
            }
            Select {
                label: "Billing currency",
                options: vec![SelectOption::new("gbp", "GBP (£)")],
                disabled: true,
                hint: "Set by your workspace owner.",
            }
        }
    }
}
