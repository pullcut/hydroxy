//! `StepTrack`: a linear stage tracker with diamond nodes.

use dioxus::prelude::*;

pub use hydroxy_headless::jewel::Jewel;

/// Steps in order, filled up to and including `current`.
#[component]
pub fn StepTrack(
    /// Step names, in order.
    steps: Vec<String>,
    /// The current step's index. None: nothing reached yet.
    #[props(default)]
    current: ReadSignal<Option<usize>>,
    /// Fill colour. Default sapphire.
    #[props(default = Jewel::Sapphire)]
    jewel: Jewel,
    /// Accessible name. Default: "Progress".
    #[props(default = "Progress".to_string())]
    label: String,
) -> Element {
    let cur = current();
    rsx! {
        ol { "data-scope": "step-track", "data-jewel": jewel.as_str(), "aria-label": "{label}",
            for (k, name) in steps.into_iter().enumerate() {
                li {
                    "data-part": "step",
                    "data-done": cur.is_some_and(|c| k <= c).then_some(""),
                    "aria-current": (cur == Some(k)).then_some("step"),
                    if k > 0 {
                        span { "data-part": "line", "aria-hidden": "true" }
                    }
                    span { "data-part": "label",
                        span { "data-part": "dot", "aria-hidden": "true" }
                        "{name}"
                        span { "data-part": "sr",
                            if cur == Some(k) {
                                ", current"
                            } else if cur.is_some_and(|c| k < c) {
                                ", done"
                            }
                        }
                    }
                }
            }
        }
    }
}
