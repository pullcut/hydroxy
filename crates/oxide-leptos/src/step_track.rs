//! `<StepTrack>`: a linear stage tracker with diamond nodes.

use leptos::prelude::*;

pub use oxide_headless::jewel::Jewel;

/// Steps in order, filled up to and including `current`.
#[component]
pub fn StepTrack(
    /// Step names, in order.
    #[prop(into)]
    steps: Vec<String>,
    /// The current step's index. None: nothing reached yet.
    #[prop(optional, into)]
    current: MaybeProp<usize>,
    /// Fill colour. Default sapphire.
    #[prop(default = Jewel::Sapphire)]
    jewel: Jewel,
    /// Accessible name. Default: "Progress".
    #[prop(default = "Progress".into(), into)]
    label: String,
) -> impl IntoView {
    let reached = move |k: usize| current.get().is_some_and(|c| k <= c);
    view! {
        <ol data-scope="step-track" data-jewel=jewel.as_str() aria-label=label>
            {steps
                .into_iter()
                .enumerate()
                .map(|(k, name)| {
                    view! {
                        <li
                            data-part="step"
                            data-done=move || reached(k).then_some("")
                            aria-current=move || (current.get() == Some(k)).then_some("step")
                        >
                            {(k > 0).then(|| view! { <span data-part="line" aria-hidden="true"></span> })}
                            <span data-part="label">
                                <span data-part="dot" aria-hidden="true"></span>
                                {name}
                                <span data-part="sr">
                                    {move || if current.get() == Some(k) { ", current" } else if reached(k) { ", done" } else { "" }}
                                </span>
                            </span>
                        </li>
                    }
                })
                .collect_view()}
        </ol>
    }
}
