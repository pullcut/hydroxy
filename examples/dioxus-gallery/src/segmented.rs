use dioxus::prelude::*;
use oxide_dioxus::icon::IconName;
use oxide_dioxus::segmented::{Segmented, SegmentedItem, SegmentedOption, SegmentedSize};

#[component]
pub fn SegmentedDemo() -> Element {
    let mut length = use_signal(|| Some(String::from("short")));

    rsx! {
        div { style: "display: grid; gap: var(--space-7); justify-items: start;",
            Segmented {
                label: "Summary length",
                items: vec![SegmentedOption::new("short"), SegmentedOption::new("medium"), SegmentedOption::new("long")],
                value: length(),
                on_value_change: move |v| length.set(Some(v)),
                SegmentedItem { value: "short", label: "Short" }
                SegmentedItem { value: "medium", label: "Medium" }
                SegmentedItem { value: "long", label: "Long" }
            }
            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);",
                "Length: {length().unwrap_or_default()}"
            }
            Segmented {
                label: "View",
                items: vec![SegmentedOption::new("list"), SegmentedOption::new("cards"), SegmentedOption::new("board").disabled()],
                default_value: "list",
                size: SegmentedSize::Sm,
                name: "view",
                SegmentedItem { value: "list", label: "List", icon: IconName::FileText }
                SegmentedItem { value: "cards", label: "Cards", icon: IconName::Copy }
                SegmentedItem { value: "board", label: "Board", icon: IconName::Inbox }
            }
        }
    }
}
