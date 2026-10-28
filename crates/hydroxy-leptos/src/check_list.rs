//! `<CheckList>`: automated check results with a summary line.

use hydroxy_headless::check_list::{overall, summary};
use leptos::either::EitherOf3;
use leptos::prelude::*;

use crate::icon::{Icon, IconName};

pub use hydroxy_headless::check_list::{Check, CheckState};

/// The state icon. Running is a breathing ring with an sr-only "running".
fn mark(state: CheckState) -> impl IntoView {
    let inner = match state {
        CheckState::Pass => EitherOf3::A(view! { <Icon name=IconName::Check size=10 /> }),
        CheckState::Fail => EitherOf3::A(view! { <Icon name=IconName::X size=10 /> }),
        CheckState::Skip => EitherOf3::B(()),
        CheckState::Running => EitherOf3::C(view! { <span data-part="sr">"running"</span> }),
    };
    view! { <span data-part="mark" data-state=state.as_str()>{inner}</span> }
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
