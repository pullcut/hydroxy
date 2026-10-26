//! `<ProgressBar>`: horizontal progress, with an optional label row and an
//! indeterminate state when `value` is `None`.

use leptos::prelude::*;
use oxide_headless::progress_bar::{self as h, progress_attrs};

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
    /// Current value. `None` (or unset) means indeterminate.
    #[prop(optional, into)]
    value: MaybeProp<f64>,
    /// The value at completion.
    #[prop(default = 100.0)]
    max: f64,
    /// Shown above the bar, and names it for assistive tech.
    #[prop(optional, into)]
    label: Option<String>,
    /// Shown value and `aria-valuetext`, such as "22 of 24". Defaults to a rounded percentage.
    #[prop(optional, into)]
    value_text: MaybeProp<String>,
    /// See [`ProgressSize`].
    #[prop(optional)]
    size: ProgressSize,
    /// Fill colour. Defaults to the accent.
    #[prop(optional)]
    jewel: Option<Jewel>,
) -> impl IntoView {
    let label_id = format!("{}-label", crate::id::next_id(h::SCOPE));
    let labelled_by = label.is_some().then(|| label_id.clone());
    let a = Memo::new(move |_| progress_attrs(value.get(), max, value_text.get().as_deref()));

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-size=size.as_str()
            data-jewel=jewel.map(Jewel::as_str)
            data-state=move || a.get().data_state
        >
            {label.map(|l| view! {
                <div data-scope=h::SCOPE data-part="header">
                    <span id=label_id data-scope=h::SCOPE data-part="label">{l}</span>
                    // The track exposes the same text as aria-valuetext.
                    <span data-scope=h::SCOPE data-part="value" aria-hidden="true">
                        {move || a.get().aria_valuetext}
                    </span>
                </div>
            })}
            <div
                role=move || a.get().role
                aria-labelledby=labelled_by
                aria-valuemin="0"
                aria-valuemax=move || a.get().aria_valuemax.to_string()
                aria-valuenow=move || a.get().aria_valuenow.map(|v| v.to_string())
                aria-valuetext=move || a.get().aria_valuetext
                data-scope=h::SCOPE
                data-part="track"
            >
                <div
                    data-scope=h::SCOPE
                    data-part="indicator"
                    style=move || a.get().percent.map(|p| format!("width: {p}%;"))
                ></div>
            </div>
        </div>
    }
}
