use leptos::prelude::*;
use oxide_leptos::model_picker::{Effort, ModelOption, ModelPicker, ModelSettings};

fn models() -> Vec<ModelOption> {
    vec![
        ModelOption::new("opus", "Opus", "Deep reasoning for long, multi-step work", 1_000_000, 128_000).tags(&["Most capable"]),
        ModelOption::new("sonnet", "Sonnet", "Fast and strong for everyday coding and writing", 1_000_000, 64_000).tags(&["Balanced"]),
        ModelOption::new("haiku", "Haiku", "Quick, low-cost replies and classification", 200_000, 8_192).tags(&["Fast", "Cheap"]),
    ]
}

pub fn demo() -> AnyView {
    let settings = RwSignal::new(ModelSettings {
        model: "sonnet".into(),
        temperature: 0.7,
        max_tokens: 16_000,
        effort: Effort::Medium,
    });

    view! {
        <div style="display: grid; gap: var(--space-3); justify-items: start;">
            <ModelPicker models=models() value=Signal::derive(move || Some(settings.get())) on_change=move |s| settings.set(s) />
            <p style="margin: 0; font: var(--text-xs); color: var(--text-3); font-family: var(--font-mono);">
                {move || {
                    let s = settings.get();
                    format!("model={} temperature={:.2} max_tokens={} effort={}", s.model, s.temperature, s.max_tokens, s.effort.as_str())
                }}
            </p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "model-picker",
    name: "Model picker",
    group: "Chat",
    summary: "A compact control for the model and how it runs: temperature, a token budget and reasoning effort.",
    demo,
    leptos: include_str!("model_picker.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/model_picker.rs"),
    parts: &[
        &["ModelPicker", "trigger", "Popover trigger showing the model name and effort"],
        &["ModelPicker", "panel", "Popover dialog content"],
        &["ModelPicker", "models", "role=radiogroup named \"Model\""],
        &["ModelPicker", "model", "role=radio card: name, tags, description, context and output limits"],
        &["ModelPicker", "settings", "Temperature Slider, Max tokens NumberField, Reasoning effort Segmented"],
    ],
    keys: &[
        &["Enter / Space", "On the trigger: open the panel"],
        &["Arrow keys", "In the model list: move and select"],
        &["Escape", "Close the panel and return to the trigger"],
    ],
    props: &[
        &["models", "Vec<ModelOption>", "Required. Id, name, description, context window, output limit, tags"],
        &["value", "Option<ModelSettings>", "Controlled settings. Pair with on_change"],
        &["default_value", "ModelSettings", "Initial settings when uncontrolled; an empty model picks the first"],
        &["on_change", "Callback<ModelSettings>", "The whole new settings after any change"],
    ],
};
