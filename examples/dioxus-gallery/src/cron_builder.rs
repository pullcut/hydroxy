use dioxus::prelude::*;
use hydroxy_dioxus::cron_builder::CronBuilder;

#[component]
pub fn CronBuilderDemo() -> Element {
    let mut saved = use_signal(|| String::from("0 9 * * 1-5"));

    rsx! {
        div { style: "display: grid; gap: var(--space-3);",
            CronBuilder { label: "Digest email schedule", value: Some(saved()), on_change: move |v| saved.set(v) }
            p { style: "margin: 0; font: var(--text-xs); color: var(--text-3);",
                "Saved: "
                code { "{saved}" }
                " · try typing "
                code { "*/20 8-18 * * MON-FRI" }
            }
        }
    }
}
