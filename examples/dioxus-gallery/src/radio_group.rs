use dioxus::prelude::*;
use hydroxy_dioxus::radio_group::{Orientation, RadioGroup, RadioGroupItem, RadioItem};

#[component]
pub fn RadioGroupDemo() -> Element {
    let mut value = use_signal(|| Some("weekly".to_string()));

    rsx! {
        // Controlled, with descriptions and a disabled option.
        RadioGroup {
            items: vec![RadioItem::new("daily"), RadioItem::new("weekly"), RadioItem::new("monthly").disabled()],
            value: value(),
            on_value_change: move |v| value.set(Some(v)),
            name: "digest",
            label: "Digest frequency",
            RadioGroupItem { value: "daily", label: "Daily", description: "A summary every morning at 8:00." }
            RadioGroupItem { value: "weekly", label: "Weekly", description: "One summary on Monday." }
            RadioGroupItem { value: "monthly", label: "Monthly", description: "Available on the team plan." }
        }
        p { "Selected: {value().unwrap_or_default()}" }

        // Uncontrolled and horizontal.
        RadioGroup {
            items: vec![RadioItem::new("light"), RadioItem::new("dark"), RadioItem::new("system")],
            default_value: "system",
            orientation: Orientation::Horizontal,
            label: "Theme",
            RadioGroupItem { value: "light", label: "Light" }
            RadioGroupItem { value: "dark", label: "Dark" }
            RadioGroupItem { value: "system", label: "System" }
        }
    }
}
