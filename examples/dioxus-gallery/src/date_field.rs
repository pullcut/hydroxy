use dioxus::prelude::*;
use hydroxy_dioxus::date_field::{Date, DateField, DateOrder};

#[component]
pub fn DateFieldDemo() -> Element {
    let mut born = use_signal(|| Date::new(1990, 4, 17));
    let shown = born().map_or_else(|| "Incomplete date".to_string(), |d| format!("Born {d}"));

    rsx! {
        div { style: "display: grid; gap: var(--space-7); max-width: 320px;",
            // Controlled. Type 17041990, or arrow through each part.
            DateField { label: "Date of birth", value: born(), on_change: move |d| born.set(d) }
            span { "{shown}" }

            // Uncontrolled, month first, nothing before 2026, submitted as `start`.
            DateField { label: "Start", order: DateOrder::Mdy, min: Date::new(2026, 1, 1), name: "start" }

            DateField { label: "Signed on", default_value: Date::new(2026, 3, 4).unwrap(), disabled: true }
        }
    }
}
