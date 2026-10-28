use dioxus::prelude::*;
use hydroxy_dioxus::notification_center::{Notification, NotificationCenter};

#[component]
pub fn NotificationCenterDemo() -> Element {
    let mut opened = use_signal(|| String::from("none yet"));

    rsx! {
        div { style: "display: flex; align-items: center; gap: var(--space-5);",
            // Uncontrolled. Opening a notification marks it read; dismissing removes it.
            NotificationCenter {
                default_items: vec![
                    Notification::new("deploy", "Deploy finished", "2m ago").description("Production is on build 482."),
                    Notification::new("comment", "Maya commented on Q3 summary", "18m ago")
                        .description("\"Can we add the churn numbers?\""),
                    Notification::new("invite", "Lee accepted your invite", "1h ago"),
                    Notification::new("export", "Export is ready", "Yesterday").read(),
                ],
                on_select: move |id| opened.set(id),
            }
            span { "Last opened: {opened}" }
        }
    }
}
