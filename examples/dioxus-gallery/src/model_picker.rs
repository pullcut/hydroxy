use dioxus::prelude::*;
use hydroxy_dioxus::model_picker::{Effort, ModelOption, ModelPicker, ModelSettings};

fn models() -> Vec<ModelOption> {
    vec![
        ModelOption::new("opus", "Opus", "Deep reasoning for long, multi-step work", 1_000_000, 128_000)
            .tags(&["Most capable"]),
        ModelOption::new("sonnet", "Sonnet", "Fast and strong for everyday coding and writing", 1_000_000, 64_000)
            .tags(&["Balanced"]),
        ModelOption::new("haiku", "Haiku", "Quick, low-cost replies and classification", 200_000, 8_192)
            .tags(&["Fast", "Cheap"]),
    ]
}

#[component]
pub fn ModelPickerDemo() -> Element {
    let mut settings = use_signal(|| ModelSettings {
        model: "sonnet".into(),
        temperature: 0.7,
        max_tokens: 16_000,
        effort: Effort::Medium,
    });
    let s = settings();
    let line = format!(
        "model={} temperature={:.2} max_tokens={} effort={}",
        s.model,
        s.temperature,
        s.max_tokens,
        s.effort.as_str()
    );

    rsx! {
        div { style: "display: grid; gap: var(--space-3); justify-items: start;",
            ModelPicker { models: models(), value: Some(s), on_change: move |next| settings.set(next) }
            p { style: "margin: 0; font: var(--text-xs); color: var(--text-3); font-family: var(--font-mono);", "{line}" }
        }
    }
}
