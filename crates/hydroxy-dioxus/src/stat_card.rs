//! `StatCard`: one key figure with its name, change and caption, for dashboards.

use dioxus::prelude::*;

pub use hydroxy_headless::jewel::Jewel;

/// Which way the change reads.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum DeltaTone {
    /// Good news, in the success colour.
    Up,
    /// Bad news, in the danger colour.
    Down,
    /// Neither.
    #[default]
    Neutral,
}

impl DeltaTone {
    const fn as_str(self) -> &'static str {
        match self {
            DeltaTone::Up => "up",
            DeltaTone::Down => "down",
            DeltaTone::Neutral => "neutral",
        }
    }
}

/// A key figure. Lead with the number: "22 of 24", "11 min".
#[component]
pub fn StatCard(
    /// What the figure measures, in sentence case.
    label: String,
    /// The figure.
    value: ReadSignal<String>,
    /// Unit after the figure, such as "min".
    unit: Option<String>,
    /// The change, such as "−4 min" or "+12%".
    #[props(default)]
    delta: ReadSignal<Option<String>>,
    /// Whether the change is good, bad or neither. Default neither.
    #[props(default)]
    delta_tone: DeltaTone,
    /// Diamond colour beside the label, to tie the card to a chart series.
    jewel: Option<Jewel>,
    /// Context after the change, such as "vs last week".
    caption: Option<String>,
) -> Element {
    let d = delta();
    rsx! {
        div { "data-scope": "stat-card",
            div { "data-part": "head",
                if let Some(j) = jewel {
                    span { "data-part": "diamond", "data-jewel": j.as_str(), "aria-hidden": "true" }
                }
                span { "data-part": "label", "{label}" }
            }
            p { "data-part": "figure",
                span { "data-part": "value", "{value}" }
                if let Some(u) = unit {
                    span { "data-part": "unit", "{u}" }
                }
            }
            if d.is_some() || caption.is_some() {
                div { "data-part": "foot",
                    if let Some(d) = d {
                        span { "data-part": "delta", "data-tone": delta_tone.as_str(), "{d}" }
                    }
                    if let Some(c) = caption {
                        span { "data-part": "caption", "{c}" }
                    }
                }
            }
        }
    }
}
