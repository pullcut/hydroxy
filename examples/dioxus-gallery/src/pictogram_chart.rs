use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::pictogram_chart::{Datum, PictogramChart, PictogramIcon};

/// A house, drawn in a 24×24 box.
const HOUSE: &str = "M12 3.2l8.5 7.3V20a1 1 0 0 1-1 1h-5v-6h-5v6h-5a1 1 0 0 1-1-1v-9.5z";

#[component]
pub fn PictogramChartDemo() -> Element {
    let visitors = vec![
        Datum::new("Science museum", 18_400.0),
        Datum::new("Botanic garden", 12_750.0),
        Datum::new("City gallery", 9_300.0),
        Datum::new("Maritime museum", 4_150.0),
    ];
    let homes = vec![Datum::new("North", 1_240.0), Datum::new("Central", 2_180.0), Datum::new("South", 860.0)];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Weekend visitors", description: "Tickets scanned on the first weekend of May.",
                PictogramChart { data: visitors, label: "Weekend visitors by venue", unit: 1_000.0, unit_label: "visitors" }
            }
            ChartFrame { title: "New homes", description: "Completions this year, by region.",
                PictogramChart {
                    data: homes,
                    label: "New homes completed by region",
                    unit: 100.0,
                    unit_label: "homes",
                    icon: PictogramIcon::Custom(HOUSE.into()),
                    icon_size: 22,
                }
            }
        }
    }
}
