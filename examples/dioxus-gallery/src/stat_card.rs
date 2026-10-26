use dioxus::prelude::*;
use oxide_dioxus::stat_card::{DeltaTone, Jewel, StatCard};

#[component]
pub fn StatCardDemo() -> Element {
    rsx! {
        div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: var(--space-5);",
            StatCard {
                label: "Interviews read",
                value: "22 of 24",
                delta: Some("+6".to_string()),
                delta_tone: DeltaTone::Up,
                caption: "this week",
                jewel: Jewel::Amethyst,
            }
            StatCard {
                label: "Median reading time",
                value: "11",
                unit: "min",
                delta: Some("−4 min".to_string()),
                delta_tone: DeltaTone::Up,
                caption: "vs last week",
                jewel: Jewel::Sapphire,
            }
            StatCard {
                label: "Open flags",
                value: "2",
                delta: Some("+1".to_string()),
                delta_tone: DeltaTone::Down,
                caption: "since Monday",
            }
        }
    }
}
