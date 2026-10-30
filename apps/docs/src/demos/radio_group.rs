use hydroxy_leptos::radio_group::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let (value, set_value) = signal(Some("weekly".to_string()));

    view! {
        // Controlled, with descriptions and a disabled option.
        <div style="display: grid; gap: var(--space-4);">
            <RadioGroup
                items=vec![RadioItem::new("daily"), RadioItem::new("weekly"), RadioItem::new("monthly").disabled()]
                value=value
                on_value_change=move |v| set_value.set(Some(v))
                name="digest"
                label="Digest frequency"
            >
                <RadioGroupItem value="daily" label="Daily" description="A summary every morning at 8:00." />
                <RadioGroupItem value="weekly" label="Weekly" description="One summary on Monday." />
                <RadioGroupItem value="monthly" label="Monthly" description="Available on the team plan." />
            </RadioGroup>
            <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">
                "Selected: " {move || value.get().unwrap_or_default()}
            </p>
        </div>

        // Uncontrolled and horizontal.
        <RadioGroup
            items=vec![RadioItem::new("light"), RadioItem::new("dark"), RadioItem::new("system")]
            default_value="system"
            orientation=Orientation::Horizontal
            label="Theme"
        >
            <RadioGroupItem value="light" label="Light" />
            <RadioGroupItem value="dark" label="Dark" />
            <RadioGroupItem value="system" label="System" />
        </RadioGroup>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "radio-group",
    name: "Radio group",
    group: "Forms",
    summary: "Pick one option from a short list. Arrow keys move between enabled options and select as they go.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("radio_group.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/radio_group.rs"),
    parts: &[
        &["RadioGroup", "root", "role=radiogroup, aria-label, aria-orientation, data-orientation"],
        &["RadioGroupItem", "item", "role=radio, aria-checked, data-state=checked|unchecked, disabled"],
        &["RadioGroupItem", "control", "The circle. aria-hidden"],
        &["RadioGroupItem", "label", "Names the option via aria-labelledby"],
        &["RadioGroupItem", "description", "Optional hint via aria-describedby"],
    ],
    keys: &[
        &["ArrowDown / ArrowRight", "Next enabled option, and selects it"],
        &["ArrowUp / ArrowLeft", "Previous enabled option, and selects it"],
        &["Space", "Selects the focused option"],
        &["Tab", "Moves into the group at the selected option, then out of it"],
    ],
    props: &[
        &["items", "Vec<RadioItem>", "Required. Order and disabled state"],
        &["value", "Option<String>", "Controlled value. Pair with on_value_change"],
        &["default_value", "String", "Initial value when uncontrolled"],
        &["disabled", "bool", "Disables the whole group"],
        &["orientation", "Orientation", "Vertical (default) or Horizontal. Layout only"],
        &["loop_focus", "bool", "Wrap at the ends. Default true"],
        &["name", "String", "Submits the selected value with a form"],
        &["label", "String", "Accessible name for the group"],
        &["on_value_change", "Callback<String>", "Called when the user selects an option"],
    ],
    events: &[],
};
