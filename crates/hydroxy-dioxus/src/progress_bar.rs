//! `ProgressBar`: horizontal progress, with an optional label row and an
//! indeterminate state when `value` is `None`.

use dioxus::prelude::*;
use hydroxy_headless::progress_bar::{self as h, progress_attrs};

pub use crate::card::Jewel;

/// Track thickness.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProgressSize {
    /// 8px.
    #[default]
    Md,
    /// 4px.
    Sm,
}

impl ProgressSize {
    /// The `data-size` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ProgressSize::Md => "md",
            ProgressSize::Sm => "sm",
        }
    }
}

/// Progress towards `max`. Without a `label`, give it a name some other way (e.g. a heading nearby).
#[component]
pub fn ProgressBar(
    /// Current value. `None` means indeterminate.
    #[props(default)]
    value: ReadSignal<Option<f64>>,
    /// The value at completion.
    #[props(default = 100.0)]
    max: f64,
    /// Shown above the bar, and names it for assistive tech.
    #[props(default)]
    label: Option<String>,
    /// Shown value and `aria-valuetext`, such as "22 of 24". Defaults to a rounded percentage.
    #[props(default)]
    value_text: ReadSignal<Option<String>>,
    /// See [`ProgressSize`].
    #[props(default)]
    size: ProgressSize,
    /// Fill colour. Defaults to the accent.
    #[props(default)]
    jewel: Option<Jewel>,
) -> Element {
    let label_id = use_hook(|| format!("{}-label", hydroxy_headless::next_id(h::SCOPE)));
    let a = progress_attrs(value(), max, value_text.read().as_deref());
    let text = a.aria_valuetext.clone().unwrap_or_default();

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-size": size.as_str(),
            "data-jewel": jewel.map(Jewel::as_str),
            "data-state": a.data_state,
            if let Some(l) = &label {
                div { "data-scope": h::SCOPE, "data-part": "header",
                    span { id: "{label_id}", "data-scope": h::SCOPE, "data-part": "label", "{l}" }
                    // The track exposes the same text as aria-valuetext.
                    span { "data-scope": h::SCOPE, "data-part": "value", "aria-hidden": "true", "{text}" }
                }
            }
            div {
                role: a.role,
                "aria-labelledby": label.is_some().then_some(label_id.clone()),
                "aria-valuemin": "0",
                "aria-valuemax": a.aria_valuemax.to_string(),
                "aria-valuenow": a.aria_valuenow.map(|v| v.to_string()),
                "aria-valuetext": a.aria_valuetext.clone(),
                "data-scope": h::SCOPE,
                "data-part": "track",
                div {
                    "data-scope": h::SCOPE,
                    "data-part": "indicator",
                    style: a.percent.map(|p| format!("width: {p}%;")),
                }
            }
        }
    }
}
