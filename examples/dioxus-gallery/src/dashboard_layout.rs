use dioxus::prelude::*;
use hydroxy_dioxus::dashboard_layout::{DashboardItem, DashboardLayout, Widget};
use hydroxy_dioxus::sparkline::Sparkline;
use hydroxy_dioxus::stat_card::{DeltaTone, Jewel, StatCard};

#[component]
pub fn DashboardLayoutDemo() -> Element {
    let mut last = use_signal(|| String::from("Drag a tile by its handle, or focus a handle and press Enter."));

    rsx! {
        div { style: "display: grid; gap: var(--space-4);",
            DashboardLayout {
                widgets: vec![
                    Widget::new("revenue", "Revenue", rsx! {
                        StatCard { label: "This month", value: "£48.2k", delta: Some("+8%".to_string()), delta_tone: DeltaTone::Up, jewel: Jewel::Emerald }
                    }),
                    Widget::new("signups", "Signups", rsx! {
                        StatCard { label: "This week", value: "312", delta: Some("−3%".to_string()), delta_tone: DeltaTone::Down, jewel: Jewel::Sapphire }
                    }),
                    Widget::new("churn", "Churn", rsx! {
                        StatCard { label: "Last 30 days", value: "1.9", unit: "%", caption: "steady", jewel: Jewel::Amethyst }
                    }),
                    Widget::new("trend", "Weekly active", rsx! {
                        Sparkline {
                            values: vec![12.0, 14.0, 13.0, 17.0, 16.0, 21.0, 24.0, 23.0, 28.0],
                            label: "Weekly active users",
                            width: 420,
                            height: 80,
                            area: true,
                        }
                    }),
                    Widget::new("notes", "Notes", rsx! {
                        p { style: "margin: 0;", "Resize a tile from its corner. Shift with the arrow keys resizes from the keyboard." }
                    }),
                ],
                default_items: vec![
                    DashboardItem::new("revenue", 0, 0, 4, 2),
                    DashboardItem::new("signups", 4, 0, 4, 2),
                    DashboardItem::new("churn", 8, 0, 4, 2),
                    DashboardItem::new("trend", 0, 2, 8, 3),
                    DashboardItem::new("notes", 8, 2, 4, 3),
                ],
                on_layout_change: move |items: Vec<DashboardItem>| {
                    last.set(
                        items.iter().map(|i| format!("{} ({}, {}) {}×{}", i.id, i.x, i.y, i.w, i.h)).collect::<Vec<_>>().join(" · "),
                    );
                },
            }
            p { style: "margin: 0; font: var(--text-label); color: var(--text-3);", "{last}" }
        }
    }
}
