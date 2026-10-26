//! `ValueDiff`: before → after. The old value is struck through.

use dioxus::prelude::*;

/// A change from one value to another, such as a renamed status or a moved date.
#[component]
pub fn ValueDiff(
    /// The old value.
    from: String,
    /// The new value.
    to: String,
) -> Element {
    rsx! {
        span { "data-scope": "value-diff",
            span { "data-part": "sr", "Changed from " }
            del { "data-part": "from", "{from}" }
            span { "data-part": "arrow", "aria-hidden": "true", "→" }
            span { "data-part": "sr", " to " }
            ins { "data-part": "to", style: "text-decoration: none;", "{to}" }
        }
    }
}
