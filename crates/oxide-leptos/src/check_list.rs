//! `<CheckList>`: automated check results with a summary line.

use leptos::prelude::*;
use oxide_headless::check_list::{overall, summary};

use crate::icon::{Icon, IconName};

pub use oxide_headless::check_list::{Check, CheckState};

/// The state icon. Running is a breathing ring with an sr-only "running".
fn mark(state: CheckState) -> AnyView {
    let s = state.as_str();
    match state {
        CheckState::Pass => view! { <span data-part="mark" data-state=s><Icon name=IconName::Check size=10 /></span> }.into_any(),
        CheckState::Fail => view! { <span data-part="mark" data-state=s><Icon name=IconName::X size=10 /></span> }.into_any(),
        CheckState::Skip => view! { <span data-part="mark" data-state=s></span> }.into_any(),
        CheckState::Running => {
            view! { <span data-part="mark" data-state=s><span data-part="sr">"running"</span></span> }.into_any()
        }
    }
}

/// A list of checks. Failed and skipped rows say so in words, not only colour.
#[component]
pub fn CheckList(
    /// The checks, in order.
    #[prop(into)]
    checks: Signal<Vec<Check>>,
    /// Replaces the generated summary line.
    #[prop(optional, into)]
    summary_text: Option<String>,
) -> impl IntoView {
    view! {
        <div data-scope="check-list">
            {move || {
                let list = checks.get();
                let line = summary_text.clone().unwrap_or_else(|| summary(&list));
                view! {
                    <div data-part="summary" aria-live="polite">{mark(overall(&list))}<span>{line}</span></div>
                    <ul data-part="rows">
                        {list
                            .into_iter()
                            .map(|c| {
                                let note = match c.state {
                                    CheckState::Fail => Some("failed"),
                                    CheckState::Skip => Some("skipped"),
                                    _ => None,
                                };
                                view! {
                                    <li data-part="row" data-state=c.state.as_str()>
                                        {mark(c.state)}
                                        <span data-part="name">{c.name}</span>
                                        {note.map(|n| view! { <span data-part="note">{n}</span> })}
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                }
            }}
        </div>
    }
}
