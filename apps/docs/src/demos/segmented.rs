use leptos::prelude::*;
use hydroxy_leptos::icon::IconName;
use hydroxy_leptos::segmented::{Segmented, SegmentedItem, SegmentedOption, SegmentedSize};

pub fn demo() -> AnyView {
    let (length, set_length) = signal(Some(String::from("short")));

    view! {
        <div style="display: grid; gap: var(--space-7); justify-items: start;">
            // Controlled.
            <Segmented
                label="Summary length"
                items=vec![SegmentedOption::new("short"), SegmentedOption::new("medium"), SegmentedOption::new("long")]
                value=length
                on_value_change=move |v| set_length.set(Some(v))
            >
                <SegmentedItem value="short" label="Short" />
                <SegmentedItem value="medium" label="Medium" />
                <SegmentedItem value="long" label="Long" />
            </Segmented>
            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">
                "Length: " {move || length.get().unwrap_or_default()}
            </p>

            // Uncontrolled, small, with icons and a disabled option.
            <Segmented
                label="View"
                items=vec![SegmentedOption::new("list"), SegmentedOption::new("cards"), SegmentedOption::new("board").disabled()]
                default_value="list"
                size=SegmentedSize::Sm
                name="view"
            >
                <SegmentedItem value="list" label="List" icon=IconName::FileText />
                <SegmentedItem value="cards" label="Cards" icon=IconName::Copy />
                <SegmentedItem value="board" label="Board" icon=IconName::Inbox />
            </Segmented>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "segmented",
    name: "Segmented control",
    group: "Forms",
    summary: "A pill row for switching between a few options. Behaves like a radio group: arrow keys move and select.",
    demo,
    leptos: include_str!("segmented.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/segmented.rs"),
    parts: &[
        &["Segmented", "root", "role=radiogroup, aria-label. data-size"],
        &["SegmentedItem", "item", "role=radio, aria-checked, roving tabindex. data-state=checked|unchecked"],
    ],
    keys: &[
        &["ArrowRight / ArrowDown", "Next enabled option, and select it"],
        &["ArrowLeft / ArrowUp", "Previous enabled option, and select it"],
        &["Space", "Selects the focused option"],
        &["Tab", "Into and out of the control (one stop)"],
    ],
    props: &[
        &["label", "String", "Required. Accessible name for the group"],
        &["items", "Vec<SegmentedOption>", "Required. Order and disabled state"],
        &["value", "Option<String>", "Controlled value. Pair with on_value_change"],
        &["default_value", "String", "Initial value when uncontrolled"],
        &["disabled", "bool", "Disable the whole control"],
        &["size", "SegmentedSize", "Md (default) or Sm"],
        &["name", "String", "Form field name; submits the selected value"],
        &["on_value_change", "Callback<String>", "Called when the selection changes"],
        &["SegmentedItem: value, label, icon", "String, String, IconName", "One option, with an optional leading icon"],
    ],
};
