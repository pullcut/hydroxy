//! `CheckList`: automated check results with a summary line.

use dioxus::prelude::*;
use oxide_headless::check_list::{overall, summary};

use crate::icon::{Icon, IconName};

pub use oxide_headless::check_list::{Check, CheckState};

/// The state icon. Running is a breathing ring with an sr-only "running".
fn mark(state: CheckState) -> Element {
    let s = state.as_str();
    match state {
        CheckState::Pass => rsx! { span { "data-part": "mark", "data-state": s, Icon { name: IconName::Check, size: 10 } } },
        CheckState::Fail => rsx! { span { "data-part": "mark", "data-state": s, Icon { name: IconName::X, size: 10 } } },
        CheckState::Skip => rsx! { span { "data-part": "mark", "data-state": s } },
        CheckState::Running => rsx! { span { "data-part": "mark", "data-state": s, span { "data-part": "sr", "running" } } },
    }
}

/// A list of checks. Failed and skipped rows say so in words, not only colour.
#[component]
pub fn CheckList(
    /// The checks, in order.
    checks: ReadSignal<Vec<Check>>,
    /// Replaces the generated summary line.
    summary_text: Option<String>,
) -> Element {
    let list = checks();
    let line = summary_text.unwrap_or_else(|| summary(&list));
    rsx! {
        div { "data-scope": "check-list",
            div { "data-part": "summary", "aria-live": "polite",
                {mark(overall(&list))}
                span { "{line}" }
            }
            ul { "data-part": "rows",
                for c in list {
                    li { key: "{c.name}", "data-part": "row", "data-state": c.state.as_str(),
                        {mark(c.state)}
                        span { "data-part": "name", "{c.name}" }
                        if c.state == CheckState::Fail {
                            span { "data-part": "note", "failed" }
                        } else if c.state == CheckState::Skip {
                            span { "data-part": "note", "skipped" }
                        }
                    }
                }
            }
        }
    }
}
