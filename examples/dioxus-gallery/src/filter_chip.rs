use dioxus::prelude::*;
use oxide_dioxus::filter_chip::FilterChip;
use oxide_dioxus::icon::IconName;

#[component]
pub fn FilterChipDemo() -> Element {
    let mut overdue = use_signal(|| true);
    let mut mine = use_signal(|| false);
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: var(--space-3);",
            FilterChip { label: "Status", count: Some(2), active: true, dropdown: true }
            FilterChip { label: "Owner", dropdown: true }
            FilterChip {
                label: "Overdue",
                icon: IconName::CircleAlert,
                active: overdue(),
                on_click: move |()| overdue.toggle(),
            }
            FilterChip { label: "Mine", icon: IconName::User, active: mine(), on_click: move |()| mine.toggle() }
        }
    }
}
