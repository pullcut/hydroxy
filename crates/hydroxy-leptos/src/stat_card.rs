//! `<StatCard>`: one key figure with its name, change and caption, for dashboards.

use leptos::prelude::*;

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
    #[prop(into)]
    label: String,
    /// The figure.
    #[prop(into)]
    value: Signal<String>,
    /// Unit after the figure, such as "min".
    #[prop(optional, into)]
    unit: Option<String>,
    /// The change, such as "−4 min" or "+12%".
    #[prop(optional, into)]
    delta: MaybeProp<String>,
    /// Whether the change is good, bad or neither. Default neither.
    #[prop(optional)]
    delta_tone: DeltaTone,
    /// Diamond colour beside the label, to tie the card to a chart series.
    #[prop(optional, into)]
    jewel: Option<Jewel>,
    /// Context after the change, such as "vs last week".
    #[prop(optional, into)]
    caption: Option<String>,
) -> impl IntoView {
    let has_caption = caption.is_some();
    view! {
        <div data-scope="stat-card">
            <div data-part="head">
                {jewel.map(|j| view! { <span data-part="diamond" data-jewel=j.as_str() aria-hidden="true"></span> })}
                <span data-part="label">{label}</span>
            </div>
            <p data-part="figure">
                <span data-part="value">{move || value.get()}</span>
                {unit.map(|u| view! { <span data-part="unit">{u}</span> })}
            </p>
            {move || {
                let d = delta.get();
                (d.is_some() || has_caption).then(|| {
                    view! {
                        <div data-part="foot">
                            {d.map(|d| view! { <span data-part="delta" data-tone=delta_tone.as_str()>{d}</span> })}
                            {caption.clone().map(|c| view! { <span data-part="caption">{c}</span> })}
                        </div>
                    }
                })
            }}
        </div>
    }
}
