use dioxus::prelude::*;
use hydroxy_dioxus::tabs::{Tabs, TabsContent, TabsItem, TabsList, TabsTrigger};

#[component]
pub fn TabsDemo() -> Element {
    let mut value = use_signal(|| Some("overview".to_string()));
    let tabs = vec![
        TabsItem::new("overview"),
        TabsItem::new("sources"),
        TabsItem::new("archived").disabled(),
        TabsItem::new("activity"),
    ];

    rsx! {
        p { "Selected: {value().unwrap_or_default()}" }
        Tabs {
            tabs,
            value: value(),
            on_value_change: move |v| value.set(Some(v)),
            TabsList {
                TabsTrigger { value: "overview", "Overview" }
                TabsTrigger { value: "sources", "Sources" }
                TabsTrigger { value: "archived", "Archived" }
                TabsTrigger { value: "activity", "Activity" }
            }
            TabsContent { value: "overview", "Nine of twelve interviewees raised onboarding." }
            TabsContent { value: "sources", "interviews-q3.pdf, tickets-sept.csv" }
            TabsContent { value: "archived", "Nothing archived." }
            TabsContent { value: "activity", "Summary drafted 26 October." }
        }
    }
}
